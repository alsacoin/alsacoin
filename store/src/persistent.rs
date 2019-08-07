//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::traits::Store;
use futures::Future;
use futures::Stream;
use rkv::SingleStore;

pub struct PersistentStore(SingleStore);

impl Store for PersistentStore {
    type Key = Vec<u8>;
    type Value = Vec<u8>;

    fn get(&self, _key: &Self::Key) -> Box<dyn Future<Output = Self::Value>> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _from: &Self::Key,
        _to: &Self::Key,
        _count: u64,
        _skip: u64,
    ) -> Box<dyn Stream<Item = Self::Value>> {
        // TODO
        unreachable!()
    }

    fn count(&self, _from: &Self::Key, _to: &Self::Key, _skip: u64) -> Box<dyn Stream<Item = u64>> {
        // TODO
        unreachable!()
    }

    fn insert(&mut self, _key: &Self::Key, _value: &Self::Value) -> Box<dyn Future<Output = ()>> {
        // TODO
        unreachable!()
    }

    fn insert_batch(
        &mut self,
        _items: &[(Self::Key, Self::Value)],
    ) -> Box<dyn Future<Output = ()>> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _key: &Self::Key) -> Box<dyn Future<Output = ()>> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> Box<dyn Future<Output = ()>> {
        // TODO
        unreachable!()
    }
}
