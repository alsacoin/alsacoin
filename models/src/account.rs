//! # Account
//!
//! `account` contains the `Account` type and functions.

use crate::address::Address;
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};

/// `Account` is the type used to represent an Alsacoin account
/// of a user, account which is identified by an ID and
/// an `Address`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Account {
    pub id: Digest,
    pub prev_id: Digest,
    pub counter: u64,
    pub address: Address,
    pub value: u64, // NB: gonna be confidential
}
