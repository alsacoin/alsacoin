//! # Transaction
//!
//! `transaction` contains the `Transaction` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::version::Version;
use crypto::ecc::ed25519::Signature;
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Input {
    address: Address,
    value: u64,
    signature: Signature,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Output {
    address: Address,
    value: u64,
}

/// `Transaction` is the Alsacoin transaction type. It is built
/// around the HybridTx model defined in `Chimeric Ledgers` papers.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Digest,
    pub version: Version,
    pub stage: Stage,
    pub time: Timestamp,
    pub locktime: Timestamp,
    pub inputs: BTreeMap<Address, Input>,
    pub outputs: BTreeMap<Address, Output>,
    pub fee: u64,
    pub nonce: u64,
}

impl Transaction {
    /// `new` creates a new `Transaction`.
    pub fn new() -> Transaction {
        Transaction::default()
    }

    /// `random` creates a new random `Transaction`.
    pub fn random() -> Result<Transaction> {
        Ok(Transaction::new()) // TODO
    }

    /// `set_locktime` sets the `Transaction` locktime.
    pub fn set_locktime(&mut self, locktime: Timestamp) -> Result<()> {
        unreachable!()
    }

    /// `clear_locktime` clears the `Transaction` locktime.
    pub fn clear_locktime(&mut self) -> Result<()> {
        unreachable!()
    }

    /// `input_balance` returns the `Transaction` inputs balance.
    pub fn input_balance(&self) -> u64 {
        unreachable!()
    }

    /// `output_balance` returns the `Transaction` outputs balance.
    pub fn output_balance(&self) -> u64 {
        unreachable!()
    }

    /// `balance` returns the `Transaction` balance.
    pub fn balance(&self) -> i64 {
        unreachable!()
    }

    /// `max_fee` returns the maximum fee available for the `Transaction`.
    pub fn max_fee(&self) -> u64 {
        unreachable!()
    }

    /// `add_input` adds an `Input` in the Transaction.
    pub fn add_input(address: Address, value: u64) -> Result<()> {
        unreachable!()
    }

    /// `del_input` deletes an `Input` from the `Transaction`.
    pub fn del_input(address: Address) -> Result<()> {
        unreachable!()
    }

    /// `add_output` adds an `Output` in the Transaction.
    pub fn add_output(address: Address, value: u64) -> Result<()> {
        unreachable!()
    }

    /// `del_output` deletes an `Output` from the `Transaction`.
    pub fn del_output(address: Address) -> Result<()> {
        unreachable!()
    }

    /// `set_fee` sets the fee in the `Transaction`.
    pub fn set_fee(&self, fee: u64) -> Result<()> {
        unreachable!()
    }

    /// `validate` validates the `Transaction`.
    pub fn validate(&self) -> Result<()> {
        Ok(()) // TODO
    }

    /// `to_bytes` converts the `Transaction` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Transaction`.
    pub fn from_bytes(b: &[u8]) -> Result<Transaction> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Transaction` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Transaction`.
    pub fn from_json(s: &str) -> Result<Transaction> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_transaction_serialize_bytes() {
    for _ in 0..10 {
        let transaction_a = Transaction::random().unwrap();

        let res = transaction_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Transaction::from_bytes(&cbor);
        assert!(res.is_ok());
        let transaction_b = res.unwrap();

        assert_eq!(transaction_a, transaction_b)
    }
}

#[test]
fn test_transaction_serialize_json() {
    for _ in 0..10 {
        let transaction_a = Transaction::random().unwrap();

        let res = transaction_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Transaction::from_json(&json);
        assert!(res.is_ok());
        let transaction_b = res.unwrap();

        assert_eq!(transaction_a, transaction_b)
    }
}
