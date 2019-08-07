//! `traits` contains Alsacoin's storage traits.

use futures::Future;
use futures::Stream;

/// `Store` is the trait implemented by `Alsacoin` stores.
pub trait Store {
    /// The type of a `Store` key.
    type Key;
    /// The type of a `Store` value.
    type Value;

    /// `get` returns a `Store` value by key.
    fn get(&self, key: &Self::Key) -> Box<dyn Future<Output = Self::Value>>;

    /// `query` queries the `Store` for values.
    fn query(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        count: u64,
        skip: u64,
    ) -> Box<dyn Stream<Item = Self::Value>>;

    /// `count` counts `Store` items matching a specific query.
    fn count(&self, from: &Self::Key, to: &Self::Key, skip: u64) -> Box<dyn Stream<Item = u64>>;

    /// `insert` inserts an item in the `Store`.
    fn insert(&mut self, key: &Self::Key, value: &Self::Value) -> Box<dyn Future<Output = ()>>;

    /// `insert_batch` inserts one or more items in the `Store`.
    fn insert_batch(&mut self, items: &[(Self::Key, Self::Value)]) -> Box<dyn Future<Output = ()>>;

    /// `remove` removes an item from the `Store`.
    fn remove(&mut self, key: &Self::Key) -> Box<dyn Future<Output = ()>>;

    /// `remove_batch` removes one or more items from the `Store`.
    fn remove_batch(&mut self, keys: &[Self::Key]) -> Box<dyn Future<Output = ()>>;
}
