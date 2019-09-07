//! # Log Config
//!
//! `log_config` is the module containing the log configuration type and functions.

use crate::error::Error;
use crate::result::Result;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use toml;

/// `LogConfig` is the type representing a log configuration.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: Option<String>,
    pub format: Option<String>,
    pub file: Option<String>,
    pub color: Option<bool>,
}

impl LogConfig {
    /// `VALID_LEVELS` sets the valid log levels.
    pub const VALID_LEVELS: &'static [&'static str] = &["debug", "critical"];

    /// `DEFAULT_LEVEL` is the default log level.
    pub const DEFAULT_LEVEL: &'static str = "critical";

    /// `VALID_FORMATS` sets the valid log formats.
    pub const VALID_FORMATS: &'static [&'static str] = &["raw", "json", "cbor"];

    /// `DEFAULT_FORMAT` is the default log format.
    pub const DEFAULT_FORMAT: &'static str = "raw";

    /// `DEFAULT_FILE` is the default log file.
    pub const DEFAULT_FILE: &'static str = "stderr";

    /// `DEFAULT_COLOR` is the default color option.
    pub const DEFAULT_COLOR: bool = false;

    /// `new` creates a new `LogConfig`.
    pub fn new(
        level: Option<String>,
        format: Option<String>,
        file: Option<String>,
        color: Option<bool>,
    ) -> Result<LogConfig> {
        let level = if let Some(level) = level {
            if !Self::VALID_LEVELS.contains(&level.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }

            level
        } else {
            Self::DEFAULT_LEVEL.into()
        };

        let format = if let Some(format) = format {
            if !Self::VALID_FORMATS.contains(&format.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }

            format
        } else {
            Self::DEFAULT_FORMAT.into()
        };

        let file = Some(file.unwrap_or(Self::DEFAULT_FILE.into()));

        let color = Some(color.unwrap_or(Self::DEFAULT_COLOR.into()));

        let config = LogConfig {
            level: Some(level),
            format: Some(format),
            file,
            color,
        };

        Ok(config)
    }

    /// `populate` populates the `None` fields in the `LogConfig` when there are
    /// defaults.
    pub fn populate(&mut self) {
        if self.level.is_none() {
            self.level = Some(Self::DEFAULT_LEVEL.into());
        }

        if self.format.is_none() {
            self.format = Some(Self::DEFAULT_FORMAT.into());
        }

        if self.file.is_none() {
            self.file = Some(Self::DEFAULT_FILE.into());
        }

        if self.color.is_none() {
            self.color = Some(Self::DEFAULT_COLOR.into());
        }
    }

    /// `validate` validates the `LogConfig`.
    pub fn validate(&self) -> Result<()> {
        if let Some(ref level) = self.level {
            if !Self::VALID_LEVELS.contains(&level.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }

        if let Some(ref format) = self.format {
            if !Self::VALID_FORMATS.contains(&format.as_str()) {
                let err = Error::InvalidKind;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `to_bytes` converts the `LogConfig` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `LogConfig`.
    pub fn from_bytes(b: &[u8]) -> Result<LogConfig> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `LogConfig` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `LogConfig`.
    pub fn from_json(s: &str) -> Result<LogConfig> {
        serde_json::from_str(s).map_err(|e| e.into())
    }

    /// `to_toml` converts the `LogConfig` into a TOML string.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string(self).map_err(|e| e.into())
    }

    /// `from_toml` converts a TOML string into an `LogConfig`.
    pub fn from_toml(s: &str) -> Result<LogConfig> {
        toml::from_str(s).map_err(|e| e.into())
    }
}

impl Default for LogConfig {
    fn default() -> LogConfig {
        let level = Some(LogConfig::DEFAULT_LEVEL.into());
        let format = Some(LogConfig::DEFAULT_FORMAT.into());
        let file = Some(LogConfig::DEFAULT_FILE.into());
        let color = Some(LogConfig::DEFAULT_COLOR.into());

        LogConfig {
            level,
            format,
            file,
            color,
        }
    }
}

#[test]
fn test_log_config_new() {
    let invalid_level: String = "level".into();
    let invalid_format: String = "format".into();

    let res = LogConfig::new(Some(invalid_level.into()), None, None, None);
    assert!(res.is_err());

    let res = LogConfig::new(None, Some(invalid_format.into()), None, None);
    assert!(res.is_err());

    for level in LogConfig::VALID_LEVELS.iter().copied() {
        for format in LogConfig::VALID_FORMATS.iter().copied() {
            let res = LogConfig::new(Some(level.into()), Some(format.into()), None, None);
            assert!(res.is_ok());
        }
    }
}

#[test]
fn test_log_config_validate() {
    let mut config = LogConfig::default();

    let res = config.validate();
    assert!(res.is_ok());

    config.level = None;
    let res = config.validate();
    assert!(res.is_ok());

    config.format = None;
    let res = config.validate();
    assert!(res.is_ok());

    config.populate();
    let res = config.validate();
    assert!(res.is_ok());

    config.level = Some("".into());
    let res = config.validate();
    assert!(res.is_err());

    config.level = None;

    config.format = Some("".into());
    let res = config.validate();
    assert!(res.is_err());
}

#[test]
fn test_log_config_serialize_bytes() {
    let config_a = LogConfig::default();

    let res = config_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = LogConfig::from_bytes(&cbor);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_log_config_serialize_json() {
    let config_a = LogConfig::default();

    let res = config_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = LogConfig::from_json(&json);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}

#[test]
fn test_log_config_serialize_toml() {
    let config_a = LogConfig::default();

    let res = config_a.to_toml();
    assert!(res.is_ok());
    let toml = res.unwrap();

    let res = LogConfig::from_toml(&toml);
    assert!(res.is_ok());
    let config_b = res.unwrap();

    assert_eq!(config_a, config_b)
}
