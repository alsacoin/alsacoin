//! # Consensus State
//!
//! `consensus_state` is the type used to manage the state of the Avalanche Consensus algorithm.

use crate::conflict_set::ConflictSet;
use crate::error::Error;
use crate::node::Node;
use crate::result::Result;
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
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ConsensusState {
    pub id: u64,
    pub known_transactions: BTreeSet<Digest>,
    pub queried_transactions: BTreeSet<Digest>,
    pub conflict_sets: BTreeSet<u64>,
    pub transaction_conflict_set: BTreeMap<Digest, u64>,
    pub transaction_chit: BTreeMap<Digest, u64>,
    pub transaction_confidence: BTreeMap<Digest, u64>,
    pub known_nodes: BTreeSet<Digest>,
}

impl ConsensusState {
    /// `new` creates a new `ConsensusState`.
    pub fn new(id: u64) -> ConsensusState {
        let mut set = ConsensusState::default();
        set.id = id;
        set
    }

    /// `lookup_known_transaction` looks up a `Transaction` in the known transactions set of the `ConsensusState`.
    pub fn lookup_known_transaction(&self, transaction: &Transaction) -> bool {
        self.known_transactions.contains(&transaction.id)
    }

    /// `add_known_transaction` adds a new `Transaction` in the known transactions set of the `ConsensusState`.
    pub fn add_known_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;

        if !self.lookup_known_transaction(transaction) {
            self.known_transactions.insert(transaction.id);
        }

        Ok(())
    }

    /// `remove_known_transaction` removes a `Transaction` from the known transaction set of the `ConsensusState`.
    pub fn remove_known_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        if !self.lookup_known_transaction(transaction) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.known_transactions.remove(&transaction.id);

        Ok(())
    }

    /// `lookup_queried_transaction` looks up a `Transaction` in the queried transactions set of the `ConsensusState`.
    pub fn lookup_queried_transaction(&self, transaction: &Transaction) -> bool {
        self.queried_transactions.contains(&transaction.id)
    }

    /// `add_queried_transaction` adds a new `Transaction` in the queried transactions set of the `ConsensusState`.
    pub fn add_queried_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;

        if !self.lookup_known_transaction(transaction) {
            let err = Error::NotFound;
            return Err(err);
        }

        if !self.lookup_queried_transaction(transaction) {
            self.queried_transactions.insert(transaction.id);
        }

        Ok(())
    }

    /// `remove_queried_transaction` removes a `Transaction` from the queried transaction set of the `ConsensusState`.
    pub fn remove_queried_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        if !self.lookup_queried_transaction(transaction) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.queried_transactions.remove(&transaction.id);

        Ok(())
    }

    /// `lookup_conflict_set` looks up a `ConflictSet` in the queried conflict_sets set of the `ConsensusState`.
    pub fn lookup_conflict_set(&self, conflict_set: &ConflictSet) -> bool {
        self.conflict_sets.contains(&conflict_set.id)
    }

    /// `add_conflict_set` adds a new `ConflictSet` in the queried conflict_sets set of the `ConsensusState`.
    pub fn add_conflict_set(&mut self, conflict_set: &ConflictSet) -> Result<()> {
        conflict_set.validate()?;

        if !self.lookup_conflict_set(conflict_set) {
            self.conflict_sets.insert(conflict_set.id);
        }

        Ok(())
    }

    /// `remove_conflict_set` removes a `ConflictSet` from the queried conflict_set set of the `ConsensusState`.
    pub fn remove_conflict_set(&mut self, conflict_set: &ConflictSet) -> Result<()> {
        if !self.lookup_conflict_set(conflict_set) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.conflict_sets.remove(&conflict_set.id);

        Ok(())
    }

    /// `lookup_known_node` looks up a `Node` in the known nodes set of the `ConsensusState`.
    pub fn lookup_known_node(&self, node: &Node) -> bool {
        self.known_nodes.contains(&node.id)
    }

    /// `add_known_node` adds a new `Node` in the known nodes set of the `ConsensusState`.
    pub fn add_known_node(&mut self, node: &Node) {
        if !self.lookup_known_node(node) {
            self.known_nodes.insert(node.id);
        }
    }

    /// `remove_known_node` removes a `Node` from the known node set of the `ConsensusState`.
    pub fn remove_known_node(&mut self, node: &Node) -> Result<()> {
        if !self.lookup_known_node(node) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.known_nodes.remove(&node.id);

        Ok(())
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
    const KEY_PREFIX: u8 = 5;

    type Key = u64;

    fn key_to_bytes(key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.write_u64::<BigEndian>(*key)?;
        Ok(buf)
    }

    fn lookup(store: &S, key: &Self::Key) -> Result<bool> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        store.lookup(&key).map_err(|e| e.into())
    }

    fn get(store: &S, key: &Self::Key) -> Result<Self> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let buf = store.get(&key)?;
        Self::from_bytes(&buf)
    }

    fn query(
        store: &S,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.query(from, to, count, skip)?;
        let mut items = Vec::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.push(item);
        }

        Ok(items)
    }

    fn count(
        store: &S,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.insert(&key, &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.create(&key, &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.update(&key, &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = Vec::new();

        for (k, v) in items {
            let key = <Self as Storable<S>>::key_to_bytes(k)?;
            let value = v.to_bytes()?;
            let item = (key, value);
            _items.push(item);
        }

        let items: Vec<(&[u8], &[u8])> = _items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, keys: &[Self::Key]) -> Result<()> {
        let mut _keys = Vec::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            _keys.push(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(_store: &mut S, _min_time: Timestamp) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn clear(store: &mut S) -> Result<()> {
        let from = Some(vec![<Self as Storable<S>>::KEY_PREFIX]);
        let from = from.as_ref().map(|from| from.as_slice());
        let to = Some(vec![<Self as Storable<S>>::KEY_PREFIX + 1]);
        let to = to.as_ref().map(|to| to.as_slice());
        store.remove_range(from, to, None).map_err(|e| e.into())
    }
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
