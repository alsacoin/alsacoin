//! # Error
//!
//! `error` contains the `mining` crate `Error` type.

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "out of bound")]
    OutOfBound,
}
