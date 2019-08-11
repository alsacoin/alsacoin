//! # Error
//!
//! `error` contains the `store` crate `Error` type.

use rkv::error::{DataError, MigrateError, StoreError};
use rkv::{Manager, Rkv};
use std::convert::From;
use std::io;
use std::sync::{PoisonError, RwLockReadGuard, RwLockWriteGuard};
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

impl From<PoisonError<RwLockReadGuard<'_, Rkv>>> for Error {
    fn from(error: PoisonError<RwLockReadGuard<'_, Rkv>>) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, Manager>>> for Error {
    fn from(error: PoisonError<RwLockWriteGuard<'_, Manager>>) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}

impl From<UnQLiteError> for Error {
    fn from(error: UnQLiteError) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}

impl From<StoreError> for Error {
    fn from(error: StoreError) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}

impl From<DataError> for Error {
    fn from(error: DataError) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}

impl From<MigrateError> for Error {
    fn from(error: MigrateError) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}
