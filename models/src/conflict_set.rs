//! # Conflict Set
//!
//! `conflict_set` is the module containing the type used to register mutually conflicting
//! transactions.

use crate::error::Error;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use byteorder::{BigEndian, WriteBytesExt};
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeSet;
use store::traits::Store;

/// `ConflictSet` is the set used to represent a set of mutually conflicting transactions.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct ConflictSet {
    pub id: u64,
    pub stage: Stage,
    pub transactions: BTreeSet<Digest>,
    pub last: Option<Digest>,
    pub preferred: Option<Digest>,
    pub count: u64,
}

impl ConflictSet {
    /// `new` creates a new `ConflictSet`.
    pub fn new(id: u64, stage: Stage) -> ConflictSet {
        let mut set = ConflictSet::default();
        set.id = id;
        set.stage = stage;
        set
    }

    /// `lookup` looks up a `Transaction` id in the transactions set of the `ConflictSet`.
    pub fn lookup(&self, tx_id: &Digest) -> bool {
        self.transactions.contains(tx_id)
    }

    /// `add` adds a new `Transaction` id in the transactions set of the `ConflictSet`.
    pub fn add(&mut self, tx_id: Digest) {
        if !self.lookup(&tx_id) {
            self.transactions.insert(tx_id);

            self.last = Some(tx_id);

            if self.preferred.is_none() {
                self.preferred = Some(tx_id);
            }
        }
    }

    /// `remove` removes a `Transaction` from the transaction set of the `ConflictSet`.
    pub fn remove(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup(tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transactions.remove(tx_id);

        if let Some(ref last) = self.last {
            if last == tx_id {
                self.last = None;
            }
        }

        if let Some(ref preferred) = self.preferred {
            if preferred == tx_id {
                self.preferred = None;
            }
        }

        Ok(())
    }

    /// `set_last` sets the last transaction in the `ConflictSet`.
    pub fn set_last(&mut self, tx_id: Digest) -> Result<()> {
        if !self.lookup(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.last = Some(tx_id);

        Ok(())
    }

    /// `set_preferred` sets the preferred transaction in the `ConflictSet`.
    pub fn set_preferred(&mut self, tx_id: Digest) -> Result<()> {
        if !self.lookup(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.preferred = Some(tx_id);

        Ok(())
    }

    /// `validate` validates the `ConflictSet`.
    pub fn validate(&self) -> Result<()> {
        if let Some(last) = self.last {
            if !self.lookup(&last) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if let Some(preferred) = self.preferred {
            if !self.lookup(&preferred) {
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

    /// `clear` clears the `ConflictSet`.
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.last = None;
        self.preferred = None;
        self.count = 0;
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
    const KEY_PREFIX: u8 = 5;

    type Key = u64;

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.write_u64::<BigEndian>(*key)?;
        Ok(buf)
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
    ) -> Result<Vec<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
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

    fn sample(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: u32,
    ) -> Result<Vec<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.sample(from, to, count)?;
        let mut items = Vec::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.push(item);
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
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.insert(&key, &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.create(&key, &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.update(&key, &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, stage: Stage, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = Vec::new();

        for (k, v) in items {
            let k = <Self as Storable<S>>::key_to_bytes(stage, k)?;
            let v = v.to_bytes()?;
            let item = (k, v);
            _items.push(item);
        }

        let items: Vec<(&[u8], &[u8])> = _items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, stage: Stage, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, stage: Stage, keys: &[Self::Key]) -> Result<()> {
        let mut _keys = Vec::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            _keys.push(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(_store: &mut S, _stage: Stage, _min_time: Timestamp) -> Result<()> {
        Err(Error::NotImplemented)
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
fn test_conflict_set_ops() {
    use crypto::random::Random;

    let id = Random::u64().unwrap();
    let stage = Stage::random().unwrap();
    let mut conflict_set = ConflictSet::new(id, stage);

    let res = conflict_set.validate();
    assert!(res.is_ok());

    let tx_id_1 = Digest::random().unwrap();

    let found = conflict_set.lookup(&tx_id_1);
    assert!(!found);

    let res = conflict_set.remove(&tx_id_1);
    assert!(res.is_err());

    conflict_set.add(tx_id_1);

    assert_eq!(conflict_set.last, Some(tx_id_1));
    assert_eq!(conflict_set.preferred, Some(tx_id_1));

    let found = conflict_set.lookup(&tx_id_1);
    assert!(found);

    let res = conflict_set.remove(&tx_id_1);
    assert!(res.is_ok());

    assert_eq!(conflict_set.last, None);
    assert_eq!(conflict_set.preferred, None);

    let res = conflict_set.set_last(tx_id_1);
    assert!(res.is_err());

    let res = conflict_set.set_preferred(tx_id_1);
    assert!(res.is_err());

    conflict_set.add(tx_id_1);

    let tx_id_2 = Digest::random().unwrap();

    conflict_set.add(tx_id_2);

    assert_eq!(conflict_set.last, Some(tx_id_2));
    assert_eq!(conflict_set.preferred, Some(tx_id_1));

    let tx_id_3 = Digest::random().unwrap();

    let res = conflict_set.set_last(tx_id_3);
    assert!(res.is_err());

    let res = conflict_set.set_preferred(tx_id_3);
    assert!(res.is_err());

    let res = conflict_set.set_last(tx_id_1);
    assert!(res.is_ok());
    assert_eq!(conflict_set.last, Some(tx_id_1));

    let res = conflict_set.set_preferred(tx_id_2);
    assert!(res.is_ok());
    assert_eq!(conflict_set.preferred, Some(tx_id_2));

    conflict_set.clear();
    assert!(conflict_set.transactions.is_empty());
    assert_eq!(conflict_set.last, None);
    assert_eq!(conflict_set.preferred, None);
    assert_eq!(conflict_set.count, 0);
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

#[test]
fn test_conflict_set_storable() {
    use store::backend::BTreeStore;
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_btree(max_value_size, max_size).unwrap();

    let stage = Stage::random().unwrap();

    let items: Vec<(u64, ConflictSet)> = (0..10)
        .map(|id| (id, ConflictSet::new(id, stage)))
        .collect();

    for (key, value) in &items {
        let res = ConflictSet::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConflictSet::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![]);

        let res = <ConflictSet as Storable<BTreeStore>>::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConflictSet::get(&store, stage, &key);
        assert!(res.is_err());

        let res = ConflictSet::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = ConflictSet::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = ConflictSet::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);

        let res = <ConflictSet as Storable<BTreeStore>>::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = ConflictSet::get(&store, stage, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = <ConflictSet as Storable<BTreeStore>>::remove(&mut store, stage, &key);
        assert!(res.is_ok());

        let res = ConflictSet::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConflictSet::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![]);

        let res = <ConflictSet as Storable<BTreeStore>>::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConflictSet::get(&store, stage, &key);
        assert!(res.is_err());

        let res = ConflictSet::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = <ConflictSet as Storable<BTreeStore>>::clear(&mut store, stage);
        assert!(res.is_ok());

        let res = <ConflictSet as Storable<BTreeStore>>::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);
    }
}
