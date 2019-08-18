//! # Transaction
//!
//! `transaction` contains the `Transaction` type and functions.

use crate::address::Address;
use crate::coinbase::Coinbase;
use crate::error::Error;
use crate::input::Input;
use crate::output::Output;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crate::version::Version;
use crypto::ecc::ed25519::{PublicKey, SecretKey};
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use store::traits::Store;

/// `Transaction` is the Alsacoin transaction type. It is built
/// around the HybridTx model defined in `Chimeric Ledgers` papers.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Digest,
    pub version: Version,
    pub stage: Stage,
    pub time: Timestamp,
    pub locktime: Timestamp,
    pub distance: u64,
    pub inputs: BTreeMap<Address, Input>,
    pub outputs: BTreeMap<Address, Output>,
    pub coinbase: Option<Coinbase>,
    pub fee: u64,
    pub nonce: u64,
}

impl Transaction {
    /// `new` creates a new `Transaction`.
    pub fn new() -> Result<Transaction> {
        let mut transaction = Transaction {
            id: Digest::default(),
            version: Version::default(),
            stage: Stage::default(),
            time: Timestamp::default(),
            locktime: Timestamp::default(),
            distance: 1,
            inputs: BTreeMap::default(),
            outputs: BTreeMap::default(),
            coinbase: None,
            fee: 0,
            nonce: Random::u64()?,
        };

        transaction.update_id()?;

        Ok(transaction)
    }

    /// `set_time` sets the `Transaction` time.
    pub fn set_time(&mut self, time: Timestamp) -> Result<()> {
        time.validate()?;

        self.time = time;

        self.update_id()
    }

    /// `set_locktime` sets the `Transaction` locktime.
    pub fn set_locktime(&mut self, locktime: Timestamp) -> Result<()> {
        locktime.validate()?;

        if locktime < self.time {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        self.locktime = locktime;

        self.update_id()
    }

    /// `clear_locktime` clears the `Transaction` locktime.
    pub fn clear_locktime(&mut self) -> Result<()> {
        self.locktime = self.time;

        self.update_id()
    }

    /// `input_balance` returns the `Transaction` inputs balance.
    pub fn input_balance(&self) -> u64 {
        self.inputs
            .iter()
            .fold(0, |acc, (_, input)| acc + input.amount)
    }

    /// `output_balance` returns the `Transaction` outputs balance.
    pub fn output_balance(&self) -> u64 {
        self.outputs
            .iter()
            .fold(0, |acc, (_, output)| acc + output.amount)
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
        input.validate()?;

        if self.lookup_input(&input.address) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self.inputs.insert(input.address, input.clone());

        if input.distance > self.distance {
            self.distance = input.distance;
        }

        self.update_id()
    }

    /// `update_input` updates an `Input` in the `Transaction`.
    pub fn update_input(&mut self, input: &Input) -> Result<()> {
        input.validate()?;

        if !self.lookup_input(&input.address) {
            let err = Error::NotFound;
            return Err(err);
        }

        if input == &self.get_input(&input.address)? {
            return Ok(());
        }

        self.inputs.insert(input.address, input.clone());

        if input.distance > self.distance {
            self.distance = input.distance;
        }

        self.update_id()
    }

    /// `delete_input` deletes an `Input` from the `Transaction`.
    pub fn delete_input(&mut self, address: &Address) -> Result<()> {
        if !self.lookup_input(address) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.inputs.remove(address);

        self.update_distance()?;

        self.update_id()
    }

    /// `validate_input` validates an `Input` in the `Transaction`.
    pub fn validate_input(&self, address: &Address) -> Result<()> {
        let input = self.get_input(address)?;
        input.validate()?;

        if &input.address != address {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        if input.distance > self.distance {
            let err = Error::InvalidDistance;
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

    /// `validate_outputs` validates all the `Output`s in the `Transaction`.
    pub fn validate_outputs(&self) -> Result<()> {
        for (address, output) in self.clone().outputs {
            if address != output.address {
                let err = Error::InvalidAddress;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `input_sign_message` returns the binary message to use when signing an `Input` in the
    /// `Transaction`.
    pub fn input_sign_message(&self) -> Result<Vec<u8>> {
        let mut clone = self.clone();

        clone.id = Digest::default();

        for input in clone.clone().inputs.values_mut() {
            input.signatures = BTreeMap::default();
            clone.update_input(&input)?;
        }

        clone.id = Digest::default();

        clone.to_bytes()
    }

    /// `sign_input` signs an `Input` in the `Transaction`.
    pub fn sign_input(&mut self, secret_key: &SecretKey, address: &Address) -> Result<()> {
        let mut input = self.get_input(&address)?;

        let msg = self.input_sign_message()?;
        input.sign(secret_key, &msg)?;
        input.validate()?;

        self.update_input(&input)
    }

    /// `verify_input_signature` verifies an `Input` signature.
    pub fn verify_input_signature(&self, public_key: &PublicKey, address: &Address) -> Result<()> {
        let input = self.get_input(address)?;
        let msg = self.input_sign_message()?;
        input.verify_signature(public_key, &msg)
    }

    /// `set_coinbase` sets the `Transaction` `Coinbase`.
    pub fn set_coinbase(&mut self, address: &Address, difficulty: u64) -> Result<()> {
        if difficulty == 0 {
            let err = Error::InvalidDifficulty;
            return Err(err);
        }

        let coinbase = Coinbase::new(address, self.distance, difficulty)?;
        self.coinbase = Some(coinbase);

        Ok(())
    }

    /// `mining_message` returns the binary message used in mining.
    pub fn mining_message(&self) -> Result<Vec<u8>> {
        if self.coinbase.is_none() {
            let err = Error::InvalidCoinbase;
            return Err(err);
        }

        let mut clone = self.clone();
        clone.id = Digest::default();

        if let Some(mut coinbase) = clone.coinbase {
            coinbase.validate()?;
            coinbase.clear();
            clone.coinbase = Some(coinbase);
        }

        clone.to_bytes()
    }

    /// `mine` mines the `Transaction` `Coinbase`.
    pub fn mine(&mut self) -> Result<()> {
        if self.coinbase.is_none() {
            let err = Error::InvalidCoinbase;
            return Err(err);
        }

        if let Some(mut coinbase) = self.coinbase {
            let msg = self.mining_message()?;
            coinbase.mine(&msg)?;
            self.coinbase = Some(coinbase);
        }

        Ok(())
    }

    /// `verify_mining_proof` verifies the `Transaction` mined `Coinbase` proof.
    pub fn validate_mining_proof(&self) -> Result<()> {
        if self.coinbase.is_none() {
            let err = Error::InvalidCoinbase;
            return Err(err);
        }

        if let Some(coinbase) = self.coinbase {
            let msg = self.mining_message()?;
            coinbase.validate_mining_proof(&msg)?;
        }

        Ok(())
    }

    /// `validate_coinbase` validates the `Transaction` `Coinbase`.
    pub fn validate_coinbase(&self) -> Result<()> {
        if let Some(coinbase) = self.coinbase {
            if coinbase.distance != self.distance {
                let err = Error::InvalidCoinbase;
                return Err(err);
            }

            coinbase.validate()?;
        }

        Ok(())
    }

    /// `set_fee` sets the fee in the `Transaction`.
    pub fn set_fee(&mut self, fee: u64) -> Result<()> {
        if fee > self.max_fee() {
            let err = Error::InvalidFee;
            return Err(err);
        }

        self.fee = fee;

        self.update_id()
    }

    /// `update_distance` updates the `Transaction` distance.
    pub fn update_distance(&mut self) -> Result<()> {
        let mut distance = self.distance;

        if distance == 0 {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        for input in self.inputs.values() {
            if input.distance > distance {
                distance = input.distance;
            }
        }

        self.distance = distance;

        Ok(())
    }

    /// `update_nonce` updates the `Transaction` nonce.
    pub fn update_nonce(&mut self) -> Result<()> {
        let mut new_nonce = Random::u64()?;

        while self.nonce != new_nonce {
            new_nonce = Random::u64()?;
        }

        self.nonce = new_nonce;

        self.update_id()
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

        self.outputs.insert(output.address, output.clone());

        self.update_id()
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

        self.outputs.insert(output.address, output.clone());

        self.update_id()
    }

    /// `delete_output` deletes an `Output` from the `Transaction`.
    pub fn delete_output(&mut self, address: &Address) -> Result<()> {
        if !self.lookup_output(address) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.outputs.remove(address);

        self.update_id()
    }

    /// `calc_id` calculates the `Transaction` id.
    pub fn calc_id(&self) -> Result<Digest> {
        let mut clone = self.clone();
        clone.id = Digest::default();

        let buf = clone.to_bytes()?;
        let id = Blake512Hasher::hash(&buf);
        Ok(id)
    }

    /// `update_id` updates the `Transaction` id.
    pub fn update_id(&mut self) -> Result<()> {
        let id = self.calc_id()?;
        if self.id != id {
            self.id = id;
        }

        Ok(())
    }

    /// `validate_id` validates the `Transaction` id.
    pub fn validate_id(&self) -> Result<()> {
        if self.id != self.calc_id()? {
            let err = Error::InvalidId;
            return Err(err);
        }

        Ok(())
    }

    /// `validate_distance` validates the `Transaction` distance.
    pub fn validate_distance(&self) -> Result<()> {
        if self.distance == 0 {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        let max_distance = self.distance;

        for input in self.inputs.values() {
            if input.distance > max_distance {
                let err = Error::InvalidDistance;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `validate_times` validates the `Transaction` time and locktime.
    pub fn validate_times(&self) -> Result<()> {
        self.time.validate()?;

        self.locktime.validate()?;

        if self.time > self.locktime {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        Ok(())
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
        self.validate_id()?;

        self.version.validate()?;

        self.validate_times()?;

        self.validate_inputs()?;

        self.validate_distance()?;

        self.validate_balance()?;

        self.validate_coinbase()?;

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

impl Default for Transaction {
    fn default() -> Transaction {
        Transaction::new().unwrap()
    }
}

impl<S: Store> Storable<S> for Transaction {
    const KEY_PREFIX: u32 = 3;

    type Key = Digest;

    fn lookup(&self, _store: &S, _key: &Self::Key) -> Result<bool> {
        // TODO
        unreachable!()
    }

    fn get(&self, _store: &S, _key: &Self::Key) -> Result<Self> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _store: &S,
        _from: Option<&Self::Key>,
        _to: Option<&Self::Key>,
        _count: Option<u32>,
        _skip: Option<u32>,
    ) -> Result<Vec<Self>> {
        // TODO
        unreachable!()
    }

    fn count(
        &self,
        _store: &S,
        _from: Option<&Self::Key>,
        _to: Option<&Self::Key>,
        _skip: Option<u32>,
    ) -> Result<u32> {
        // TODO
        unreachable!()
    }

    fn insert(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn create(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn update(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn insert_batch(&mut self, _store: &mut S, _items: &[(Self::Key, Self)]) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _store: &mut S, _key: &Self::Key) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _store: &mut S, _keys: &[Self::Key]) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn cleanup(&mut self, _store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn clear(&mut self, _store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
    }
}

#[test]
fn test_transaction_new() {
    let res = Transaction::new();
    assert!(res.is_ok());

    let transaction = res.unwrap();
    let res = transaction.validate();
    assert!(res.is_ok())
}

#[test]
fn test_transaction_id() {
    let mut transaction = Transaction::new().unwrap();

    let res = transaction.validate_id();
    assert!(res.is_ok());

    let mut invalid_id = Digest::random().unwrap();
    while invalid_id == transaction.id {
        invalid_id = Digest::random().unwrap();
    }

    transaction.id = invalid_id;

    let res = transaction.validate_id();
    assert!(res.is_err());
}

#[test]
fn test_transaction_times() {
    let mut transaction = Transaction::new().unwrap();

    let res = transaction.validate_times();
    assert!(res.is_ok());

    let invalid_date = "2012-12-12T00:00:00Z";
    let invalid_timestamp = Timestamp::parse(invalid_date).unwrap();
    let res = transaction.set_time(invalid_timestamp);
    assert!(res.is_err());
    let res = transaction.set_locktime(invalid_timestamp);
    assert!(res.is_err());

    transaction.time = Timestamp::now();
    let res = transaction.validate_times();
    assert!(res.is_ok());

    let invalid_locktime_i64 = transaction.time.to_i64() - 1_000;
    let invalid_locktime = Timestamp::from_i64(invalid_locktime_i64).unwrap();
    transaction.locktime = invalid_locktime;
    let res = transaction.validate_times();
    assert!(res.is_err());

    transaction.clear_locktime().unwrap();
    assert_eq!(transaction.time, transaction.locktime);
    let res = transaction.validate_times();
    assert!(res.is_ok());
}

#[test]
fn test_transaction_inputs() {
    use crate::account::Account;
    use crate::signer::Signer;
    use crate::signers::Signers;
    use crypto::random::Random;

    let mut transaction = Transaction::new().unwrap();

    for _ in 0..10 {
        let secret_key = SecretKey::random().unwrap();
        let public_key = secret_key.to_public();

        let threshold = 10;
        let weight = threshold;

        let signer = Signer { public_key, weight };
        let mut signers = Signers::new().unwrap();
        signers.add(&signer).unwrap();
        signers.set_threshold(threshold).unwrap();

        let value = Random::u64().unwrap();
        let account = Account::new(&signers, value).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let amount = Random::u64().unwrap();
        let mut input = Input::new(&account, distance, amount).unwrap();

        let found = transaction.lookup_input(&input.address);
        assert!(!found);

        let res = transaction.get_input(&input.address);
        assert!(res.is_err());

        let res = transaction.add_input(&input);
        assert!(res.is_ok());

        let found = transaction.lookup_input(&input.address);
        assert!(found);

        let res = transaction.get_input(&input.address);
        assert!(res.is_ok());

        let entry = res.unwrap();
        assert_eq!(entry, input);

        input.amount = 10;

        let res = transaction.update_input(&input);
        assert!(res.is_ok());

        let entry = transaction.get_input(&input.address).unwrap();
        assert_eq!(entry, input);

        let res = entry.fully_signed();
        assert!(res.is_ok());
        assert!(!res.unwrap());

        let res = transaction.sign_input(&secret_key, &input.address);
        assert!(res.is_ok());

        let entry = transaction.get_input(&input.address).unwrap();
        let res = entry.fully_signed();
        assert!(res.is_ok());
        assert!(res.unwrap());

        let msg = transaction.input_sign_message().unwrap();

        let res = entry.verify_signature(&public_key, &msg);
        assert!(res.is_ok());

        let res = transaction.verify_input_signature(&public_key, &input.address);
        assert!(res.is_ok());

        let res = transaction.validate_input(&input.address);
        assert!(res.is_ok());

        let res = transaction.validate_inputs();
        assert!(res.is_ok());

        let res = transaction.delete_input(&input.address);
        assert!(res.is_ok());

        let found = transaction.lookup_input(&input.address);
        assert!(!found);

        let res = transaction.get_input(&input.address);
        assert!(res.is_err());

        let res = transaction.validate_inputs();
        assert!(res.is_ok());
    }
}

#[test]
fn test_transaction_outputs() {
    let mut transaction = Transaction::new().unwrap();

    for _ in 0..10 {
        let mut output = Output::random().unwrap();

        let found = transaction.lookup_output(&output.address);
        assert!(!found);

        let res = transaction.get_output(&output.address);
        assert!(res.is_err());

        let res = transaction.add_output(&output);
        assert!(res.is_ok());

        let res = transaction.validate_outputs();
        assert!(res.is_ok());

        let found = transaction.lookup_output(&output.address);
        assert!(found);

        let res = transaction.get_output(&output.address);
        assert!(res.is_ok());

        let entry = res.unwrap();
        assert_eq!(entry, output);

        output.amount = 10;

        let res = transaction.update_output(&output);
        assert!(res.is_ok());

        let res = transaction.validate_outputs();
        assert!(res.is_ok());

        let entry = transaction.get_output(&output.address).unwrap();
        assert_eq!(entry, output);

        let res = transaction.delete_output(&output.address);
        assert!(res.is_ok());

        let found = transaction.lookup_output(&output.address);
        assert!(!found);

        let res = transaction.get_output(&output.address);
        assert!(res.is_err());
    }

    let res = transaction.validate_outputs();
    assert!(res.is_ok());
}

#[test]
fn test_transaction_distance() {
    use crate::account::Account;
    use crate::signer::Signer;
    use crate::signers::Signers;
    use crypto::random::Random;

    let mut transaction = Transaction::new().unwrap();
    let mut max_distance = transaction.distance;

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64().unwrap();
        let threshold = weight;

        let signer = Signer { public_key, weight };
        let mut signers = Signers::new().unwrap();
        signers.add(&signer).unwrap();
        signers.set_threshold(threshold).unwrap();

        let value = Random::u64().unwrap();
        let account = Account::new(&signers, value).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let amount = Random::u64().unwrap();
        let input = Input::new(&account, distance, amount).unwrap();

        transaction.add_input(&input).unwrap();

        if input.distance > max_distance {
            max_distance = input.distance;
        }

        assert_eq!(transaction.distance, max_distance);

        let res = transaction.validate_distance();
        assert!(res.is_ok());
    }

    transaction.distance -= 1;
    let res = transaction.validate_distance();
    assert!(res.is_err());
}

#[test]
fn test_transaction_balance() {
    use crate::account::Account;
    use crate::signer::Signer;
    use crate::signers::Signers;
    use crypto::random::Random;

    let mut transaction = Transaction::new().unwrap();
    let mut input_balance = 0;
    let mut output_balance = 0;
    let mut fee = 0;
    let mut expected_balance = 0i64;

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64().unwrap();
        let threshold = weight;

        let signer = Signer { public_key, weight };
        let mut signers = Signers::new().unwrap();
        signers.add(&signer).unwrap();
        signers.set_threshold(threshold).unwrap();

        let value = Random::u64().unwrap();
        let account = Account::new(&signers, value).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let amount = 10;
        let input = Input::new(&account, distance, amount).unwrap();

        transaction.add_input(&input).unwrap();
        input_balance += input.amount;
        expected_balance += input.amount as i64;

        let max_fee = transaction.max_fee();
        assert_eq!(max_fee, input_balance);

        let balance = transaction.balance();
        assert_eq!(balance, expected_balance);
        assert_eq!(balance, input_balance as i64);

        let res = transaction.validate_balance();
        assert!(res.is_err());

        transaction.delete_input(&input.address).unwrap();
        input_balance -= input.amount;
        expected_balance -= input.amount as i64;

        let max_fee = transaction.max_fee();
        assert_eq!(max_fee, 0);

        let balance = transaction.balance();
        assert_eq!(balance, expected_balance);
        assert_eq!(balance, input_balance as i64);

        let res = transaction.validate_balance();
        assert!(res.is_ok());
    }

    assert_eq!(expected_balance, 0);

    for _ in 0..10 {
        let mut output = Output::random().unwrap();
        output.amount = 10;

        transaction.add_output(&output).unwrap();
        output_balance += output.amount;
        expected_balance -= output.amount as i64;

        let max_fee = transaction.max_fee();
        assert_eq!(max_fee, 0);

        let balance = transaction.balance();
        assert_eq!(balance, expected_balance);
        assert_eq!(balance, -(output_balance as i64));

        let res = transaction.validate_balance();
        assert!(res.is_err());

        transaction.delete_output(&output.address).unwrap();
        output_balance -= output.amount;
        expected_balance += output.amount as i64;

        let max_fee = transaction.max_fee();
        assert_eq!(max_fee, 0);

        let balance = transaction.balance();
        assert_eq!(expected_balance, 0);
        assert_eq!(balance, expected_balance);

        let res = transaction.validate_balance();
        assert!(res.is_ok());
    }

    let res = transaction.validate_balance();
    assert!(res.is_ok());

    fee += 10;
    transaction.fee = fee;
    expected_balance -= fee as i64;

    let max_fee = transaction.max_fee();
    assert_eq!(max_fee, 0);

    let balance = transaction.balance();
    assert_eq!(balance, expected_balance);

    let res = transaction.validate_balance();
    assert!(res.is_err());

    let res = transaction.set_fee(fee);
    assert!(res.is_err());

    transaction.fee = 0;
    let res = transaction.validate_balance();
    assert!(res.is_ok());
}

#[test]
fn test_transaction_coinbase() {
    use crypto::random::Random;

    let invalid_difficulty = 0;

    for _ in 0..10 {
        let mut transaction = Transaction::default();
        let address = Address::random().unwrap();
        let valid_difficulty = Random::u64_range(1, 10).unwrap();

        let res = transaction.validate_coinbase();
        assert!(res.is_ok());

        let res = transaction.set_coinbase(&address, invalid_difficulty);
        assert!(res.is_err());

        let res = transaction.set_coinbase(&address, valid_difficulty);
        assert!(res.is_ok());

        let res = transaction.validate_coinbase();
        assert!(res.is_ok());
    }
}

#[test]
fn test_transaction_mine() {
    use crypto::random::Random;

    for _ in 0..10 {
        let mut transaction = Transaction::default();
        let address = Address::random().unwrap();
        let difficulty = Random::u64_range(1, 10).unwrap();

        transaction.set_coinbase(&address, difficulty).unwrap();

        let res = transaction.mine();
        assert!(res.is_ok());

        let res = transaction.validate_coinbase();
        assert!(res.is_ok());

        let res = transaction.validate_mining_proof();
        assert!(res.is_ok());

        let mut coinbase = transaction.coinbase.unwrap();

        if coinbase.nonce == u64::max_value() {
            coinbase.nonce = 0;
        } else {
            coinbase.nonce += 1;
        }

        transaction.coinbase = Some(coinbase);

        let res = transaction.validate_mining_proof();
        assert!(res.is_err());
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
