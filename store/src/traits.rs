//! # Traits
//!
//! `traits` contains Alsacoin's storage traits.

use crate::result::Result;
use futures::future::BoxFuture;

/// `Store` is the trait implemented by `Alsacoin` stores.
pub trait Store {
    /// The type of a `Store` key.
    type Key;
    /// The type of a `Store` value.
    type Value;

    /// `keys_size` returns the size of the store keys.
    fn keys_size(&self) -> u32;

    /// `values_size` returns the size of the store values.
    fn values_size(&self) -> u32;

    /// `size` returns the size of the store items.
    fn size(&self) -> u32;

    /// `lookup` looks up a `Store` value by key.
    fn lookup(&self, key: &Self::Key) -> BoxFuture<Result<bool>>;

    /// `get` returns a `Store` value by key.
    fn get(&self, key: &Self::Key) -> BoxFuture<Result<Self::Value>>;

    // TODO: de-lame query: use streams
    /// `query` queries the `Store` for values.
    fn query(
        &self,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<Vec<Self::Value>>>;

    /// `count` counts `Store` items matching a specific query.
    fn count(
        &self,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<u32>>;

    /// `insert` inserts an item in the `Store`.
    fn insert(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>>;

    /// `create` creates a previously not existing item in the `Store`.
    fn create(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>>;

    /// `update` updates a previously existing item in the `Store`.
    fn update(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>>;

    /// `insert_batch` inserts one or more items in the `Store`.
    fn insert_batch(&mut self, items: &[(Self::Key, Self::Value)]) -> BoxFuture<Result<()>>;

    /// `remove` removes an item from the `Store`.
    fn remove(&mut self, key: &Self::Key) -> BoxFuture<Result<()>>;

    /// `remove_batch` removes one or more items from the `Store`.
    fn remove_batch(&mut self, keys: &[Self::Key]) -> BoxFuture<Result<()>>;

    /// `clear` clears the `Store`.
    fn clear(&mut self) -> BoxFuture<Result<()>>;
}
