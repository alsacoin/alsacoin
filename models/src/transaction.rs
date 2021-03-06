//! # Transaction
//!
//! `transaction` contains the `Transaction` type and functions.

use crate::account::Account;
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
use std::collections::{BTreeMap, BTreeSet};
use store::traits::Store;

/// `Transaction` is the Alsacoin transaction type. It is built
/// around the HybridTx model defined in `Chimeric Ledgers` papers.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Digest,
    pub version: Version,
    pub stage: Stage,
    pub time: Timestamp,
    pub locktime: Option<Timestamp>,
    pub distance: u64,
    pub inputs: BTreeMap<Address, Input>,
    pub outputs: BTreeMap<Address, Output>,
    pub coinbase: Option<Coinbase>,
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
            locktime: None,
            distance: 1,
            inputs: BTreeMap::default(),
            outputs: BTreeMap::default(),
            coinbase: None,
            nonce: Random::u64()?,
        };

        transaction.update_id()?;

        Ok(transaction)
    }

    /// `new_eve` creates a new eve `Transaction`.
    pub fn new_eve(stage: Stage, address: &Address) -> Result<Transaction> {
        let coinbase = Coinbase::new_eve(address)?;

        let mut transaction = Transaction {
            id: Digest::default(),
            version: Version::default(),
            stage,
            time: Timestamp::default(),
            locktime: None,
            distance: 0,
            inputs: BTreeMap::default(),
            outputs: BTreeMap::default(),
            coinbase: Some(coinbase),
            nonce: Random::u64()?,
        };

        transaction.update_id()?;

        Ok(transaction)
    }

    /// `is_eve` returns if a `Transaction` is an eve `Transaction`.
    pub fn is_eve(&self) -> Result<bool> {
        self.validate_coinbase()?;

        if let Some(coinbase) = self.coinbase {
            let res = self.distance == 0
                && self.inputs.is_empty()
                && self.outputs.is_empty()
                && coinbase.is_eve()?;

            Ok(res)
        } else {
            Ok(false)
        }
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

        self.locktime = Some(locktime);

        self.update_id()
    }

    /// `input_balance` returns the `Transaction` inputs balance.
    pub fn input_balance(&self) -> Result<u64> {
        let mut res = 0;

        for input in self.inputs.values() {
            if let Some(amount) = input.amount.checked_add(res) {
                res += amount;
            } else {
                let err = Error::InvalidBalance;
                return Err(err);
            }
        }

        Ok(res)
    }

    /// `output_balance` returns the `Transaction` outputs balance.
    pub fn output_balance(&self) -> Result<u64> {
        let mut res = 0;

        for output in self.outputs.values() {
            if let Some(amount) = output.amount.checked_add(res) {
                res += amount;
            } else {
                let err = Error::InvalidBalance;
                return Err(err);
            }
        }

        Ok(res)
    }

    /// `coinbase_amount` returns the `Transaction` coinbase amount.
    pub fn coinbase_amount(&self) -> u64 {
        if let Some(ref coinbase) = self.coinbase {
            coinbase.amount
        } else {
            0
        }
    }

    /// `balance` returns the `Transaction` balance.
    pub fn balance(&self) -> Result<i64> {
        let ibalance = self.input_balance()? as i64;
        let obalance = self.output_balance()? as i64;
        let cbalance = self.coinbase_amount() as i64;

        let res = ibalance + cbalance - obalance;

        Ok(res)
    }

    /// `ancestors` returns the `Transaction` ancestors' ids.
    pub fn ancestors(&self) -> Result<BTreeSet<Digest>> {
        let mut ancestors = BTreeSet::new();

        if self.is_eve()? {
            return Ok(ancestors);
        }

        for input in self.inputs.values() {
            if let Some(tx_id) = input.account.transaction_id {
                ancestors.insert(tx_id);
            } else {
                let err = Error::InvalidInput;
                return Err(err);
            }
        }

        Ok(ancestors)
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
        let address = input.address();

        if self.lookup_input(&address) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self.inputs.insert(address, input.clone());

        if input.distance > self.distance {
            self.distance = input.distance;

            if let Some(mut coinbase) = self.coinbase {
                coinbase.distance = self.distance;
                coinbase.update_amount()?;
                self.coinbase = Some(coinbase);
            }
        }

        self.update_id()
    }

    /// `update_input` updates an `Input` in the `Transaction`.
    pub fn update_input(&mut self, input: &Input) -> Result<()> {
        input.validate()?;
        let address = input.address();

        if !self.lookup_input(&address) {
            let err = Error::NotFound;
            return Err(err);
        }

        if input == &self.get_input(&address)? {
            return Ok(());
        }

        self.inputs.insert(address, input.clone());

        if input.distance > self.distance {
            self.distance = input.distance;

            if let Some(mut coinbase) = self.coinbase {
                coinbase.distance = self.distance;
                coinbase.update_amount()?;
                self.coinbase = Some(coinbase);
            }
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

        if &input.address() != address {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        if input.distance > self.distance {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        if input.account.transaction_id == Some(self.id) {
            let err = Error::InvalidId;
            return Err(err);
        }

        if input.is_fully_signed()? {
            let msg = self.input_sign_message()?;
            input.verify_fully_signed(&msg)?;
        } else if input.is_signed() {
            let msg = self.input_sign_message()?;
            input.verify_signatures(&msg)?;
        }

        Ok(())
    }

    /// `validate_fully_signed_input` validates a signed `Input`.
    pub fn validate_fully_signed_input(&self, address: &Address) -> Result<()> {
        let input = self.get_input(address)?;
        input.validate()?;

        if &input.address() != address {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        if input.distance > self.distance {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        if input.account.transaction_id == Some(self.id) {
            let err = Error::InvalidId;
            return Err(err);
        }

        let msg = self.input_sign_message()?;
        input.verify_fully_signed(&msg)?;

        Ok(())
    }

    /// `validate_inputs` validates all the `Input`s in the `Transaction`.
    pub fn validate_inputs(&self) -> Result<()> {
        for address in self.inputs.keys() {
            self.validate_input(address)?;
        }

        Ok(())
    }

    /// `validate_fully_signed_inputs` validate all the `Input` expecting them to be fully
    /// signed.
    pub fn validate_fully_signed_inputs(&self) -> Result<()> {
        for address in self.inputs.keys() {
            self.validate_fully_signed_input(address)?;
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

    /// `verify_fully_signed_input` verifies a fully signed `Input`.
    pub fn verify_fully_signed_input(&self, address: &Address) -> Result<()> {
        let input = self.get_input(address)?;
        let msg = self.input_sign_message()?;
        input.verify_fully_signed(&msg)
    }

    /// `is_signed` returns if at least one `Input` has been signed.
    pub fn is_signed(&self) -> bool {
        self.inputs
            .values()
            .filter(|input| input.is_signed())
            .count()
            != 0
    }

    /// `is_fully_signed` returns if all the `Input`s are fully signed.
    pub fn is_fully_signed(&self) -> Result<bool> {
        for input in self.inputs.values() {
            if !input.is_fully_signed()? {
                return Ok(false);
            }
        }

        Ok(true)
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

    /// `is_mined` checks if the `Transaction` is mined.
    pub fn is_mined(&self) -> bool {
        if let Some(coinbase) = self.coinbase {
            coinbase.is_mined()
        } else {
            false
        }
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

    /// `validate_mined` verifies the `Transaction` mined `Coinbase` proof.
    pub fn validate_mined(&self) -> Result<()> {
        if self.coinbase.is_none() {
            let err = Error::InvalidCoinbase;
            return Err(err);
        }

        if let Some(coinbase) = self.coinbase {
            let msg = self.mining_message()?;
            coinbase.validate_mined(&msg)?;
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

    /// `update_distance` updates the `Transaction` distance.
    pub fn update_distance(&mut self) -> Result<()> {
        if self.is_eve()? {
            return Ok(());
        }

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

        if let Some(mut coinbase) = self.coinbase {
            coinbase.distance = distance;
            coinbase.update_amount()?;
            self.coinbase = Some(coinbase);
        }

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
        if self.is_eve()? {
            return Ok(());
        }

        if self.distance == 0 {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        if let Some(coinbase) = self.coinbase {
            if coinbase.distance != self.distance {
                let err = Error::InvalidDistance;
                return Err(err);
            }
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

        if let Some(locktime) = self.locktime {
            locktime.validate()?;

            if self.time > locktime {
                let err = Error::InvalidTimestamp;
                return Err(err);
            }
        }

        for input in self.inputs.values() {
            if let Some(locktime) = input.account.locktime {
                if self.time < locktime {
                    let err = Error::InvalidTimestamp;
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    /// `validate_balance` validates the `Transaction` balance.
    pub fn validate_balance(&self) -> Result<()> {
        if self.balance()? != self.coinbase_amount() as i64 {
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

    /// `validate_fully_signed` validates the `Transaction` expecting it to be fully
    /// signed.
    pub fn validate_fully_signed(&self) -> Result<()> {
        self.validate_id()?;

        self.version.validate()?;

        self.validate_times()?;

        self.validate_fully_signed_inputs()?;

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
    const KEY_PREFIX: u8 = 3;

    type Key = Digest;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key.to_bytes());
        Ok(buf)
    }

    fn validate_single(store: &S, stage: Stage, value: &Self) -> Result<()> {
        if value.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        value.validate()?;

        for input in value.inputs.values() {
            let account = input.account.clone();
            Account::validate_single(store, stage, &account)?;
        }

        Ok(())
    }

    fn validate_all(store: &S, stage: Stage) -> Result<()> {
        for value in Self::query(store, stage, None, None, None, None)? {
            Self::validate_single(store, stage, &value)?;
        }

        Ok(())
    }

    fn lookup(store: &S, stage: Stage, key: &Self::Key) -> Result<bool> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.lookup(&key).map_err(|e| e.into())
    }

    fn get(store: &S, stage: Stage, key: &Self::Key) -> Result<Self> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let buf = store.get(&key)?;
        Self::from_bytes(&buf)
    }

    fn query(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<BTreeSet<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _from = Digest::default();
            _from[0] = stage as u8;
            _from[1] = <Self as Storable<S>>::KEY_PREFIX;
            Some(_from.to_vec())
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _to = Digest::default();
            _to[0] = stage as u8;
            _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
            Some(_to.to_vec())
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.query(from, to, count, skip)?;
        let mut items = BTreeSet::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.insert(item);
        }

        Ok(items)
    }

    fn sample(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: u32,
    ) -> Result<BTreeSet<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _from = Digest::default();
            _from[0] = stage as u8;
            _from[1] = <Self as Storable<S>>::KEY_PREFIX;
            Some(_from.to_vec())
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _to = Digest::default();
            _to[0] = stage as u8;
            _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
            Some(_to.to_vec())
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.sample(from, to, count)?;
        let mut items = BTreeSet::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.insert(item);
        }

        Ok(items)
    }

    fn count(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _from = Digest::default();
            _from[0] = stage as u8;
            _from[1] = <Self as Storable<S>>::KEY_PREFIX;
            Some(_from.to_vec())
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _to = Digest::default();
            _to[0] = stage as u8;
            _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
            Some(_to.to_vec())
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let mut stored_accounts = BTreeSet::new();
        let mut clean_accounts = false;

        for input in value.inputs.values() {
            if !clean_accounts {
                let account = input.account.clone();

                if !Account::lookup(store, stage, &account.address())? {
                    let res = Account::insert(store, stage, &account);

                    if res.is_err() {
                        clean_accounts = true;
                    } else {
                        stored_accounts.insert(account);
                    }
                }
            } else {
                break;
            }
        }

        if clean_accounts {
            for account in stored_accounts {
                Account::remove(store, stage, &account.address())?;
            }
        }

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.insert(&store_key, &store_value).map_err(|e| e.into())
    }

    fn create(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let mut stored_accounts = BTreeSet::new();
        let mut clean_accounts = false;

        for input in value.inputs.values() {
            if !clean_accounts {
                let account = input.account.clone();

                if !Account::lookup(store, stage, &account.address())? {
                    let res = Account::insert(store, stage, &account);

                    if res.is_err() {
                        clean_accounts = true;
                    } else {
                        stored_accounts.insert(account);
                    }
                }
            } else {
                break;
            }
        }

        if clean_accounts {
            for account in stored_accounts {
                Account::remove(store, stage, &account.address())?;
            }
        }

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.create(&store_key, &store_value).map_err(|e| e.into())
    }

    fn update(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let mut stored_accounts = BTreeSet::new();
        let mut clean_accounts = false;

        for input in value.inputs.values() {
            if !clean_accounts {
                let account = input.account.clone();

                let res = <Account as Storable<S>>::update(store, stage, &account);

                if res.is_err() {
                    clean_accounts = true;
                } else {
                    stored_accounts.insert(account);
                }
            } else {
                break;
            }
        }

        if clean_accounts {
            for account in stored_accounts {
                Account::remove(store, stage, &account.address())?;
            }
        }

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.update(&store_key, &store_value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, stage: Stage, values: &BTreeSet<Self>) -> Result<()> {
        let mut items = BTreeSet::new();
        let mut accounts = BTreeSet::new();

        for value in values {
            Self::validate_single(store, stage, value)?;

            let key = <Self as Storable<S>>::key(value);
            let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
            let store_value = value.to_bytes()?;
            let item = (store_key, store_value);
            items.insert(item);

            for input in value.inputs.values() {
                let account = input.account.clone();
                accounts.insert(account);
            }
        }

        let res = Account::insert_batch(store, stage, &accounts);
        if res.is_err() {
            let addresses = accounts.iter().map(|account| account.address()).collect();
            let _ = Account::remove_batch(store, stage, &addresses);
        }

        let items: Vec<(&[u8], &[u8])> = items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, stage: Stage, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, stage: Stage, keys: &BTreeSet<Self::Key>) -> Result<()> {
        let mut _keys = BTreeSet::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            _keys.insert(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(store: &mut S, stage: Stage, min_time: Option<Timestamp>) -> Result<()> {
        let min_time = min_time.unwrap_or_default();

        let mut _from = Digest::default();
        _from[0] = stage as u8;
        _from[1] = <Self as Storable<S>>::KEY_PREFIX;
        let from = Some(_from.to_vec());
        let from = from.as_ref().map(|from| from.as_slice());

        let mut _to = Digest::default();
        _to[0] = stage as u8;
        _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
        let to = Some(_to.to_vec());
        let to = to.as_ref().map(|to| to.as_slice());

        for value in store.query(from, to, None, None)? {
            let tx = Transaction::from_bytes(&value)?;
            if tx.time < min_time {
                let key = <Self as Storable<S>>::key_to_bytes(stage, &tx.id)?;
                store.remove(&key)?;
            }
        }

        Ok(())
    }

    fn clear(store: &mut S, stage: Stage) -> Result<()> {
        let from = Some(vec![stage as u8, <Self as Storable<S>>::KEY_PREFIX]);
        let from = from.as_ref().map(|from| from.as_slice());

        let to = Some(vec![stage as u8, <Self as Storable<S>>::KEY_PREFIX + 1]);
        let to = to.as_ref().map(|to| to.as_slice());

        store.remove_range(from, to, None).map_err(|e| e.into())
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
fn test_transaction_eve() {
    let stage = Stage::random().unwrap();
    let address = Address::random().unwrap();

    let res = Transaction::new_eve(stage, &address);
    assert!(res.is_ok());

    let mut eve_transaction = res.unwrap();

    let res = eve_transaction.validate();
    assert!(res.is_ok());

    let res = eve_transaction.is_eve();
    assert!(res.is_ok());
    assert!(res.unwrap());

    eve_transaction.distance = 1;

    let res = eve_transaction.validate();
    assert!(res.is_err());

    let transaction = Transaction::new().unwrap();
    let res = transaction.is_eve();
    assert!(res.is_ok());
    assert!(!res.unwrap());
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

    transaction.locktime = Some(invalid_locktime);
    let res = transaction.validate_times();
    assert!(res.is_err());
}

#[test]
fn test_transaction_inputs() {
    use crate::account::Account;
    use crate::signer::Signer;
    use crate::signers::Signers;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();
    let mut transaction = Transaction::new().unwrap();
    transaction.stage = stage;
    transaction.update_id().unwrap();

    for _ in 0..10 {
        let secret_key = SecretKey::random().unwrap();
        let public_key = secret_key.to_public();

        let threshold = 10;
        let weight = threshold;

        let signer = Signer { public_key, weight };
        let mut signers = Signers::new().unwrap();
        signers.add(&signer).unwrap();
        signers.set_threshold(threshold).unwrap();

        let amount = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let mut input = Input::new(&account, distance, amount).unwrap();
        let address = input.address();

        let found = transaction.lookup_input(&address);
        assert!(!found);

        let res = transaction.get_input(&address);
        assert!(res.is_err());

        let res = transaction.add_input(&input);
        assert!(res.is_ok());

        let found = transaction.lookup_input(&address);
        assert!(found);

        let res = transaction.get_input(&address);
        assert!(res.is_ok());

        let entry = res.unwrap();
        assert_eq!(entry, input);

        input.amount = 10;

        let res = transaction.update_input(&input);
        assert!(res.is_ok());

        let entry = transaction.get_input(&address).unwrap();
        assert_eq!(entry, input);

        let res = entry.is_fully_signed();
        assert!(res.is_ok());
        assert!(!res.unwrap());

        let res = transaction.is_fully_signed();
        assert!(res.is_ok());
        assert!(!res.unwrap());

        let res = transaction.validate_fully_signed_input(&address);
        assert!(res.is_err());

        let res = transaction.validate_fully_signed_inputs();
        assert!(res.is_err());

        let res = transaction.sign_input(&secret_key, &address);
        assert!(res.is_ok());

        let entry = transaction.get_input(&address).unwrap();
        let res = entry.is_fully_signed();
        assert!(res.is_ok());
        assert!(res.unwrap());

        let msg = transaction.input_sign_message().unwrap();

        let res = entry.verify_signature(&public_key, &msg);
        assert!(res.is_ok());

        let res = transaction.verify_input_signature(&public_key, &address);
        assert!(res.is_ok());

        let res = transaction.is_fully_signed();
        assert!(res.is_ok());
        assert!(res.unwrap());

        let res = transaction.validate_input(&address);
        assert!(res.is_ok());

        let res = transaction.validate_fully_signed_input(&address);
        assert!(res.is_ok());

        let res = transaction.validate_inputs();
        assert!(res.is_ok());

        let res = transaction.validate_fully_signed_inputs();
        assert!(res.is_ok());

        let res = transaction.delete_input(&address);
        assert!(res.is_ok());

        let found = transaction.lookup_input(&address);
        assert!(!found);

        let res = transaction.get_input(&address);
        assert!(res.is_err());

        let res = transaction.validate_inputs();
        assert!(res.is_ok());
    }
}

#[test]
fn test_transaction_outputs() {
    let custom_len = 10;
    let mut transaction = Transaction::new().unwrap();

    for _ in 0..10 {
        let mut output = Output::random(custom_len).unwrap();

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

    let stage = Stage::random().unwrap();
    let mut transaction = Transaction::new().unwrap();
    transaction.stage = stage;
    transaction.update_id().unwrap();

    let mut max_distance = transaction.distance;

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64().unwrap();
        let threshold = weight;

        let signer = Signer { public_key, weight };
        let mut signers = Signers::new().unwrap();
        signers.add(&signer).unwrap();
        signers.set_threshold(threshold).unwrap();

        let amount = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

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

    let stage = Stage::random().unwrap();
    let mut transaction = Transaction::new().unwrap();
    transaction.stage = stage;
    transaction.update_id().unwrap();

    let mut input_balance = 0;
    let mut output_balance = 0;
    let mut expected_balance = 0i64;

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64().unwrap();
        let threshold = weight;

        let signer = Signer { public_key, weight };
        let mut signers = Signers::new().unwrap();
        signers.add(&signer).unwrap();
        signers.set_threshold(threshold).unwrap();

        let amount = 10;
        let tx_id = Digest::random().unwrap();
        let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let input = Input::new(&account, distance, amount).unwrap();

        transaction.add_input(&input).unwrap();
        input_balance += input.amount;
        expected_balance += input.amount as i64;

        let balance = transaction.balance().unwrap();
        assert_eq!(balance, expected_balance);
        assert_eq!(balance, input_balance as i64);

        let res = transaction.validate_balance();
        assert!(res.is_err());

        transaction.delete_input(&input.address()).unwrap();
        input_balance -= input.amount;
        expected_balance -= input.amount as i64;

        let balance = transaction.balance().unwrap();
        assert_eq!(balance, expected_balance);
        assert_eq!(balance, input_balance as i64);

        let res = transaction.validate_balance();
        assert!(res.is_ok());
    }

    assert_eq!(expected_balance, 0);

    let custom_len = 10;

    for _ in 0..10 {
        let mut output = Output::random(custom_len).unwrap();
        output.amount = 10;

        transaction.add_output(&output).unwrap();
        output_balance += output.amount;
        expected_balance -= output.amount as i64;

        let balance = transaction.balance().unwrap();
        assert_eq!(balance, expected_balance);
        assert_eq!(balance, -(output_balance as i64));

        let res = transaction.validate_balance();
        assert!(res.is_err());

        transaction.delete_output(&output.address).unwrap();
        output_balance -= output.amount;
        expected_balance += output.amount as i64;

        let balance = transaction.balance().unwrap();
        assert_eq!(expected_balance, 0);
        assert_eq!(balance, expected_balance);

        let res = transaction.validate_balance();
        assert!(res.is_ok());
    }

    let balance = transaction.balance().unwrap();
    assert_eq!(balance, expected_balance);

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

        let res = transaction.validate_mined();
        assert!(res.is_ok());

        let mut coinbase = transaction.coinbase.unwrap();

        if coinbase.nonce == u64::max_value() {
            coinbase.nonce = 0;
        } else {
            coinbase.nonce += 1;
        }

        transaction.coinbase = Some(coinbase);

        let res = transaction.validate_mined();
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

#[test]
fn test_transaction_storable() {
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_unqlite(max_value_size, max_size).unwrap();

    let stage = Stage::random().unwrap();

    let items: Vec<(Digest, Transaction)> = (0..10)
        .map(|_| {
            let mut transaction = Transaction::new().unwrap();
            transaction.stage = stage;
            transaction.update_id().unwrap();

            (transaction.id, transaction)
        })
        .collect();

    for (key, value) in &items {
        let res = Transaction::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = Transaction::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = Transaction::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = Transaction::get(&store, stage, &key);
        assert!(res.is_err());

        let res = Transaction::insert(&mut store, stage, &value);
        assert!(res.is_ok());

        let res = Transaction::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = Transaction::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().iter().next(), Some(value));

        let res = Transaction::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = Transaction::get(&store, stage, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = Transaction::remove(&mut store, stage, &key);
        assert!(res.is_ok());

        let res = Transaction::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = Transaction::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = Transaction::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = Transaction::get(&store, stage, &key);
        assert!(res.is_err());

        let res = Transaction::insert(&mut store, stage, &value);
        assert!(res.is_ok());

        let res = Transaction::clear(&mut store, stage);
        assert!(res.is_ok());

        let res = Transaction::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);
    }
}
