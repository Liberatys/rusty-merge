use crate::client::Client;
use crate::pull_request::PullRequest;
use anyhow::Result;
use clap::ArgMatches;

pub fn list(_matches: &ArgMatches) -> Result<()> {
    log::info!("Command: List");

    let client = Client::new()?;
    client.list()?;

    Ok(())
}

pub fn clear(_matches: &ArgMatches) -> Result<()> {
    log::info!("Command: Clear");

    let client = Client::new()?;
    client.clear()?;

    Ok(())
}

pub fn push(matches: &ArgMatches) -> Result<()> {
    log::info!("Command: Push");

    if let Some(url) = matches.value_of("URL") {
        if !PullRequest::valid(url) {
            panic!("{} is an invalid pull request source", url);
        }

        let client = Client::new()?;
        client.push(url.to_string())?;
    }

    Ok(())
}

pub fn force(matches: &ArgMatches) -> Result<()> {
    let client = Client::new()?;
    log::info!("Command: Force");
    client.force()?;

    Ok(())
}

pub fn pop(matches: &ArgMatches) -> Result<()> {
    if let Some(url) = matches.value_of("URL") {
        if !PullRequest::valid(url) {
            panic!("{} is an invalid pull request source", url);
        }
        let client = Client::new()?;
        client.pop(url.to_string())?;
    }

    Ok(())
}
