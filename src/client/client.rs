use crate::agent::{Agent, Message, Request, RequestBody, Response, PROTOCOL_VERSION};
use anyhow::Result;
use std::os::unix::net::UnixStream;

pub struct Client {
    stream: UnixStream,
}

impl Client {
    pub fn new() -> Result<Self> {
        let stream = UnixStream::connect(Agent::path())?;
        Ok(Self { stream })
    }

    /// Issue the given request to the agent, returning the agent's `Response`.
    fn request(&self, body: RequestBody) -> Result<Response> {
        #[allow(clippy::redundant_field_names)]
        let req = Request {
            protocol: PROTOCOL_VERSION,
            body,
        };
        req.write(&self.stream)?;
        let resp = Response::read(&self.stream)?;
        Ok(resp)
    }

    /// Ask the agent to quit gracefully.
    pub fn quit_agent(self) -> Result<()> {
        self.request(RequestBody::Quit)?;
        Ok(())
    }

    /// Ask the agent to quit gracefully.
    pub fn push(self, url: String) -> Result<()> {
        self.request(RequestBody::Push(url))?;
        Ok(())
    }

    /// Ask the agent to quit gracefully.
    pub fn pop(self, url: String) -> Result<()> {
        self.request(RequestBody::Pop(url))?;
        Ok(())
    }

    /// Ask the agent to quit gracefully.
    pub fn clear(self) -> Result<()> {
        self.request(RequestBody::Clear)?;
        Ok(())
    }

    /// Ask the agent to quit gracefully.
    pub fn force(self) -> Result<()> {
        log::info!("Client-Command: Force");
        self.request(RequestBody::ForceProcess)?;
        Ok(())
    }

    /// Ask the agent to quit gracefully.
    pub fn list(self) -> Result<()> {
        let response = self.request(RequestBody::List)?;
        match response {
            Response::List(list) => {
                println!("Queue: \n{}", list.join("\n"));
            }
            _ => {}
        }
        Ok(())
    }
}
