//! # Transaction
//!
//! `transaction` contains the `Transaction` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::input::Input;
use crate::output::Output;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::version::Version;
use crypto::ecc::ed25519::SecretKey;
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub fn new() -> Result<Transaction> {
        let mut transaction = Transaction::default();
        transaction.nonce = Random::u64()?;

        transaction.id = transaction.calc_id()?;

        Ok(transaction)
    }

    /// `set_locktime` sets the `Transaction` locktime.
    pub fn set_locktime(&mut self, locktime: Timestamp) -> Result<()> {
        if locktime < self.time {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        self.locktime = locktime;

        Ok(())
    }

    /// `clear_locktime` clears the `Transaction` locktime.
    pub fn clear_locktime(&mut self) {
        self.locktime = self.time;
    }

    /// `input_balance` returns the `Transaction` inputs balance.
    pub fn input_balance(&self) -> u64 {
        self.inputs
            .iter()
            .fold(0, |acc, (_, input)| acc + input.value)
    }

    /// `output_balance` returns the `Transaction` outputs balance.
    pub fn output_balance(&self) -> u64 {
        self.outputs
            .iter()
            .fold(0, |acc, (_, output)| acc + output.value)
    }

    /// `balance` returns the `Transaction` balance.
    pub fn balance(&self) -> i64 {
        let ibalance = self.input_balance() as i64;
        let obalance = self.output_balance() as i64;
        let fee = self.fee as i64;

        ibalance - obalance - fee
    }

    /// `max_fee` returns the maximum fee available for the `Transaction`.
    pub fn max_fee(&self) -> u64 {
        let imbalance = self.balance() - (self.fee as i64);

        if imbalance <= 0 {
            0
        } else {
            imbalance as u64
        }
    }

    /// `lookup_input` look ups an `Input` in the `Transaction`.
    pub fn lookup_input(&self, address: &Address) -> bool {
        self.inputs.contains_key(address)
    }

    /// `get_input` returns an `Input` from the `Transaction`.
    pub fn get_input(&self, address: &Address) -> Result<Input> {
        if !self.lookup_input(address) {
            let err = Error::NotFound;
            return Err(err);
        }

        let input = self.inputs.get(address).unwrap().clone();
        Ok(input)
    }

    /// `add_input` adds an `Input` in the Transaction.
    pub fn add_input(&mut self, input: &Input) -> Result<()> {
        if self.lookup_input(&input.address) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        if self.inputs.insert(input.address, input.clone()).is_none() {
            let err = Error::NoResult;
            return Err(err);
        }

        Ok(())
    }

    /// `update_input` updates an `Input` in the `Transaction`.
    pub fn update_input(&mut self, input: &Input) -> Result<()> {
        if !self.lookup_input(&input.address) {
            let err = Error::NotFound;
            return Err(err);
        }

        if input == &self.get_input(&input.address)? {
            return Ok(());
        }

        if self.inputs.insert(input.address, input.clone()).is_none() {
            let err = Error::NoResult;
            return Err(err);
        }

        Ok(())
    }

    /// `del_input` deletes an `Input` from the `Transaction`.
    pub fn del_input(&mut self, input: &Input) -> Result<()> {
        if !self.lookup_input(&input.address) {
            let err = Error::NotFound;
            return Err(err);
        }

        if self.inputs.remove(&input.address).is_none() {
            let err = Error::NoResult;
            return Err(err);
        }

        Ok(())
    }

    /// `validate_input` validates an `Input` in the `Transaction`.
    pub fn validate_input(&self, address: &Address) -> Result<()> {
        let input = self.get_input(address)?;
        input.validate()?;

        if &input.address != address {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        Ok(())
    }

    /// `validate_inputs` validates all the `Input`s in the `Transaction`.
    pub fn validate_inputs(&self) -> Result<()> {
        for address in self.clone().inputs.keys() {
            self.validate_input(address)?;
        }

        Ok(())
    }

    /// `input_sign_message` returns the binary message to use when signing an `Input` in the
    /// `Transaction`.
    pub fn input_sign_message(&self) -> Result<Vec<u8>> {
        let mut clone = self.clone();

        for input in clone.clone().inputs.values_mut() {
            if input.signature.is_some() {
                input.signature = None;
                clone.update_input(&input)?;
            }
        }

        clone.to_bytes()
    }

    /// `sign_input` signs an `Input` in the `Transaction`.
    pub fn sign_input(&mut self, secret_key: &SecretKey) -> Result<()> {
        let address = secret_key.to_public();
        let mut input = self.get_input(&address)?;

        let msg = self.input_sign_message()?;
        input.sign(secret_key, &msg)?;

        self.update_input(&input)
    }

    /// `validate_input_signature` validates an `Input` signature.
    pub fn validate_input_signature(&self, secret_key: &SecretKey) -> Result<()> {
        let address = secret_key.to_public();
        let input = self.get_input(&address)?;
        if input.signature.is_none() {
            let err = Error::InvalidSignature;
            return Err(err);
        }

        let msg = self.input_sign_message()?;
        if input.signature.unwrap() != input.calc_signature(secret_key, &msg)? {
            let err = Error::InvalidSignature;
            return Err(err);
        }

        Ok(())
    }

    /// `verify_input_signature` verifies an `Input` signature.
    pub fn verify_input_signature(&self, address: &Address) -> Result<()> {
        let input = self.get_input(address)?;
        let msg = self.input_sign_message()?;
        input.verify_signature(address, &msg)
    }

    /// `set_fee` sets the fee in the `Transaction`.
    pub fn set_fee(&mut self, fee: u64) -> Result<()> {
        if fee > self.max_fee() {
            let err = Error::InvalidFee;
            return Err(err);
        }

        self.fee = fee;

        Ok(())
    }

    /// `lookup_output` look ups an `Output` in the `Transaction`.
    pub fn lookup_output(&self, address: &Address) -> bool {
        self.outputs.contains_key(address)
    }

    /// `get_output` returns an `Output` from the `Transaction`.
    pub fn get_output(&self, address: &Address) -> Result<Output> {
        if !self.lookup_output(address) {
            let err = Error::NotFound;
            return Err(err);
        }

        let output = self.outputs.get(address).unwrap().clone();
        Ok(output)
    }

    /// `add_output` adds an `Output` in the Transaction.
    pub fn add_output(&mut self, output: &Output) -> Result<()> {
        if self.lookup_output(&output.address) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        if self
            .outputs
            .insert(output.address, output.clone())
            .is_none()
        {
            let err = Error::NoResult;
            return Err(err);
        }

        Ok(())
    }

    /// `update_output` updates an `Output` in the `Transaction`.
    pub fn update_output(&mut self, output: &Output) -> Result<()> {
        if !self.lookup_output(&output.address) {
            let err = Error::NotFound;
            return Err(err);
        }

        if output == &self.get_output(&output.address)? {
            return Ok(());
        }

        if self
            .outputs
            .insert(output.address, output.clone())
            .is_none()
        {
            let err = Error::NoResult;
            return Err(err);
        }

        Ok(())
    }

    /// `del_output` deletes an `Output` from the `Transaction`.
    pub fn del_output(&mut self, output: &Output) -> Result<()> {
        if !self.lookup_output(&output.address) {
            let err = Error::NotFound;
            return Err(err);
        }

        if self.outputs.remove(&output.address).is_none() {
            let err = Error::NoResult;
            return Err(err);
        }

        Ok(())
    }

    /// `calc_id` calculates the `Transaction` id.
    pub fn calc_id(&self) -> Result<Digest> {
        let mut clone = self.clone();
        clone.id = Digest::default();

        let buf = clone.to_bytes()?;
        let id = Blake512Hasher::hash(&buf);
        Ok(id)
    }

    /// `validate_balance` validates the `Transaction` balance.
    pub fn validate_balance(&self) -> Result<()> {
        // TODO: check that balance == coinbase_amount (if any)
        if self.balance() != 0 {
            let err = Error::InvalidBalance;
            return Err(err);
        }

        Ok(())
    }

    /// `validate` validates the `Transaction`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id()? {
            let err = Error::InvalidId;
            return Err(err);
        }

        self.version.validate()?;

        self.time.validate()?;

        self.locktime.validate()?;

        if self.time > self.locktime {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        self.validate_inputs()?;

        self.validate_balance()?;

        Ok(())
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
        let transaction_a = Transaction::new().unwrap();

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
        let transaction_a = Transaction::new().unwrap();

        let res = transaction_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Transaction::from_json(&json);
        assert!(res.is_ok());
        let transaction_b = res.unwrap();

        assert_eq!(transaction_a, transaction_b)
    }
}
