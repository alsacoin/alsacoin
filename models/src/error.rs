//! # Error
//!
//! `error` contains the `models` crate `Error` type.

use chrono;
use regex;
use std::num;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Chrono: {}", msg)]
    Chrono { msg: String },
    #[fail(display = "Regex: {}", msg)]
    Regex { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
    #[fail(display = "No regex match")]
    NoRegexMatch,
    #[fail(display = "Invalid version")]
    InvalidVersion,
    #[fail(display = "Invalid stage")]
    InvalidStage,
    #[fail(display = "Invalid timestamp")]
    InvalidTimestamp,
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
