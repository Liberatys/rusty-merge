use crate::agent::Agent;
use crate::client::Client;
use crate::config::Configuration;
use anyhow::Result;
use clap::ArgMatches;
use daemonize_me::Daemon;

pub fn agent(matches: &ArgMatches) -> Result<()> {
    let config: Configuration = crate::config::load()?;
    crate::config::store(&config)?;

    // No subcommand: run the agent itself
    if matches.subcommand().is_none() {
        let mut agent = Agent::new(config)?;
        if !matches.is_present("foreground") {
            Daemon::new().start()?;
        }
        log::info!("Starting agent");
        agent.run()?;

        return Ok(());
    }

    match matches.subcommand() {
        Some(("kill", matches)) => agent_kill(matches),
        _ => unreachable!(),
    }
}

fn agent_kill(_matches: &ArgMatches) -> Result<()> {
    log::info!("Killing agent");

    let client = Client::new()?;
    client.quit_agent()?;

    Ok(())
}
