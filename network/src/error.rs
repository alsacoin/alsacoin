//! # Error
//!
//! `error` contains the `store` crate `Error` type.

use crypto::error::Error as CryptoError;
use mining::error::Error as MiningError;
use models::error::Error as ModelError;
use models::node::Node;
use std::convert::From;
use std::io;
use std::sync::mpsc::{RecvError, SendError};
use store::error::Error as StoreError;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Crypto: {}", msg)]
    Crypto { msg: String },
    #[fail(display = "Mining: {}", msg)]
    Mining { msg: String },
    #[fail(display = "Store: {}", msg)]
    Store { msg: String },
    #[fail(display = "Model: {}", msg)]
    Model { msg: String },
    #[fail(display = "Invalid id")]
    InvalidId,
    #[fail(display = "Not implemented")]
    NotImplemented,
    #[fail(display = "Not allowed")]
    NotAllowed,
    #[fail(display = "Already found")]
    AlreadyFound,
    #[fail(display = "Not found")]
    NotFound,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<SendError<(Node, Vec<u8>)>> for Error {
    fn from(error: SendError<(Node, Vec<u8>)>) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<RecvError> for Error {
    fn from(error: RecvError) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<CryptoError> for Error {
    fn from(error: CryptoError) -> Error {
        let msg = format!("{}", error);
        Error::Crypto { msg }
    }
}

impl From<MiningError> for Error {
    fn from(error: MiningError) -> Error {
        let msg = format!("{}", error);
        Error::Mining { msg }
    }
}

impl From<StoreError> for Error {
    fn from(error: StoreError) -> Error {
        let msg = format!("{}", error);
        Error::Store { msg }
    }
}

impl From<ModelError> for Error {
    fn from(error: ModelError) -> Error {
        let msg = format!("{}", error);
        Error::Model { msg }
    }
}
