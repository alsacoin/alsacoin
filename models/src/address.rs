//! # Address
//!
//! `address` contains the `Address` type and functions.

use crypto::ecc::ed25519::PublicKey;

/// `Address` is the address of an Alsacoin `Account`.
/// It's an alias of an Ed25519 `PublicKey`.
pub type Address = PublicKey;
