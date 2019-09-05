//! # Error
//!
//! `error` contains the `consensus` crate `Error` type.

use crypto::error::Error as CryptoError;
use mining::error::Error as MiningError;
use models::error::Error as ModelError;
use network::error::Error as NetworkError;
use config::error::Error as ConfigError;
use serde_cbor;
use serde_json;
use std::convert::From;
use std::io;
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
    #[fail(display = "Network: {}", msg)]
    Network { msg: String },
    #[fail(display = "Config: {}", msg)]
    Config { msg: String },
    #[fail(display = "Parse: {}", msg)]
    Parse { msg: String },
    #[fail(display = "Invalid id")]
    InvalidId,
    #[fail(display = "Invalid length")]
    InvalidLength,
    #[fail(display = "Not implemented")]
    NotImplemented,
    #[fail(display = "Not allowed")]
    NotAllowed,
    #[fail(display = "Already found")]
    AlreadyFound,
    #[fail(display = "Not found")]
    NotFound,
    #[fail(display = "Invalid stage")]
    InvalidStage,
    #[fail(display = "Invalid account")]
    InvalidAccount,
    #[fail(display = "Invalid node")]
    InvalidNode,
    #[fail(display = "Invalid transaction")]
    InvalidTransaction,
    #[fail(display = "Invalid address")]
    InvalidAddress,
    #[fail(display = "Already mined")]
    AlreadyMined,
    #[fail(display = "Not mined")]
    NotMined,
    #[fail(display = "Invalid message")]
    InvalidMessage,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        let msg = format!("{}", error);
        Error::IO { msg }
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

impl From<NetworkError> for Error {
    fn from(error: NetworkError) -> Error {
        let msg = format!("{}", error);
        Error::Network { msg }
    }
}

impl From<ConfigError> for Error {
    fn from(error: ConfigError) -> Error {
        let msg = format!("{}", error);
        Error::Config { msg }
    }
}
