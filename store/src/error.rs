//! # Error
//!
//! `error` contains the `store` crate `Error` type.

use std::convert::From;
use std::io;
use unqlite::Error as UnQLiteError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Store: {}", msg)]
    Store { msg: String },
    #[fail(display = "Not implemented")]
    NotImplemented,
    #[fail(display = "Not allowed")]
    NotAllowed,
    #[fail(display = "Invalid key")]
    InvalidKey,
    #[fail(display = "Invalid value")]
    InvalidValue,
    #[fail(display = "Invalid range")]
    InvalidRange,
    #[fail(display = "Invalid length")]
    InvalidLength,
    #[fail(display = "Invalid size")]
    InvalidSize,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Already found")]
    AlreadyFound,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<UnQLiteError> for Error {
    fn from(error: UnQLiteError) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}
