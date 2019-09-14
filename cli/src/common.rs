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

/// `add_verbose_option` adds a verbose option to a command.
pub fn add_verbose_option(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name("verbose")
            .help("Turns on verbose output")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .required(false),
    )
}

/// `add_config_option` adds a config option to a command.
pub fn add_config_option(app: App<'static, 'static>) -> App<'static, 'static> {
    app.arg(
        Arg::with_name("config")
            .help("Sets a custom config file")
            .short("c")
            .long("config")
            .takes_value(true)
            .value_name("FILE")
            .required(false),
    )
}
