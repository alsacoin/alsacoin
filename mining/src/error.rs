//! # Error
//!
//! `error` contains the `mining` crate `Error` type.

use crypto::error::Error as CryptoError;
use serde_cbor;
use serde_json;
use std::convert::From;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Crypto: {}", msg)]
    Crypto { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
    #[fail(display = "Out of bound")]
    OutOfBound,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Invalid mining solution")]
    InvalidMiningSolution,
}

impl From<CryptoError> for Error {
    fn from(error: CryptoError) -> Error {
        let msg = format!("{}", error);
        Error::Crypto { msg }
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
