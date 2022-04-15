use clap::{arg, command};
use clap::{Arg, ArgMatches, Command, ValueHint};

pub fn app() -> Command<'static> {
    Command::new(env!("CARGO_PKG_NAME"))
        .allow_external_subcommands(true)
        .arg_required_else_help(true)
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .subcommand(
            Command::new("agent")
                .about("run the agent")
                .arg(
                    Arg::new("foreground")
                        .help("run the agent in the foreground")
                        .short('F')
                        .long("foreground"),
                )
                .subcommand(Command::new("kill").about("kills the background agent")),
        )
        .subcommand(Command::new("clear").about("clear the current queue"))
        .subcommand(Command::new("force").about("force the current queue process"))
        .subcommand(Command::new("list").about("list the current queue"))
        .subcommand(
            Command::new("push")
                .about("push a pull request into the queue")
                .arg(arg!(<URL>)),
        )
        .subcommand(
            Command::new("pop")
                .about("pop a pull request from the queue")
                .arg(arg!(<URL>)),
        )
}
