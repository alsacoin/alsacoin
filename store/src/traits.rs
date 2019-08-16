//! # Traits
//!
//! `traits` contains Alsacoin's storage traits.

use crate::result::Result;
use futures::future::BoxFuture;

/// `Store` is the trait implemented by `Alsacoin` stores.
pub trait Store {
    /// `keys_size` returns the size of the store keys.
    fn keys_size(&self) -> u32;

    /// `values_size` returns the size of the store values.
    fn values_size(&self) -> u32;

    /// `size` returns the size of the store items.
    fn size(&self) -> u32;

    /// `lookup` looks up a `Store` value by key.
    fn lookup(&self, key: &[u8]) -> BoxFuture<Result<bool>>;

    /// `get` returns a `Store` value by key.
    fn get(&self, key: &[u8]) -> BoxFuture<Result<Vec<u8>>>;

    // TODO: de-lame query: use streams
    /// `query` queries the `Store` for values.
    fn query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<Vec<Vec<u8>>>>;

    /// `count` counts `Store` items matching a specific query.
    fn count(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<u32>>;

    /// `insert` inserts an item in the `Store`.
    fn insert(&mut self, key: &[u8], value: &[u8]) -> BoxFuture<Result<()>>;

    /// `create` creates a previously not existing item in the `Store`.
    fn create(&mut self, key: &[u8], value: &[u8]) -> BoxFuture<Result<()>>;

    /// `update` updates a previously existing item in the `Store`.
    fn update(&mut self, key: &[u8], value: &[u8]) -> BoxFuture<Result<()>>;

    /// `insert_batch` inserts one or more items in the `Store`.
    fn insert_batch(&mut self, items: &[(&[u8], &[u8])]) -> BoxFuture<Result<()>>;

    /// `remove` removes an item from the `Store`.
    fn remove(&mut self, key: &[u8]) -> BoxFuture<Result<()>>;

    /// `remove_batch` removes one or more items from the `Store`.
    fn remove_batch(&mut self, keys: &[&[u8]]) -> BoxFuture<Result<()>>;

    /// `clear` clears the `Store`.
    fn clear(&mut self) -> BoxFuture<Result<()>>;
}
