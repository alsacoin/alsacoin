//! `traits` contains Alsacoin's storage traits.

use crate::error::Error;
use futures::{TryFuture, TryStream};

/// `Store` is the trait implemented by `Alsacoin` stores.
pub trait Store {
    /// The type of a `Store` key.
    type Key;
    /// The type of a `Store` value.
    type Value;

    /// `lookup` looks up a `Store` value by key.
    fn lookup(&self, key: &Self::Key) -> Box<dyn TryFuture<Ok = bool, Error = Error>>;

    /// `get` returns a `Store` value by key.
    fn get(&self, key: &Self::Key) -> Box<dyn TryFuture<Ok = Self::Value, Error = Error>>;

    /// `query` queries the `Store` for values.
    fn query(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        count: u32,
        skip: u32,
    ) -> Box<dyn TryStream<Ok = Self::Value, Error = Error>>;

    /// `count` counts `Store` items matching a specific query.
    fn count(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        skip: u32,
    ) -> Box<dyn TryFuture<Ok = u32, Error = Error>>;

    /// `insert` inserts an item in the `Store`.
    fn insert(
        &mut self,
        key: &Self::Key,
        value: &Self::Value,
    ) -> Box<dyn TryFuture<Ok = (), Error = Error>>;

    /// `insert_batch` inserts one or more items in the `Store`.
    fn insert_batch(
        &mut self,
        items: &[(Self::Key, Self::Value)],
    ) -> Box<dyn TryFuture<Ok = (), Error = Error>>;

    /// `remove` removes an item from the `Store`.
    fn remove(&mut self, key: &Self::Key) -> Box<dyn TryFuture<Ok = (), Error = Error>>;

    /// `remove_batch` removes one or more items from the `Store`.
    fn remove_batch(&mut self, keys: &[Self::Key]) -> Box<dyn TryFuture<Ok = (), Error = Error>>;
}
