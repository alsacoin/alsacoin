//! # Common
//!
//! `common` contains the CLI common functionalities.

use clap::{App, AppSettings, Arg};
use models::version::VERSION;

/// `app` creates and returns a clap `App`.
pub fn app(name: &'static str, about: &'static str) -> App<'static, 'static> {
    App::new(name).version(VERSION).about(about).settings(&[
        AppSettings::ArgsNegateSubcommands,
        AppSettings::ArgRequiredElseHelp,
        AppSettings::ColorAuto,
        AppSettings::DontCollapseArgsInUsage,
        AppSettings::DeriveDisplayOrder,
        AppSettings::GlobalVersion,
        AppSettings::StrictUtf8,
        AppSettings::VersionlessSubcommands,
    ])
}

/// `add_verbose` adds a verbose option to a command.
pub fn add_verbose(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name("verbose")
            .help("Turns on verbose output")
            .short("V")
            .long("verbose")
            .takes_value(false)
            .required(false),
    )
}
