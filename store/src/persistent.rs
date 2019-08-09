//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::result::Result;
use crate::traits::Store;
use futures::future::BoxFuture;
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

    fn lookup(&self, _key: &Self::Key) -> BoxFuture<Result<bool>> {
        // TODO
        unreachable!()
    }

    fn get(&self, _key: &Self::Key) -> BoxFuture<Result<Self::Value>> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _from: &Self::Key,
        _to: &Self::Key,
        _count: u32,
        _skip: u32,
    ) -> BoxFuture<Result<Vec<Self::Value>>> {
        // TODO
        unreachable!()
    }

    fn count(&self, _from: &Self::Key, _to: &Self::Key, _skip: u32) -> BoxFuture<Result<u32>> {
        // TODO
        unreachable!()
    }

    fn insert(&mut self, _key: &Self::Key, _value: &Self::Value) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }

    fn insert_batch(&mut self, _items: &[(Self::Key, Self::Value)]) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _key: &Self::Key) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }
}
