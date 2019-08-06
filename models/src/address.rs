//! # Address
//!
//! `address` contains the `Address` type and functions.

use crypto::hash::Digest;

/// `Address` is the address of an Alsacoin `Account`.
/// It's an alias of a `Blake512` `Digest`.
pub type Address = Digest;
