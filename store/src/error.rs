//! # Error
//!
//! `error` contains the `store` crate `Error` type.

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Not implemented")]
    NotImplemented,
    #[fail(display = "Not allowed")]
    NotAllowed,
    #[fail(display = "Invalid key")]
    InvalidKey,
    #[fail(display = "Invalid value")]
    InvalidValue,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Already found")]
    AlreadyFound,
}
