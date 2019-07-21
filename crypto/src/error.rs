//! # Error
//!
//! `error` contains the `crypto` crate `Error` type.

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "IO: {}", msg)]
    IO { msg: String },
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
}
