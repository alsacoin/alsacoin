#[macro_use]
extern crate failure;

/// `error` contains the error type used in the crate.
pub mod error;

/// `result` contains the result type used in the crate.
pub mod result;

/// `utils` contains various utilities used in the crate.
pub mod utils;

/// `timestamp` contains the timestamping types and functions.
pub mod timestamp;

/// `stage` contains the stage type and functions.
pub mod stage;

/// `version` contains the version type and functions.
pub mod version;

/// `node` contains the node type and functions.
pub mod node;

/// `address` contains the address type and functions.
pub mod address;

/// `account` contains the account type and functions.
pub mod account;

/// `wallet` contains the wallet type and functions.
pub mod wallet;

/// `input` contains the input type and functions.
pub mod input;

/// `ouput` contains the output type and functions.
pub mod output;

/// `coinbase` contains the coinbase type and functions.
pub mod coinbase;

/// `transaction` contains the transaction type and functions.
pub mod transaction;
