//! # Consensus State
//!
//! `consensus_state` is the type used to manage the state of the Avalanche Consensus algorithm.

use crate::account::Account;
use crate::address::Address;
use crate::conflict_set::ConflictSet;
use crate::error::Error;
use crate::node::Node;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crate::transaction::Transaction;
use byteorder::{BigEndian, WriteBytesExt};
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::{BTreeMap, BTreeSet};
use store::traits::Store;

/// `ConsensusState` represents the Avalanche Consensus state.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct ConsensusState {
    pub id: u64,
    pub stage: Stage,
    pub eve_account_address: Address,
    pub eve_transaction_id: Digest,
    pub seed_nodes: BTreeSet<Digest>,
    pub known_transactions: BTreeSet<Digest>,
    pub transaction_successors: BTreeMap<Digest, BTreeSet<Digest>>,
    pub queried_transactions: BTreeSet<Digest>,
    pub transaction_conflict_set: BTreeMap<Digest, Address>,
    pub transaction_chit: BTreeMap<Digest, bool>,
    pub transaction_confidence: BTreeMap<Digest, u64>,
    pub known_nodes: BTreeSet<Digest>,
}

impl ConsensusState {
    /// `new` creates a new `ConsensusState`.
    pub fn new(
        id: u64,
        stage: Stage,
        eve_account_address: &Address,
        eve_transaction_id: &Digest,
        seed_nodes: &BTreeSet<Digest>,
    ) -> ConsensusState {
        let mut state = ConsensusState::default();
        state.id = id;
        state.stage = stage;
        state.eve_account_address = eve_account_address.to_owned();
        state.eve_transaction_id = eve_transaction_id.to_owned();
        state.seed_nodes = seed_nodes.to_owned();
        state
    }

    /// `lookup_known_transaction` looks up a `Transaction` id in the known transactions set of the `ConsensusState`.
    pub fn lookup_known_transaction(&self, tx_id: &Digest) -> bool {
        self.known_transactions.contains(tx_id)
    }

    /// `add_known_transaction` adds a new `Transaction` id in the known transactions set of the `ConsensusState`.
    pub fn add_known_transaction(&mut self, tx_id: Digest) {
        if !self.lookup_known_transaction(&tx_id) {
            self.known_transactions.insert(tx_id);
        }
    }

    /// `remove_known_transaction` removes a `Transaction` id from the known transaction set of the `ConsensusState`.
    pub fn remove_known_transaction(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_known_transaction(tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.known_transactions.remove(tx_id);

        Ok(())
    }

    /// `lookup_transaction_successors` looks up the transaction_successors of a `Transaction`.
    pub fn lookup_transaction_successors(&self, tx_id: &Digest) -> bool {
        self.transaction_successors.contains_key(tx_id)
    }

    /// `get_transaction_successors` returns a `Transaction` transaction_successors.
    pub fn get_transaction_successors(&self, tx_id: &Digest) -> Option<BTreeSet<Digest>> {
        self.transaction_successors.get(tx_id).cloned()
    }

    /// `add_transaction_successors` adds the transaction_successors of a `Transaction`.
    pub fn add_transaction_successors(
        &mut self,
        tx_id: Digest,
        succ_ids: BTreeSet<Digest>,
    ) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        if succ_ids.contains(&tx_id) {
            let err = Error::InvalidId;
            return Err(err);
        }

        self.transaction_successors.insert(tx_id, succ_ids);

        Ok(())
    }

    /// `add_transaction_successor` adds a single successor of a `Transaction`
    pub fn add_transaction_successor(&mut self, tx_id: &Digest, succ_id: Digest) -> Result<()> {
        if tx_id == &succ_id {
            let err = Error::InvalidId;
            return Err(err);
        }

        if !self.lookup_known_transaction(tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        let mut succ_ids = self.get_transaction_successors(tx_id).unwrap_or_default();
        succ_ids.insert(succ_id);

        self.transaction_successors.insert(*tx_id, succ_ids);

        Ok(())
    }

    /// `remove_transaction_successors` removes the transaction_successors of a `Transaction`.
    pub fn remove_transaction_successors(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_transaction_successors(tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_successors.remove(tx_id);

        Ok(())
    }

    /// `lookup_queried_transaction` looks up a `Transaction` id in the queried transactions set of the `ConsensusState`.
    pub fn lookup_queried_transaction(&self, tx_id: &Digest) -> bool {
        self.queried_transactions.contains(tx_id)
    }

    /// `add_queried_transaction` adds a new `Transaction` id in the queried transactions set of the `ConsensusState`.
    pub fn add_queried_transaction(&mut self, tx_id: Digest) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        if !self.lookup_queried_transaction(&tx_id) {
            self.queried_transactions.insert(tx_id);
        }

        Ok(())
    }

    /// `remove_queried_transaction` removes a `Transaction` id from the queried transaction set of the `ConsensusState`.
    pub fn remove_queried_transaction(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_queried_transaction(tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.queried_transactions.remove(tx_id);

        Ok(())
    }

    /// `lookup_transaction_conflict_set` looks up a `Transaction` id in the transaction
    /// conflict set of the `ConsensusState`.
    pub fn lookup_transaction_conflict_set(&self, tx_id: &Digest) -> bool {
        self.transaction_conflict_set.contains_key(&tx_id)
    }

    /// `get_transaction_conflict_set` gets the conflict_set of a `Transaction`.
    pub fn get_transaction_conflict_set(&self, tx_id: &Digest) -> Option<Address> {
        self.transaction_conflict_set.get(tx_id).copied()
    }

    /// `set_transaction_conflict_set` sets a known `Transaction` conflict set id in
    /// the `ConsensusState`.
    pub fn set_transaction_conflict_set(&mut self, tx_id: Digest, address: Address) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_conflict_set.insert(tx_id, address);

        Ok(())
    }

    /// `remove_transaction_conflict_set` removes a known `Transaction` conflict set id in
    /// the `ConsensusState`.
    pub fn remove_transaction_conflict_set(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        if !self.lookup_transaction_conflict_set(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_conflict_set.remove(tx_id);

        Ok(())
    }

    /// `lookup_transaction_chit` looks up a `Transaction` id in the transaction
    /// conflict set of the `ConsensusState`.
    pub fn lookup_transaction_chit(&self, tx_id: &Digest) -> bool {
        self.transaction_chit.contains_key(&tx_id)
    }

    /// `get_transaction_chit` gets the chit of a `Transaction`.
    pub fn get_transaction_chit(&self, tx_id: &Digest) -> Option<bool> {
        self.transaction_chit.get(tx_id).copied()
    }

    /// `set_transaction_chit` sets a known `Transaction` chit in
    /// the `ConsensusState`.
    pub fn set_transaction_chit(&mut self, tx_id: Digest, chit: bool) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_chit.insert(tx_id, chit);

        Ok(())
    }

    /// `remove_transaction_chit` removes a known `Transaction` chit in
    /// the `ConsensusState`.
    pub fn remove_transaction_chit(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        if !self.lookup_transaction_chit(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_chit.remove(tx_id);

        Ok(())
    }

    /// `lookup_transaction_confidence` looks up a `Transaction` id in the transaction
    /// conflict set of the `ConsensusState`.
    pub fn lookup_transaction_confidence(&self, tx_id: &Digest) -> bool {
        self.transaction_confidence.contains_key(&tx_id)
    }

    /// `get_transaction_confidence` gets the confidence of a `Transaction`.
    pub fn get_transaction_confidence(&self, tx_id: &Digest) -> Option<u64> {
        self.transaction_confidence.get(tx_id).copied()
    }

    /// `set_transaction_confidence` sets a known `Transaction` confidence in
    /// the `ConsensusState`.
    pub fn set_transaction_confidence(&mut self, tx_id: Digest, confidence: u64) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_confidence.insert(tx_id, confidence);

        Ok(())
    }

    /// `remove_transaction_confidence` removes a known `Transaction` confidence in
    /// the `ConsensusState`.
    pub fn remove_transaction_confidence(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_known_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        if !self.lookup_transaction_confidence(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transaction_confidence.remove(tx_id);

        Ok(())
    }

    /// `lookup_known_node` looks up a `Node` id in the known nodes set of the `ConsensusState`.
    pub fn lookup_known_node(&self, node_id: &Digest) -> bool {
        self.known_nodes.contains(node_id)
    }

    /// `add_known_node` adds a new `Node` id address in the known nodes set of the `ConsensusState`.
    pub fn add_known_node(&mut self, node_id: Digest) {
        if !self.lookup_known_node(&node_id) {
            self.known_nodes.insert(node_id);
        }
    }

    /// `remove_known_node` removes a `Node` from the known node set of the `ConsensusState`.
    pub fn remove_known_node(&mut self, node_id: &Digest) -> Result<()> {
        if !self.lookup_known_node(node_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.known_nodes.remove(node_id);

        Ok(())
    }

    /// `validate` validates the `ConsensusState`.
    pub fn validate(&self) -> Result<()> {
        for id in &self.queried_transactions {
            if !self.lookup_known_transaction(&id) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        for tx_id in self.transaction_conflict_set.keys() {
            if !self.lookup_known_transaction(&tx_id) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        for id in self.transaction_chit.keys() {
            if !self.lookup_known_transaction(&id) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        for id in self.transaction_confidence.keys() {
            if !self.lookup_known_transaction(&id) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `clear` clears the `ConsensusState`.
    pub fn clear(&mut self) {
        self.known_transactions.clear();
        self.queried_transactions.clear();
        self.transaction_conflict_set.clear();
        self.transaction_chit.clear();
        self.transaction_confidence.clear();
        self.known_nodes.clear();
    }

    /// `to_bytes` converts the `ConsensusState` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `ConsensusState`.
    pub fn from_bytes(b: &[u8]) -> Result<ConsensusState> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `ConsensusState` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `ConsensusState`.
    pub fn from_json(s: &str) -> Result<ConsensusState> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for ConsensusState {
    const KEY_PREFIX: u8 = 6;

    type Key = u64;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.write_u64::<BigEndian>(*key)?;
        Ok(buf)
    }

    fn validate_single(store: &S, stage: Stage, value: &Self) -> Result<()> {
        if value.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        value.validate()?;

        if !Account::lookup(store, stage, &value.eve_account_address)? {
            let err = Error::NotFound;
            return Err(err);
        }

        if !Transaction::lookup(store, stage, &value.eve_transaction_id)? {
            let err = Error::NotFound;
            return Err(err);
        }

        for id in &value.seed_nodes {
            if !Node::lookup(store, stage, &id)? {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        for id in &value.known_transactions {
            if !Transaction::lookup(store, stage, &id)? {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        for address in value.transaction_conflict_set.keys() {
            if !ConflictSet::lookup(store, stage, &address)? {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        for id in &value.known_nodes {
            if !Node::lookup(store, stage, &id)? {
                let err = Error::NotFound;
                return Err(err);
            }
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

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.insert(&store_key, &store_value).map_err(|e| e.into())
    }

    fn create(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.create(&store_key, &store_value).map_err(|e| e.into())
    }

    fn update(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.update(&store_key, &store_value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, stage: Stage, values: &BTreeSet<Self>) -> Result<()> {
        let mut items = BTreeSet::new();

        for value in values {
            Self::validate_single(store, stage, value)?;

            let key = <Self as Storable<S>>::key(value);
            let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
            let store_value = value.to_bytes()?;
            let item = (store_key, store_value);
            items.insert(item);
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

    fn cleanup(store: &mut S, stage: Stage, _min_time: Option<Timestamp>) -> Result<()> {
        let states = ConsensusState::query(store, stage, None, None, None, None)?;

        let max_id: u64 = states.iter().map(|state| state.id).max().unwrap_or(0);

        if states.len() > 1 {
            for state in states {
                if state.id != max_id {
                    ConsensusState::remove(store, stage, &state.id)?;
                }
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
fn test_consensus_state_known_transactions_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();

    let eve_account_address = Address::random().unwrap();
    let eve_transaction_id = Digest::random().unwrap();

    let mut seed = BTreeSet::new();
    for _ in 0..10 {
        let id = Digest::random().unwrap();
        seed.insert(id);
    }

    let mut state =
        ConsensusState::new(id, stage, &eve_account_address, &eve_transaction_id, &seed);

    let res = state.validate();
    assert!(res.is_ok());

    let tx_id = Digest::random().unwrap();

    let found = state.lookup_known_transaction(&tx_id);
    assert!(!found);

    let res = state.remove_known_transaction(&tx_id);
    assert!(res.is_err());

    state.add_known_transaction(tx_id);

    let res = state.validate();
    assert!(res.is_ok());

    let found = state.lookup_known_transaction(&tx_id);
    assert!(found);

    let res = state.remove_known_transaction(&tx_id);
    assert!(res.is_ok());

    state.clear();
    assert!(state.known_transactions.is_empty());

    let res = state.validate();
    assert!(res.is_ok());
}

#[test]
fn test_consensus_state_queried_transactions_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();

    let eve_account_address = Address::random().unwrap();
    let eve_transaction_id = Digest::random().unwrap();

    let mut seed = BTreeSet::new();
    for _ in 0..10 {
        let id = Digest::random().unwrap();
        seed.insert(id);
    }

    let mut state =
        ConsensusState::new(id, stage, &eve_account_address, &eve_transaction_id, &seed);

    let res = state.validate();
    assert!(res.is_ok());

    let tx_id_1 = Digest::random().unwrap();
    let tx_id_2 = Digest::random().unwrap();

    state.add_known_transaction(tx_id_1);

    let found = state.lookup_queried_transaction(&tx_id_2);
    assert!(!found);

    let res = state.remove_queried_transaction(&tx_id_2);
    assert!(res.is_err());

    let res = state.add_queried_transaction(tx_id_2);
    assert!(res.is_err());

    let found = state.lookup_queried_transaction(&tx_id_1);
    assert!(!found);

    let res = state.remove_queried_transaction(&tx_id_1);
    assert!(res.is_err());

    let res = state.add_queried_transaction(tx_id_1);
    assert!(res.is_ok());

    let res = state.validate();
    assert!(res.is_ok());

    let found = state.lookup_queried_transaction(&tx_id_1);
    assert!(found);

    let res = state.remove_queried_transaction(&tx_id_1);
    assert!(res.is_ok());

    state.clear();
    assert!(state.queried_transactions.is_empty());

    let res = state.validate();
    assert!(res.is_ok());
}

#[test]
fn test_consensus_state_transaction_conflict_sets_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();

    let eve_account_address = Address::random().unwrap();
    let eve_transaction_id = Digest::random().unwrap();

    let mut seed = BTreeSet::new();
    for _ in 0..10 {
        let id = Digest::random().unwrap();
        seed.insert(id);
    }

    let mut state =
        ConsensusState::new(id, stage, &eve_account_address, &eve_transaction_id, &seed);

    let res = state.validate();
    assert!(res.is_ok());

    let tx_id = Digest::random().unwrap();
    let cs_addr = Address::random().unwrap();

    state.add_known_transaction(tx_id);

    let found = state.lookup_transaction_conflict_set(&tx_id);
    assert!(!found);

    let res = state.remove_transaction_conflict_set(&tx_id);
    assert!(res.is_err());

    let res = state.set_transaction_conflict_set(tx_id, cs_addr);
    assert!(res.is_ok());

    let res = state.validate();
    assert!(res.is_ok());

    let found = state.lookup_transaction_conflict_set(&tx_id);
    assert!(found);

    let opt = state.get_transaction_conflict_set(&tx_id);
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), cs_addr);

    let res = state.remove_transaction_conflict_set(&tx_id);
    assert!(res.is_ok());

    state.clear();
    assert!(state.transaction_conflict_set.is_empty());

    let res = state.validate();
    assert!(res.is_ok());
}

#[test]
fn test_consensus_state_transaction_chit_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();

    let eve_account_address = Address::random().unwrap();
    let eve_transaction_id = Digest::random().unwrap();

    let mut seed = BTreeSet::new();
    for _ in 0..10 {
        let id = Digest::random().unwrap();
        seed.insert(id);
    }

    let mut state =
        ConsensusState::new(id, stage, &eve_account_address, &eve_transaction_id, &seed);

    let res = state.validate();
    assert!(res.is_ok());

    let tx_id = Digest::random().unwrap();
    let tx_chit = Random::u32_range(0, 2).unwrap() != 0;

    state.add_known_transaction(tx_id);

    let found = state.lookup_transaction_chit(&tx_id);
    assert!(!found);

    let res = state.remove_transaction_chit(&tx_id);
    assert!(res.is_err());

    let res = state.set_transaction_chit(tx_id, tx_chit);
    assert!(res.is_ok());

    let res = state.validate();
    assert!(res.is_ok());

    let found = state.lookup_transaction_chit(&tx_id);
    assert!(found);

    let opt = state.get_transaction_chit(&tx_id);
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), tx_chit);

    let res = state.remove_transaction_chit(&tx_id);
    assert!(res.is_ok());

    state.clear();
    assert!(state.transaction_chit.is_empty());

    let res = state.validate();
    assert!(res.is_ok());
}

#[test]
fn test_consensus_state_transaction_confidence_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();

    let eve_account_address = Address::random().unwrap();
    let eve_transaction_id = Digest::random().unwrap();

    let mut seed = BTreeSet::new();
    for _ in 0..10 {
        let id = Digest::random().unwrap();
        seed.insert(id);
    }

    let mut state =
        ConsensusState::new(id, stage, &eve_account_address, &eve_transaction_id, &seed);

    let res = state.validate();
    assert!(res.is_ok());

    let tx_id = Digest::random().unwrap();
    let tx_confidence = Random::u64().unwrap();

    state.add_known_transaction(tx_id);

    let found = state.lookup_transaction_confidence(&tx_id);
    assert!(!found);

    let res = state.remove_transaction_confidence(&tx_id);
    assert!(res.is_err());

    let res = state.set_transaction_confidence(tx_id, tx_confidence);
    assert!(res.is_ok());

    let res = state.validate();
    assert!(res.is_ok());

    let found = state.lookup_transaction_confidence(&tx_id);
    assert!(found);

    let opt = state.get_transaction_confidence(&tx_id);
    assert!(opt.is_some());
    assert_eq!(opt.unwrap(), tx_confidence);

    let res = state.remove_transaction_confidence(&tx_id);
    assert!(res.is_ok());

    state.clear();
    assert!(state.transaction_confidence.is_empty());

    let res = state.validate();
    assert!(res.is_ok());
}

#[test]
fn test_consensus_state_known_nodes_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();

    let eve_account_address = Address::random().unwrap();
    let eve_transaction_id = Digest::random().unwrap();

    let mut seed = BTreeSet::new();
    for _ in 0..10 {
        let id = Digest::random().unwrap();
        seed.insert(id);
    }

    let mut state =
        ConsensusState::new(id, stage, &eve_account_address, &eve_transaction_id, &seed);

    let res = state.validate();
    assert!(res.is_ok());

    let node_id = Digest::random().unwrap();

    let found = state.lookup_known_node(&node_id);
    assert!(!found);

    let res = state.remove_known_node(&node_id);
    assert!(res.is_err());

    state.add_known_node(node_id);

    let res = state.validate();
    assert!(res.is_ok());

    let found = state.lookup_known_node(&node_id);
    assert!(found);

    let res = state.remove_known_node(&node_id);
    assert!(res.is_ok());

    state.clear();
    assert!(state.known_nodes.is_empty());

    let res = state.validate();
    assert!(res.is_ok());
}

#[test]
fn test_consensus_state_serialize_bytes() {
    let consensus_state_a = ConsensusState::default();

    let res = consensus_state_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = ConsensusState::from_bytes(&cbor);
    assert!(res.is_ok());
    let consensus_state_b = res.unwrap();

    assert_eq!(consensus_state_a, consensus_state_b)
}

#[test]
fn test_consensus_state_serialize_json() {
    let consensus_state_a = ConsensusState::default();

    let res = consensus_state_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = ConsensusState::from_json(&json);
    assert!(res.is_ok());
    let consensus_state_b = res.unwrap();

    assert_eq!(consensus_state_a, consensus_state_b)
}

#[test]
fn test_consensus_state_storable() {
    use crate::account::Account;
    use crate::signers::Signers;
    use crate::transaction::Transaction;
    use crate::wallet::Wallet;
    use crypto::random::Random;
    use store::backend::BTreeStore;
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 12;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_btree(max_value_size, max_size).unwrap();

    let stage = Stage::random().unwrap();

    let wallet = Wallet::new(stage).unwrap();
    let weight = 1;
    let signer = wallet.to_signer(weight).unwrap();
    let mut signers = Signers::new().unwrap();
    signers.set_threshold(weight).unwrap();
    signers.add(&signer).unwrap();

    let mut account = Account::new_eve(stage, &signers).unwrap();
    let transaction = Transaction::new_eve(stage, &account.address()).unwrap();

    Transaction::create(&mut store, stage, &transaction).unwrap();

    account.transaction_id = Some(transaction.id);
    account.counter += 1;

    Account::create(&mut store, stage, &account).unwrap();

    let address = account.address();
    let tx_id = transaction.id;

    let node_address_len = 100;

    let items: Vec<(u64, ConsensusState)> = (0..10)
        .map(|id| {
            let mut seed = BTreeSet::new();
            for _ in 0..10 {
                let node_address = Random::bytes(node_address_len).unwrap();
                let node = Node::new(stage, &node_address);
                Node::create(&mut store, stage, &node).unwrap();
                seed.insert(node.id);
            }

            let state = ConsensusState::new(id, stage, &address, &tx_id, &seed);

            (id, state)
        })
        .collect();

    for (key, value) in &items {
        let res = ConsensusState::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConsensusState::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = ConsensusState::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConsensusState::get(&store, stage, &key);
        assert!(res.is_err());

        let res = ConsensusState::insert(&mut store, stage, &value);
        assert!(res.is_ok());

        let res = ConsensusState::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = ConsensusState::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().iter().next(), Some(value));

        let res = ConsensusState::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = ConsensusState::get(&store, stage, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = ConsensusState::remove(&mut store, stage, &key);
        assert!(res.is_ok());

        let res = ConsensusState::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConsensusState::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = ConsensusState::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConsensusState::get(&store, stage, &key);
        assert!(res.is_err());

        ConsensusState::insert(&mut store, stage, &value).unwrap();

        let count = ConsensusState::count(&store, stage, Some(*key), None, None).unwrap();
        assert_eq!(count, 1);
    }

    let count = ConsensusState::count(&store, stage, None, None, None).unwrap();

    let max_key = count as u64;

    let res = ConsensusState::cleanup(&mut store, stage, None);
    assert!(res.is_ok());

    assert!(res.is_ok());

    let res = ConsensusState::lookup(&store, stage, &max_key);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(found);

    let res = <ConsensusState as Storable<BTreeStore>>::clear(&mut store, stage);
    assert!(res.is_ok());

    let count = ConsensusState::count(&store, stage, None, None, None).unwrap();
    assert_eq!(count, 0);

    let res = ConsensusState::lookup(&store, stage, &max_key);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(!found);
}
