use anyhow::{anyhow, Context, Result};
use clap_complete::{generate, Shell};
use std::io;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;

mod agent;
mod client;
mod commands;
mod config;
mod notifier;
mod pull_request;
mod runner;

use crate::pull_request::PullRequest;
use commands::app;

lazy_static! {
    static ref REPOS: Mutex<Vec<PullRequest>> = Mutex::new(Vec::new());
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let mut app = app();

    let matches = app.clone().get_matches();
    // Shell completion generation is completely independent, so perform it before
    // any config or subcommand operations.
    if let Ok(shell) = matches.value_of_t::<Shell>("completions") {
        generate(shell, &mut app, env!("CARGO_PKG_NAME"), &mut io::stdout());
        return Ok(());
    }

    match std::env::var("GITHUB_API_TOKEN") {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Problem parsing arguments: {}", err);
            std::process::exit(1);
        }
    }

    if let Some(("agent", matches)) = matches.subcommand() {
        return commands::agent(matches);
    }

    if matches.subcommand().is_none() {
        return Ok(());
    }

    match matches.subcommand() {
        Some(("list", matches)) => commands::list(matches)?,
        Some(("push", matches)) => commands::push(matches)?,
        Some(("pop", matches)) => commands::pop(matches)?,
        Some(("clear", matches)) => commands::clear(matches)?,
        Some(("force", matches)) => commands::force(matches)?,
        _ => unreachable!(),
    }

    Ok(())
}
