//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::error::Error;
use crate::traits::Store;
use futures::{TryFuture, TryStream};
use rkv::SingleStore;

pub struct PersistentStore {
    _db: SingleStore,
    keys_size: u32,
    values_size: u32,
}

impl Store for PersistentStore {
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

    fn lookup(&self, _key: &Self::Key) -> Box<dyn TryFuture<Ok = bool, Error = Error>> {
        // TODO
        unreachable!()
    }

    fn get(&self, _key: &Self::Key) -> Box<dyn TryFuture<Ok = Self::Value, Error = Error>> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _from: &Self::Key,
        _to: &Self::Key,
        _count: u32,
        _skip: u32,
    ) -> Box<dyn TryStream<Ok = Self::Value, Error = Error>> {
        // TODO
        unreachable!()
    }

    fn count(
        &self,
        _from: &Self::Key,
        _to: &Self::Key,
        _skip: u32,
    ) -> Box<dyn TryFuture<Ok = u32, Error = Error>> {
        // TODO
        unreachable!()
    }

    fn insert(
        &mut self,
        _key: &Self::Key,
        _value: &Self::Value,
    ) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        // TODO
        unreachable!()
    }

    fn insert_batch(
        &mut self,
        _items: &[(Self::Key, Self::Value)],
    ) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _key: &Self::Key) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> Box<dyn TryFuture<Ok = (), Error = Error>> {
        // TODO
        unreachable!()
    }
}
