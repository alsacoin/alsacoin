//! # Error
//!
//! `error` contains the `models` crate `Error` type.

use chrono;
use std::num;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Invalid timestamp")]
    InvalidTimestamp,
    #[fail(display = "Invalid stage")]
    InvalidStage,
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
}

impl From<chrono::ParseError> for Error {
    fn from(err: chrono::ParseError) -> Error {
       let msg = format!("{}", err);
       Error::Parse { msg }
    }
}

impl From<num::ParseIntError> for Error {
    fn from(err: num::ParseIntError) -> Error {
       let msg = format!("{}", err);
       Error::Parse { msg }
    }
}
