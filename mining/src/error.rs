//! # Error
//!
//! `error` contains the `mining` crate `Error` type.

use crypto::error::Error as CryptoError;
use std::convert::From;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Crypto: {}", msg)]
    Crypto { msg: String },
    #[fail(display = "Out of bound")]
    OutOfBound,
    #[fail(display = "Not found")]
    NotFound,
}

impl From<CryptoError> for Error {
    fn from(error: CryptoError) -> Error {
        let msg = format!("{}", error);
        Error::Crypto { msg }
    }
}
