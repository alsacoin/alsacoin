//! # Daemon
//!
//! `daemon` contains the CLI daemon type and functions.

use crate::common;
use clap::{App, ArgMatches};

/// `CliDaemon` is the type of the CLI daemon.
pub struct CliDaemon {}

impl CliDaemon {
    /// `CLI_DAEMON_NAME` is the CLI daemon app name.
    pub const CLI_DAEMON_NAME: &'static str = "alsad";

    /// `CLI_DAEMON_ABOUT` is the CLI daemon app description.
    pub const CLI_DAEMON_ABOUT: &'static str = "Alsacoin daemon";

    /// `app` returns the `CliDaemon` clap `App`.
    pub fn app() -> App<'static, 'static> {
        common::app(Self::CLI_DAEMON_NAME, Self::CLI_DAEMON_ABOUT)
    }

    /// `args` returns the `CliDaemon` clap `ArgMatches`.
    pub fn args() -> ArgMatches<'static> {
        CliDaemon::app().get_matches()
    }
}
