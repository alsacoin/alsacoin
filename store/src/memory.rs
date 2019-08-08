//! # Memory
//
// `memory` contains the memory store type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use futures::{future, stream, TryFuture, TryStream};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
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

    fn lookup(&self, key: &Self::Key) -> Box<dyn TryFuture<Ok = bool, Error = Error>> {
        let res = self.db.contains_key(key);
        Box::new(future::ok(res))
    }

    fn get(&self, key: &Self::Key) -> Box<dyn TryFuture<Ok = Self::Value, Error = Error>> {
        match self.db.get(key) {
            Some(value) => Box::new(future::ok(value.to_owned())),
            None => {
                let err = Error::NotFound;
                Box::new(future::err(err))
            }
        }
    }

    fn query(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        count: u32,
        skip: u32,
    ) -> Box<dyn TryStream<Ok = Self::Value, Error = Error>> {
        let res: Vec<Result<Self::Value>> = self
            .db
            .iter()
            .filter(|(k, _)| (from <= k) && (to > k))
            .skip(skip as usize)
            .take(count as usize)
            .map(|(_, v)| Ok(v.to_owned()))
            .collect();

        // TODO: de-lame

        Box::new(stream::iter(res))
    }

    fn count(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        skip: u32,
    ) -> Box<dyn TryFuture<Ok = u32, Error = Error>> {
        let res = self
            .db
            .iter()
            .filter(|(k, _)| (from <= k) && (to > k))
            .skip(skip as usize)
            .count();

        Box::new(future::ok(res as u32))
    }

    fn insert(
        &mut self,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        self.db.insert(key.to_owned(), value.to_owned());
        self.keys_size += key.len() as u32;
        self.values_size += value.len() as u32;
        Box::new(future::ok(()))
    }

    fn insert_batch(
        &mut self,
        _items: &[(Self::Key, Self::Value)],
    ) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        let err = Error::NotImplemented;
        Box::new(future::err(err))
    }

    fn remove(&mut self, key: &Self::Key) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        match self.db.remove(key) {
            Some(value) => {
                self.keys_size -= key.len() as u32;
                self.values_size -= value.len() as u32;
                Box::new(future::ok(()))
            }
            None => {
                let err = Error::NotFound;
                Box::new(future::err(err))
            }
        }
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        let err = Error::NotImplemented;
        Box::new(future::err(err))
    }
}
