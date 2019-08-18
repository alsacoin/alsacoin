//! # BTreeMapStore
//
// `btree_map_store` contains the `BTreeMap` store backend type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct BTreeMapStore {
    db: BTreeMap<Vec<u8>, Vec<u8>>,
    keys_size: u32,
    values_size: u32,
}

impl BTreeMapStore {
    /// `new` creates a new `BTreeMapStore`.
    pub fn new() -> BTreeMapStore {
        BTreeMapStore::default()
    }

    /// `_lookup` looks up a key-value pair from the `PersistentStore`.
    fn _lookup(&self, key: &[u8]) -> bool {
        self.db.contains_key(key)
    }

    /// `_get` gets a key-value pair from the `PersistentStore`.
    fn _get(&self, key: &[u8]) -> Result<Vec<u8>> {
        match self.db.get(key) {
            Some(value) => Ok(value.to_owned()),
            None => {
                let err = Error::NotFound;
                Err(err)
            }
        }
    }

    /// `_query` returns a list of values from the `PersistentStore`.
    fn _query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Vec<u8>>> {
        let res: Vec<Vec<u8>> = if let Some(from) = from {
            if let Some(to) = to {
                if from < to {
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

    /// `_count` returns the count of a list of values from the `PersistentStore`.
    fn _count(&self, from: Option<&[u8]>, to: Option<&[u8]>, skip: Option<u32>) -> Result<u32> {
        let res: u32 = if let Some(from) = from {
            if let Some(to) = to {
                if from < to {
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

    /// `_insert` inserts a binary key-value pair in the `PersistentStore`.
    fn _insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.insert(key.to_owned(), value.to_owned());
        self.keys_size += key.len() as u32;
        self.values_size += value.len() as u32;
        Ok(())
    }

    /// `_create` inserts a non-existing binary key-value pair in the `PersistentStore`.
    fn _create(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if self._lookup(key) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_update` updates an existing key-value pair in the `PersistentStore`.
    pub fn _update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_remove` removes a key-value pair from the `PersistentStore`.
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

    /// `_clear` clears the `BTreeMapStore`.
    fn _clear(&mut self) {
        self.db.clear();
        self.keys_size = 0;
        self.values_size = 0;
    }
}

impl Store for BTreeMapStore {
    fn keys_size(&self) -> u32 {
        self.keys_size
    }

    fn values_size(&self) -> u32 {
        self.values_size
    }

    fn size(&self) -> u32 {
        self.keys_size + self.values_size
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

    fn clear(&mut self) -> Result<()> {
        self._clear();
        Ok(())
    }
}

#[test]
fn test_memory_store_ops() {
    use crypto::random::Random;

    let mut store = BTreeMapStore::new();
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
}
