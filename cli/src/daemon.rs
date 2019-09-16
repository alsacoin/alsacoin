//! # Daemon
//!
//! `daemon` contains the CLI daemon type and functions.

use crate::common;
use crate::result::Result;
use clap::{App, Arg, ArgMatches, SubCommand};

/// `add_start` adds a start command to `App`.
fn add_start(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("start")
        .about("Starts the daemon")
        .arg(
            Arg::with_name("without-consensus")
                .help("Turns off the consensus server")
                .long("without-consensus")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("without-miner")
                .help("Turns off the miner server")
                .long("without-miner")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("without-client")
                .help("Turns off the client server")
                .long("without-client")
                .takes_value(false)
                .required(false),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_stop` adds a start command to `App`.
fn add_stop(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("stop").about("Stops the daemon");

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `add_restart` adds a start command to `App`.
fn add_restart(app: App<'static, 'static>) -> App<'static, 'static> {
    let mut cmd = SubCommand::with_name("start")
        .about("Restarts the daemon")
        .arg(
            Arg::with_name("without-consensus")
                .help("Turns off the consensus server")
                .long("without-consensus")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("without-miner")
                .help("Turns off the miner server")
                .long("without-miner")
                .takes_value(false)
                .required(false),
        )
        .arg(
            Arg::with_name("without-client")
                .help("Turns off the client server")
                .long("without-client")
                .takes_value(false)
                .required(false),
        );

    cmd = common::add_verbose(cmd);

    app.subcommand(cmd)
}

/// `CliDaemon` is the type of the CLI daemon.
pub struct CliDaemon {}

impl CliDaemon {
    /// `CLI_NAME` is the CLI daemon app name.
    pub const CLI_NAME: &'static str = "alsad";

    /// `CLI_ABOUT` is the CLI daemon app description.
    pub const CLI_ABOUT: &'static str = "Alsacoin daemon";

    /// `app` returns the `CliDaemon` clap `App`.
    pub fn app() -> App<'static, 'static> {
        let mut app = common::app(Self::CLI_NAME, Self::CLI_ABOUT);
        app = add_start(app);
        app = add_stop(app);
        add_restart(app)
    }

    /// `args` returns the `CliDaemon` clap `ArgMatches`.
    pub fn args() -> ArgMatches<'static> {
        CliDaemon::app().get_matches()
    }

    /// `init` inits the `CliDaemon` environment.
    pub fn init() -> Result<()> {
        common::init()
    }

    /// `reset` resets the `CliDaemon` environment.
    pub fn reset() -> Result<()> {
        common::reset()
    }

    /// `clean` cleans the `CliDaemon` environment.
    pub fn clean() -> Result<()> {
        common::destroy()
    }

    /// `run` runs the `CliDaemon` application.
    pub fn run() -> Result<()> {
        CliDaemon::init()?;

        let _matches = CliDaemon::args();

        Ok(())
    }
}
