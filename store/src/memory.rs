//! # Memory
//
// `memory` contains the memory store type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use futures::future::{self, BoxFuture};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct MemoryStore {
    db: BTreeMap<Vec<u8>, Vec<u8>>,
    keys_size: u32,
    values_size: u32,
}

impl MemoryStore {
    /// `new` creates a new `MemoryStore`.
    pub fn new() -> MemoryStore {
        MemoryStore::default()
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
    fn _query(&self, from: &[u8], to: &[u8], count: u32, skip: u32) -> Result<Vec<Vec<u8>>> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let res: Vec<Vec<u8>> = self
            .db
            .iter()
            .filter(|(k, _)| (from <= k) && (to > k))
            .skip(skip as usize)
            .take(count as usize)
            .map(|(_, v)| v.to_owned())
            .collect();

        Ok(res)
    }

    /// `_count` returns the count of a list of values from the `PersistentStore`.
    fn _count(&self, from: &[u8], to: &[u8], skip: u32) -> Result<u32> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let count = self
            .db
            .iter()
            .filter(|(k, _)| (from <= k) && (to > k))
            .skip(skip as usize)
            .count();

        Ok(count as u32)
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

    /// `clear` clears the `MemoryStore`.
    pub fn clear(&mut self) {
        self.db.clear()
    }
}

impl Store for MemoryStore {
    type Key = Vec<u8>;
    type Value = Vec<u8>;

    fn keys_size(&self) -> u32 {
        self.keys_size
    }

    fn values_size(&self) -> u32 {
        self.values_size
    }

    fn size(&self) -> u32 {
        self.keys_size + self.values_size
    }

    fn lookup(&self, key: &Self::Key) -> BoxFuture<Result<bool>> {
        let res = self._lookup(key);
        Box::pin(future::ok(res))
    }

    fn get(&self, key: &Self::Key) -> BoxFuture<Result<Self::Value>> {
        let res = self._get(key);
        Box::pin(future::ready(res))
    }

    fn query(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        count: u32,
        skip: u32,
    ) -> BoxFuture<Result<Vec<Self::Value>>> {
        let res = self._query(from, to, count, skip);
        Box::pin(future::ready(res))
    }

    fn count(&self, from: &Self::Key, to: &Self::Key, skip: u32) -> BoxFuture<Result<u32>> {
        let res = self._count(from, to, skip);
        Box::pin(future::ready(res))
    }

    fn insert(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>> {
        let res = self._insert(key, value);
        Box::pin(future::ready(res))
    }

    fn insert_batch(&mut self, _items: &[(Self::Key, Self::Value)]) -> BoxFuture<Result<()>> {
        let err = Error::NotImplemented;
        Box::pin(future::err(err))
    }

    fn remove(&mut self, key: &Self::Key) -> BoxFuture<Result<()>> {
        let res = self._remove(key);
        Box::pin(future::ready(res))
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> BoxFuture<Result<()>> {
        let err = Error::NotImplemented;
        Box::pin(future::err(err))
    }
}

#[test]
fn test_memory_store_sync_ops() {
    use crypto::random::Random;

    let mut store = MemoryStore::new();
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

        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let found = store._lookup(&key);
        assert!(!found);

        let res = store._get(&key);
        assert!(res.is_err());

        let res = store._insert(&key, &value);
        assert!(res.is_ok());

        expected_size += (key.len() + value.len()) as u32;

        /*
        let res = store.count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = store.query(&key, &key, 0, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);
        */

        let found = store._lookup(&key);
        assert!(found);

        let res = store._get(&key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = store._remove(&key);
        assert!(res.is_ok());

        expected_size -= (key.len() + value.len()) as u32;

        /*
        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store._query(&key, &key, 0, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);
        */

        let found = store._lookup(&key);
        assert!(!found);

        let res = store._get(&key);
        assert!(res.is_err());
    }
}

#[test]
fn test_memory_store_async_ops() {
    use crypto::random::Random;
    use std::sync::{Arc, Mutex};

    let store = Arc::new(Mutex::new(MemoryStore::new()));
    let key_len = 100;
    let value_len = 1000;
    let expected_size = Arc::new(Mutex::new(0));;

    let items: Vec<(Vec<u8>, Vec<u8>)> = (0..10)
        .map(|_| {
            (
                Random::bytes(key_len).unwrap(),
                Random::bytes(value_len).unwrap(),
            )
        })
        .collect();

    for (key, value) in &items {
        let store = Arc::clone(&store);
        let expected_size = Arc::clone(&expected_size);

        let _ = async move {
            let mut store = store.lock().unwrap();
            let mut expected_size = expected_size.lock().unwrap();

            let size = store.size();
            assert_eq!(size, *expected_size);

            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());

            let res = store.insert_batch(&[]).await;
            assert!(res.is_err());

            let res = store.insert(&key, &value).await;
            assert!(res.is_ok());

            *expected_size += (key.len() + value.len()) as u32;

            /*
            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 1);

            let res = store.query(&key, &key, 0, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap().len(), 0);
            */

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_ok());
            assert_eq!(&res.unwrap(), value);

            let res = store.remove(&key).await;
            assert!(res.is_ok());

            *expected_size -= (key.len() + value.len()) as u32;

            /*
            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.query(&key, &key, 0, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), vec![value.to_owned()]);
            */

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());
        };
    }
}
