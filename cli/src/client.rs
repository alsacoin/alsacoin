//! # Client
//!
//! `client` contains the CLI client type and functions.

use crate::common;
use clap::{App, ArgMatches};

/// `CliClient` is the type of the CLI client.
pub struct CliClient {}

impl CliClient {
    /// `CLI_CLIENT_NAME` is the CLI client app name.
    pub const CLI_CLIENT_NAME: &'static str = "alsac";

    /// `CLI_CLIENT_ABOUT` is the CLI client app description.
    pub const CLI_CLIENT_ABOUT: &'static str = "Alsacoin client";

    /// `app` returns the `CliClient` clap `App`.
    pub fn app() -> App<'static, 'static> {
        common::app(Self::CLI_CLIENT_NAME, Self::CLI_CLIENT_ABOUT)
    }

    /// `args` returns the `CliClient` clap `ArgMatches`.
    pub fn args() -> ArgMatches<'static> {
        CliClient::app().get_matches()
    }
}
