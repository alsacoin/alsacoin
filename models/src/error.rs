//! # Error
//!
//! `error` contains the `models` crate `Error` type.

use chrono;
use crypto;
use mining;
use regex;
use serde_cbor;
use serde_json;
use std::io;
use std::num;
use store;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Chrono: {}", msg)]
    Chrono { msg: String },
    #[fail(display = "Regex: {}", msg)]
    Regex { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
    #[fail(display = "Crypto: {}", msg)]
    Crypto { msg: String },
    #[fail(display = "Mining: {}", msg)]
    Mining { msg: String },
    #[fail(display = "Store: {}", msg)]
    Store { msg: String },
    #[fail(display = "Out of bound")]
    OutOfBound,
    #[fail(display = "No regex match")]
    NoRegexMatch,
    #[fail(display = "Invalid version")]
    InvalidVersion,
    #[fail(display = "Invalid stage")]
    InvalidStage,
    #[fail(display = "Invalid timestamp")]
    InvalidTimestamp,
    #[fail(display = "Invalid id")]
    InvalidId,
    #[fail(display = "Invalid public key")]
    InvalidPublicKey,
    #[fail(display = "Invalid signature")]
    InvalidSignature,
    #[fail(display = "Invalid checksum")]
    InvalidChecksum,
    #[fail(display = "Not implemented")]
    NotImplemented,
    #[fail(display = "Already found")]
    AlreadyFound,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Invalid balance")]
    InvalidBalance,
    #[fail(display = "Invalid fee")]
    InvalidFee,
    #[fail(display = "Invalid address")]
    InvalidAddress,
    #[fail(display = "Invalid distance")]
    InvalidDistance,
    #[fail(display = "Invalid difficulty")]
    InvalidDifficulty,
    #[fail(display = "Invalid coinbase")]
    InvalidCoinbase,
    #[fail(display = "Invalid threshold")]
    InvalidThreshold,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
    }
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
        let msg = format!("{}", err);
        Error::Chrono { msg }
    }
}

impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Error {
        let msg = format!("{}", err);
        Error::Regex { msg }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
        let msg = format!("{}", err);
        Error::Parse { msg }
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

impl From<crypto::error::Error> for Error {
    fn from(err: crypto::error::Error) -> Error {
        let msg = format!("{}", err);
        Error::Crypto { msg }
    }
}

impl From<mining::error::Error> for Error {
    fn from(err: mining::error::Error) -> Error {
        let msg = format!("{}", err);
        Error::Mining { msg }
    }
}

impl From<store::error::Error> for Error {
    fn from(err: store::error::Error) -> Error {
        let msg = format!("{}", err);
        Error::Store { msg }
    }
}
