//! # Error
//!
//! `error` contains the `config` crate `Error` type.

use config::error::Error as ConfigError;
use serde_cbor;
use serde_json;
use std::convert::From;
use std::io;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Config: {}", msg)]
    Config { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
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

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Error {
        let msg = format!("{}", error);
        Error::Config { msg }
    }
}
