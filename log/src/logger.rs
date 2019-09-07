//! # Logger
//!
//! `logger` is the module containing the logger type and functions.

use crate::file::LogFile;
use crate::format::LogFormat;
use crate::level::LogLevel;
use crate::result::Result;
use config::log_config::LogConfig;
use serde::{Deserialize, Serialize};

/// `Logger` is the logger type used in Alsacoin.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Hash, Serialize, Deserialize)]
pub struct Logger {
    level: LogLevel,
    format: LogFormat,
    file: LogFile,
}

impl Logger {
    /// `new` creates a new `Logger`.
    pub fn new(level: LogLevel, format: LogFormat, file: &LogFile) -> Logger {
        Logger {
            level,
            format,
            file: file.to_owned(),
        }
    }

    /// `from_config` creates a new `Logger` from a `LogConfig`.
    pub fn from_config(config: &LogConfig) -> Result<Logger> {
        config.validate()?;

        let mut config = config.clone();
        config.populate();

        let level = LogLevel::parse(&config.level.unwrap())?;
        let format = LogFormat::parse(&config.format.unwrap())?;
        let file = LogFile::parse(&config.file.unwrap());

        let logger = Logger {
            level,
            format,
            file,
        };

        Ok(logger)
    }
}
