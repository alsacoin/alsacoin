//! # Error
//!
//! `error` contains the `crypto` crate `Error` type.

use base16;
use ed25519_dalek as ed25519;
use rand_core;
use std::convert::From;
use std::io;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
    #[fail(display = "Decode: {}", msg)]
    Decode { msg: String },
    #[fail(display = "Rand: {}", msg)]
    Rand { msg: String },
    #[fail(display = "Out of bound")]
    OutOfBound,
    #[fail(display = "Invalid range")]
    InvalidRange,
    #[fail(display = "Invalid length")]
    InvalidLength,
    #[fail(display = "Scalar: {}", msg)]
    Scalar { msg: String },
    #[fail(display = "PrivateKey: {}", msg)]
    PrivateKey { msg: String },
    #[fail(display = "PublicKey: {}", msg)]
    PublicKey { msg: String },
    #[fail(display = "Keys: {}", msg)]
    Keys { msg: String },
    #[fail(display = "SharedSecret: {}", msg)]
    SharedSecret { msg: String },
    #[fail(display = "Message: {}", msg)]
    Message { msg: String },
    #[fail(display = "CypherText: {}", msg)]
    CypherText { msg: String },
    #[fail(display = "Signature: {}", msg)]
    Signature { msg: String },
    #[fail(display = "BalloonParams: {}", msg)]
    BalloonParams { msg: String },
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        let msg = format!("{}", err);
        Error::IO { msg }
    }
}

impl From<rand_core::Error> for Error {
    fn from(err: rand_core::Error) -> Error {
        let msg = format!("{}", err);
        Error::Rand { msg }
    }
}

impl From<base16::DecodeError> for Error {
    fn from(err: base16::DecodeError) -> Error {
        let msg = format!("{}", err);
        Error::Decode { msg }
    }
}

impl From<ed25519::SignatureError> for Error {
    fn from(err: ed25519::SignatureError) -> Error {
        let msg = format!("{}", err);
        Error::Signature { msg }
    }
}
