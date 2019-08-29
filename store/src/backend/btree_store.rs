//! # BTreeStore
//
// `btree_map_store` contains the `BTreeMap` store backend type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::{MemoryStore, Store};
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// `BTreeStore` is an implementor of `Store` built on a `BTreeMap`.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BTreeStore {
    db: BTreeMap<Vec<u8>, Vec<u8>>,
    max_value_size: u32,
    max_size: u32,
    keys_size: u32,
    values_size: u32,
}

impl BTreeStore {
    /// `new` creates a new `BTreeStore`.
    pub fn new(max_value_size: u32, max_size: u32) -> Result<BTreeStore> {
        if max_size < max_value_size {
            let err = Error::InvalidSize;
            return Err(err);
        }

        let store = BTreeStore {
            db: BTreeMap::default(),
            max_value_size,
            max_size,
            keys_size: 0,
            values_size: 0,
        };

        Ok(store)
    }

    /// `size` returns the size of the `BTreeStore`.
    pub fn size(&self) -> u32 {
        self.keys_size + self.values_size
    }

    /// `_lookup` looks up a key-value pair from the `BTreeStore`.
    fn _lookup(&self, key: &[u8]) -> bool {
        self.db.contains_key(key)
    }

    /// `_get` gets a key-value pair from the `BTreeStore`.
    fn _get(&self, key: &[u8]) -> Result<Vec<u8>> {
        match self.db.get(key) {
            Some(value) => Ok(value.to_owned()),
            None => {
                let err = Error::NotFound;
                Err(err)
            }
        }
    }

    /// `_query` returns a list of values from the `BTreeStore`.
    fn _query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Vec<u8>>> {
        let res: Vec<Vec<u8>> = if let Some(from) = from {
            if let Some(to) = to {
                if from > to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }

                if let Some(skip) = skip {
                    if let Some(count) = count {
                        self.db
                            .iter()
                            .filter(|(k, _)| (from <= k) && (to > k))
                            .skip(skip as usize)
                            .take(count as usize)
                            .map(|(_, v)| v.to_owned())
                            .collect()
                    } else {
                        self.db
                            .iter()
                            .filter(|(k, _)| (from <= k) && (to > k))
                            .skip(skip as usize)
                            .map(|(_, v)| v.to_owned())
                            .collect()
                    }
                } else {
                    self.db
                        .iter()
                        .filter(|(k, _)| (from <= k) && (to > k))
                        .map(|(_, v)| v.to_owned())
                        .collect()
                }
            } else if let Some(skip) = skip {
                if let Some(count) = count {
                    self.db
                        .iter()
                        .filter(|(k, _)| (from <= k))
                        .skip(skip as usize)
                        .take(count as usize)
                        .map(|(_, v)| v.to_owned())
                        .collect()
                } else {
                    self.db
                        .iter()
                        .filter(|(k, _)| (from <= k))
                        .skip(skip as usize)
                        .map(|(_, v)| v.to_owned())
                        .collect()
                }
            } else {
                self.db
                    .iter()
                    .filter(|(k, _)| (from <= k))
                    .map(|(_, v)| v.to_owned())
                    .collect()
            }
        } else if let Some(skip) = skip {
            if let Some(count) = count {
                self.db
                    .iter()
                    .skip(skip as usize)
                    .take(count as usize)
                    .map(|(_, v)| v.to_owned())
                    .collect()
            } else {
                self.db
                    .iter()
                    .skip(skip as usize)
                    .map(|(_, v)| v.to_owned())
                    .collect()
            }
        } else {
            self.db.iter().map(|(_, v)| v.to_owned()).collect()
        };

        Ok(res)
    }

    /// `_sample` samples values from the `BTreeStore`.
    fn _sample(&self, from: Option<&[u8]>, to: Option<&[u8]>, count: u32) -> Result<Vec<Vec<u8>>> {
        if let Some(from) = from {
            if let Some(to) = to {
                if from > to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }
            }
        }

        let len = self.count(from, to, Some(count))?;

        let count = u32::min(count, len);

        let values = self.query(from, to, Some(count), None)?;

        let idxs: Vec<u32> = Random::u32_sample_unique_range(0, len, count)?;

        let mut res = Vec::new();

        for (idx, value) in values.iter().enumerate() {
            if idxs.contains(&(idx as u32)) {
                res.push(value.clone());
            }
        }

        Ok(res)
    }

    /// `_count` returns the count of a list of values from the `BTreeStore`.
    fn _count(&self, from: Option<&[u8]>, to: Option<&[u8]>, skip: Option<u32>) -> Result<u32> {
        let res: u32 = if let Some(from) = from {
            if let Some(to) = to {
                if from > to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }

                if let Some(skip) = skip {
                    self.db
                        .iter()
                        .filter(|(k, _)| (from <= k) && (to > k))
                        .skip(skip as usize)
                        .count() as u32
                } else {
                    self.db
                        .iter()
                        .filter(|(k, _)| (from <= k) && (to > k))
                        .count() as u32
                }
            } else if let Some(skip) = skip {
                self.db
                    .iter()
                    .filter(|(k, _)| (from <= k))
                    .skip(skip as usize)
                    .count() as u32
            } else {
                self.db.iter().filter(|(k, _)| (from <= k)).count() as u32
            }
        } else if let Some(skip) = skip {
            self.db.iter().skip(skip as usize).count() as u32
        } else {
            self.db.iter().count() as u32
        };

        Ok(res)
    }

    /// `_insert` inserts a binary key-value pair in the `BTreeStore`.
    fn _insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let key_size = key.len() as u32;
        let value_size = value.len() as u32;

        if value_size > self.get_max_value_size() {
            let err = Error::InvalidSize;
            return Err(err);
        }

        if key_size + value_size + self.size() > self.get_max_size() {
            let err = Error::InvalidSize;
            return Err(err);
        }

        self.db.insert(key.to_owned(), value.to_owned());
        self.keys_size += key_size;
        self.values_size += value_size;
        Ok(())
    }

    /// `_create` inserts a non-existing binary key-value pair in the `BTreeStore`.
    fn _create(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if self._lookup(key) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_update` updates an existing key-value pair in the `BTreeStore`.
    pub fn _update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_remove` removes a key-value pair from the `BTreeStore`.
    fn _remove(&mut self, key: &[u8]) -> Result<()> {
        match self.db.remove(key) {
            Some(value) => {
                self.keys_size -= key.len() as u32;
                self.values_size -= value.len() as u32;
                Ok(())
            }
            None => {
                let err = Error::NotFound;
                Err(err)
            }
        }
    }

    fn _remove_range_complete(&mut self, from: &[u8], to: &[u8], skip: u32) -> Result<()> {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut skipped = 0;

        // TODO: that clone
        for (key, _) in self.db.clone().iter().filter(|(k, _)| from <= k && to > k) {
            if skipped >= skip {
                self._remove(&key)?;
            } else {
                skipped += 1;
            }
        }

        Ok(())
    }

    fn _remove_range_no_skip(&mut self, from: &[u8], to: &[u8]) -> Result<()> {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        // TODO: that clone
        for (key, _) in self.db.clone().iter().filter(|(k, _)| from <= k && to > k) {
            self._remove(&key)?;
        }

        Ok(())
    }

    fn _remove_range_no_from(&mut self, to: &[u8], skip: u32) -> Result<()> {
        let mut skipped = 0;

        // TODO: that clone
        for (key, _) in self.db.clone().iter().filter(|(k, _)| to > k) {
            if to > key.as_slice() {
                if skipped >= skip {
                    self._remove(&key)?;
                } else {
                    skipped += 1;
                }
            }
        }

        Ok(())
    }

    fn _remove_range_no_from_no_skip(&mut self, to: &[u8]) -> Result<()> {
        // TODO: that clone
        for (key, _) in self.db.clone().iter().filter(|(k, _)| to > k) {
            self._remove(&key)?;
        }

        Ok(())
    }

    fn _remove_range_no_to(&mut self, from: &[u8], skip: u32) -> Result<()> {
        let mut skipped = 0;

        // TODO: that clone
        for (key, _) in self.db.clone().iter().filter(|(k, _)| from <= k) {
            if skipped >= skip {
                self._remove(&key)?;
            } else {
                skipped += 1;
            }
        }

        Ok(())
    }

    fn _remove_range_no_to_no_skip(&mut self, from: &[u8]) -> Result<()> {
        // TODO: that clone
        for (key, _) in self.db.clone().iter().filter(|(k, _)| from <= k) {
            self._remove(&key)?;
        }

        Ok(())
    }

    fn _remove_range_no_from_no_to(&mut self, skip: u32) -> Result<()> {
        let mut skipped = 0;

        // TODO: that clone
        for (key, _) in self.db.clone().iter() {
            if skipped >= skip {
                self._remove(&key)?;
            } else {
                skipped += 1;
            }
        }

        Ok(())
    }

    fn _remove_range_none(&mut self) -> Result<()> {
        // TODO: that clone
        for (key, _) in self.db.clone().iter() {
            self._remove(&key)?;
        }

        Ok(())
    }

    /// `_remove_range` removes a range of items from the `BTreeStore`.
    fn _remove_range(
        &mut self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        skip: Option<u32>,
    ) -> Result<()> {
        if let Some(from) = from {
            if let Some(to) = to {
                if from > to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }

                if let Some(skip) = skip {
                    self._remove_range_complete(from, to, skip)
                } else {
                    self._remove_range_no_skip(from, to)
                }
            } else if let Some(skip) = skip {
                self._remove_range_no_to(from, skip)
            } else {
                self._remove_range_no_to_no_skip(from)
            }
        } else if let Some(to) = to {
            if let Some(skip) = skip {
                self._remove_range_no_from(to, skip)
            } else {
                self._remove_range_no_from_no_skip(to)
            }
        } else if let Some(skip) = skip {
            self._remove_range_no_from_no_to(skip)
        } else {
            self._remove_range_none()
        }
    }

    /// `_clear` clears the `BTreeStore`.
    fn _clear(&mut self) {
        self.db.clear();
        self.keys_size = 0;
        self.values_size = 0;
    }
}

impl Store for BTreeStore {
    fn keys_size(&self) -> u32 {
        self.keys_size
    }

    fn values_size(&self) -> u32 {
        self.values_size
    }

    fn size(&self) -> u32 {
        self.keys_size + self.values_size
    }

    fn set_max_value_size(&mut self, size: u32) {
        self.max_value_size = size
    }

    fn get_max_value_size(&self) -> u32 {
        self.max_value_size
    }

    fn set_max_size(&mut self, size: u32) -> Result<()> {
        if size < self.get_max_value_size() {
            let err = Error::InvalidSize;
            return Err(err);
        }

        self.max_size = size;

        Ok(())
    }

    fn get_max_size(&self) -> u32 {
        self.max_size
    }

    fn lookup(&self, key: &[u8]) -> Result<bool> {
        Ok(self._lookup(key))
    }

    fn get(&self, key: &[u8]) -> Result<Vec<u8>> {
        self._get(key)
    }

    fn query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Vec<u8>>> {
        self._query(from, to, count, skip)
    }

    fn sample(&self, from: Option<&[u8]>, to: Option<&[u8]>, count: u32) -> Result<Vec<Vec<u8>>> {
        self._sample(from, to, count)
    }

    fn count(&self, from: Option<&[u8]>, to: Option<&[u8]>, skip: Option<u32>) -> Result<u32> {
        self._count(from, to, skip)
    }

    fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self._insert(key, value)
    }

    fn create(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self._create(key, value)
    }

    fn update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self._update(key, value)
    }

    fn insert_batch(&mut self, _items: &[(&[u8], &[u8])]) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn remove(&mut self, key: &[u8]) -> Result<()> {
        self._remove(key)
    }

    fn remove_batch(&mut self, _keys: &[&[u8]]) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn remove_range(
        &mut self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        skip: Option<u32>,
    ) -> Result<()> {
        self._remove_range(from, to, skip)
    }

    fn clear(&mut self) -> Result<()> {
        self._clear();
        Ok(())
    }
}

impl MemoryStore for BTreeStore {}

#[test]
fn test_btree_store_ops() {
    use crypto::random::Random;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let res = BTreeStore::new(max_size, max_value_size);
    assert!(res.is_err());

    let res = BTreeStore::new(max_value_size, max_size);
    assert!(res.is_ok());

    let mut store = res.unwrap();

    let key_len = 100;
    let value_len = 1000;
    let mut expected_size = 0;

    let items: Vec<(Vec<u8>, Vec<u8>)> = (0..10)
        .map(|_| {
            (
                Random::bytes(key_len).unwrap(),
                Random::bytes(value_len).unwrap(),
            )
        })
        .collect();

    for (key, value) in &items {
        let size = store.size();
        assert_eq!(size, expected_size);

        let res = store.count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store.lookup(&key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = store.get(&key);
        assert!(res.is_err());

        let res = store.insert(&key, &value);
        assert!(res.is_ok());

        expected_size += (key.len() + value.len()) as u32;

        let res = store.count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = store.query(Some(&key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);

        let res = store.lookup(&key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = store.get(&key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = store.remove(&key);
        assert!(res.is_ok());

        expected_size -= (key.len() + value.len()) as u32;

        let res = store.count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store.query(Some(&key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![] as Vec<Vec<u8>>);

        let res = store.lookup(&key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = store.get(&key);
        assert!(res.is_err());

        let res = store.insert(&key, &value);
        assert!(res.is_ok());

        let res = store.clear();
        assert!(res.is_ok());

        assert_eq!(store.keys_size(), 0);
        assert_eq!(store.values_size(), 0);
    }

    let invalid_value_len = max_value_size + 1;

    let invalid_item = (
        Random::bytes(key_len).unwrap(),
        Random::bytes(invalid_value_len as usize).unwrap(),
    );

    let res = store.insert(&invalid_item.0, &invalid_item.1);
    assert!(res.is_err());

    store.set_max_value_size(invalid_value_len as u32);

    let res = store.insert(&invalid_item.0, &invalid_item.1);
    assert!(res.is_ok());

    let res = store.lookup(&invalid_item.0);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(found);
}
