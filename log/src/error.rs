//! # Error
//!
//! `error` contains the `config` crate `Error` type.

use config::error::Error as ConfigError;
use models::error::Error as ModelError;
use serde_cbor;
use serde_json;
use std::convert::From;
use std::io;
use term;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Model: {}", msg)]
    Model { msg: String },
    #[fail(display = "Config: {}", msg)]
    Config { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
    #[fail(display = "Invalid level")]
    InvalidLevel,
    #[fail(display = "Invalid format")]
    InvalidFormat,
    #[fail(display = "Invalid file")]
    InvalidFile,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<term::Error> for Error {
    fn from(error: term::Error) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Error {
        let msg = format!("{}", err);
        Error::Parse { msg }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Error {
        let msg = format!("{}", err);
        Error::Parse { msg }
    }
}

impl From<ModelError> for Error {
    fn from(error: ModelError) -> Error {
        let msg = format!("{}", error);
        Error::Model { msg }
    }
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Error {
        let msg = format!("{}", error);
        Error::Config { msg }
    }
}
