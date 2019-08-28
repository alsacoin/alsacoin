//! # Error
//!
//! `error` contains the `config` crate `Error` type.

use crypto::error::Error as CryptoError;
use models::error::Error as ModelError;
use serde_cbor;
use serde_json;
use std::convert::From;
use std::io;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Crypto: {}", msg)]
    Crypto { msg: String },
    #[fail(display = "Model: {}", msg)]
    Model { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
    #[fail(display = "Not implemented")]
    NotImplemented,
    #[fail(display = "Not allowed")]
    NotAllowed,
    #[fail(display = "Already found")]
    AlreadyFound,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Invalid kind")]
    InvalidKind,
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

impl From<CryptoError> for Error {
    fn from(error: CryptoError) -> Error {
        let msg = format!("{}", error);
        Error::Crypto { msg }
    }
}

impl From<ModelError> for Error {
    fn from(error: ModelError) -> Error {
        let msg = format!("{}", error);
        Error::Model { msg }
    }
}
