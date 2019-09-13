//! # Common
//!
//! `common` contains the CLI common functionalities.

use clap::{App, AppSettings};
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
