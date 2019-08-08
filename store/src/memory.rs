//! # Memory
//
// `memory` contains the memory store type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use futures::future::{self, BoxFuture};
use futures::stream::BoxStream;
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
        let res = self.db.contains_key(key);
        Box::pin(future::ok(res))
    }

    fn get(&self, key: &Self::Key) -> BoxFuture<Result<Self::Value>> {
        match self.db.get(key) {
            Some(value) => Box::pin(future::ok(value.to_owned())),
            None => {
                let err = Error::NotFound;
                Box::pin(future::err(err))
            }
        }
    }

    fn query(
        &self,
        _from: &Self::Key,
        _to: &Self::Key,
        _count: u32,
        _skip: u32,
    ) -> BoxFuture<Result<BoxStream<Self::Value>>> {
        unreachable!()
        /*
        let res: Vec<Self::Value> = self
            .db
            .iter()
            .filter(|(k, _)| (from <= k) && (to > k))
            .skip(skip as usize)
            .take(count as usize)
            .map(|(_, v)| v.to_owned())
            .collect();

        // TODO: de-lame

        Box::pin(future::ok(Box::pin(stream::iter(res))))
        */
    }

    fn count(&self, from: &Self::Key, to: &Self::Key, skip: u32) -> BoxFuture<Result<u32>> {
        let res = self
            .db
            .iter()
            .filter(|(k, _)| (from <= k) && (to > k))
            .skip(skip as usize)
            .count();

        Box::pin(future::ok(res as u32))
    }

    fn insert(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>> {
        self.db.insert(key.to_owned(), value.to_owned());
        self.keys_size += key.len() as u32;
        self.values_size += value.len() as u32;
        Box::pin(future::ok(()))
    }

    fn insert_batch(&mut self, _items: &[(Self::Key, Self::Value)]) -> BoxFuture<Result<()>> {
        let err = Error::NotImplemented;
        Box::pin(future::err(err))
    }

    fn remove(&mut self, key: &Self::Key) -> BoxFuture<Result<()>> {
        match self.db.remove(key) {
            Some(value) => {
                self.keys_size -= key.len() as u32;
                self.values_size -= value.len() as u32;
                Box::pin(future::ok(()))
            }
            None => {
                let err = Error::NotFound;
                Box::pin(future::err(err))
            }
        }
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> BoxFuture<Result<()>> {
        let err = Error::NotImplemented;
        Box::pin(future::err(err))
    }
}

#[test]
fn test_memory_store() {
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

            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 1);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_ok());
            assert_eq!(&res.unwrap(), value);

            let res = store.remove(&key).await;
            assert!(res.is_ok());

            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());
        };
    }
}
