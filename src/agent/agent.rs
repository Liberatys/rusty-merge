use crate::config::Configuration;
use crate::pull_request::PullRequest;
use crate::runner::Runner;
use crate::REPOS;
use anyhow::{anyhow, Context, Result};
use clokwerk::ScheduleHandle;
use clokwerk::{Scheduler, TimeUnits};
use nix::unistd::Uid;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::task;

use super::{FailureKind, Message, Request, RequestBody, Response, PROTOCOL_VERSION};

/// Represents the state in a running agent.
pub struct Agent {
    /// The local path to the Unix domain socket.
    agent_path: PathBuf,

    /// The scheduler
    scheduler: Option<ScheduleHandle>,

    /// Whether or not the agent intends to quit momentarily.
    quitting: bool,

    /// The configuration read from a file
    configuration: Configuration,
}

impl Agent {
    /// Initializes a new agent without accepting connections.
    pub fn new(configuration: Configuration) -> Result<Self> {
        let agent_path = Self::path();

        if agent_path.exists() {
            return Err(anyhow!(
                "an agent is already running or didn't exit cleanly"
            ));
        }

        #[allow(clippy::redundant_field_names)]
        Ok(Self {
            agent_path,
            quitting: false,
            scheduler: None,
            configuration: configuration,
        })
    }

    pub fn path() -> PathBuf {
        let mut agent_path = PathBuf::from("/tmp");
        agent_path.push(format!(
            "{}-agent-{}",
            env!("CARGO_PKG_NAME"),
            whoami::username()
        ));

        agent_path
    }

    fn setup_process(&mut self) -> Result<()> {
        let mut scheduler = Scheduler::new();
        let configuration: Configuration = self.configuration.clone();

        let scheduler_interval = (self.configuration.queue.interval_in_minutes as u32).minutes();

        scheduler.every(scheduler_interval).run(move || {
            // TODO: Fix double clone
            Self::run_process(configuration.clone(), false);
        });

        let thread_handle = scheduler.watch_thread(Duration::from_millis(1000));
        self.scheduler = Some(thread_handle);

        Ok(())
    }

    fn run_process(configuration: Configuration, forced: bool) {
        log::info!("Starting queue process");

        let closure = || async {
            let mut runner = Runner::new(configuration).unwrap();

            let pull_requests = crate::REPOS.lock().unwrap().to_vec();

            let _ = runner.process(pull_requests).await;

            runner.cleanup(&mut crate::REPOS.lock().unwrap());
        };

        if forced {
            task::spawn(closure());
        } else {
            let rt = Runtime::new().unwrap();

            rt.block_on(closure());
        }
    }

    /// Spawns a new agent as a daemon process, returning once the daemon
    /// is ready to begin serving clients.
    pub fn spawn() -> Result<()> {
        let agent_path = Self::path();

        if agent_path.exists() {
            return Ok(());
        }

        let (uid, euid) = (Uid::current(), Uid::effective());
        if uid.is_root() || uid != euid {
            return Err(anyhow!(
                "unusual UID or UID/EUID pair found, refusing to spawn"
            ));
        }

        let rusty_merger =
            std::env::current_exe().with_context(|| "failed to locate the rusty-merge binary")?;

        let _ = Command::new(rusty_merger)
            .arg("agent")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        for _attempt in 0..10 {
            thread::sleep(Duration::from_millis(10));
            if agent_path.exists() {
                return Ok(());
            }
        }

        Err(anyhow!("agent spawn timeout exhausted"))
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly",
    ))]
    fn auth_client(&self, stream: &UnixStream) -> bool {
        use nix::unistd;

        if let Ok((peer_uid, _)) = unistd::getpeereid(stream.as_raw_fd()) {
            peer_uid == Uid::effective()
        } else {
            false
        }
    }

    /// Handles a single client connection.
    /// Individual clients may issue multiple requests in a single session.
    fn handle_client(&mut self, stream: UnixStream) {
        let reader = BufReader::new(&stream);
        let mut writer = BufWriter::new(&stream);

        if !self.auth_client(&stream) {
            // This can fail, but we don't care.
            let _ = Response::Failure(FailureKind::Auth).write(&mut writer);
            return;
        }

        for line in reader.lines() {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    // This can fail, but we don't care.
                    let _ = Response::Failure(FailureKind::Io(e.to_string())).write(&mut writer);
                    return;
                }
            };

            let req: Request = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    let _ =
                        Response::Failure(FailureKind::Malformed(e.to_string())).write(&mut writer);
                    return;
                }
            };

            if req.protocol != PROTOCOL_VERSION {
                let _ = Response::Failure(FailureKind::VersionMismatch(PROTOCOL_VERSION))
                    .write(&mut writer);
                return;
            }

            let resp = match req.body {
                RequestBody::Quit => {
                    self.quitting = true;
                    Response::Success("OK".into())
                }
                RequestBody::Push(url) => {
                    match PullRequest::new(url) {
                        Ok(pull_reqeuest) => {
                            REPOS.lock().unwrap().push(pull_reqeuest);
                        }
                        Err(_) => {}
                    }
                    Response::Success("OK".into())
                }
                RequestBody::Pop(url) => {
                    let mut locked = REPOS.lock().unwrap();
                    if let Some(pos) = locked.iter().position(|x| {
                        (*x).same(&PullRequest {
                            url: Some(url.clone()),
                            ..PullRequest::default()
                        })
                    }) {
                        locked.remove(pos);
                    }
                    Response::Success("OK".into())
                }
                RequestBody::ForceProcess => {
                    log::info!("Result of forced process");

                    let result = Agent::run_process(self.configuration.clone(), true);

                    log::info!("Result of forced process: {:?}", result);

                    Response::Success("OK".into())
                }
                RequestBody::List => Response::List(
                    REPOS
                        .lock()
                        .unwrap()
                        .iter()
                        .map(|pr| pr.url.as_ref().unwrap())
                        .map(|v| v.to_string())
                        .collect(),
                ),
                RequestBody::Clear => {
                    REPOS.lock().unwrap().clear();
                    Response::Success("OK".into())
                }
            };

            // This can fail, but we don't care.
            let _ = resp.write(&mut writer);
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let listener = UnixListener::bind(&self.agent_path)?;

        self.setup_process()?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.handle_client(stream);
                    if self.quitting {
                        break;
                    }
                }
                Err(_e) => {
                    continue;
                }
            }
        }

        Ok(())
    }
}

impl Drop for Agent {
    fn drop(&mut self) {
        #[allow(clippy::expect_used)]
        fs::remove_file(Agent::path()).expect("attempted to remove missing agent socket");
    }
}
