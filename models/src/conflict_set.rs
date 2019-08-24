//! # Conflict Set
//!
//! `conflict_set` is the module containing the type used to register mutually conflicting
//! transactions.

use crate::error::Error;
use crate::result::Result;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crate::transaction::Transaction;
use byteorder::{BigEndian, WriteBytesExt};
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeSet;
use store::traits::Store;

/// `ConflictSet` is the set used to represent a set of mutually conflicting transactions.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ConflictSet {
    pub id: u64,
    pub transactions: BTreeSet<Digest>,
    pub last: Option<Digest>,
    pub preferred: Option<Digest>,
    pub counter: u64,
}

impl ConflictSet {
    /// `new` creates a new `ConflictSet`.
    pub fn new(id: u64) -> ConflictSet {
        let mut set = ConflictSet::default();
        set.id = id;
        set
    }

    /// `lookup` looks up a `Transaction` in the transactions set.
    pub fn lookup(&self, transaction: &Transaction) -> bool {
        self.transactions.contains(&transaction.id)
    }

    /// `add` adds a new `Transaction` in the transactions set.
    pub fn add(&mut self, transaction: &Transaction) {
        if !self.lookup(transaction) {
            self.transactions.insert(transaction.id);
            self.last = Some(transaction.id);
        }
    }

    /// `remove` removes a `Transaction` from the transaction set.
    pub fn remove(&mut self, transaction: &Transaction) -> Result<()> {
        if !self.lookup(transaction) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transactions.remove(&transaction.id);

        if let Some(last) = self.last {
            if last == transaction.id {
                self.last = None;
            }
        }

        if let Some(preferred) = self.preferred {
            if preferred == transaction.id {
                self.preferred = None;
            }
        }

        Ok(())
    }

    /// `set_last` sets the last transaction in the `ConflictSet`.
    pub fn set_last(&mut self, transaction: &Transaction) -> Result<()> {
        if !self.lookup(transaction) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.last = Some(transaction.id);

        Ok(())
    }

    /// `set_preferred` sets the preferred transaction in the `ConflictSet`.
    pub fn set_preferred(&mut self, transaction: &Transaction) -> Result<()> {
        if !self.lookup(transaction) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.preferred = Some(transaction.id);

        Ok(())
    }

    /// `validate` validates the `ConflictSet`.
    pub fn validate(&self) -> Result<()> {
        if let Some(last) = self.last {
            if !self.transactions.contains(&last) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if let Some(preferred) = self.preferred {
            if !self.transactions.contains(&preferred) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if self.last.is_some() ^ self.preferred.is_some() {
            let err = Error::NotFound;
            return Err(err);
        }

        Ok(())
    }

    /// `to_bytes` converts the `ConflictSet` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `ConflictSet`.
    pub fn from_bytes(b: &[u8]) -> Result<ConflictSet> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `ConflictSet` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `ConflictSet`.
    pub fn from_json(s: &str) -> Result<ConflictSet> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for ConflictSet {
    const KEY_PREFIX: u8 = 4;

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
fn test_conflict_set_serialize_bytes() {
    let conflict_set_a = ConflictSet::default();

    let res = conflict_set_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = ConflictSet::from_bytes(&cbor);
    assert!(res.is_ok());
    let conflict_set_b = res.unwrap();

    assert_eq!(conflict_set_a, conflict_set_b)
}

#[test]
fn test_conflict_set_serialize_json() {
    let conflict_set_a = ConflictSet::default();

    let res = conflict_set_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = ConflictSet::from_json(&json);
    assert!(res.is_ok());
    let conflict_set_b = res.unwrap();

    assert_eq!(conflict_set_a, conflict_set_b)
}
