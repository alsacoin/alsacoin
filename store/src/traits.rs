//! # Traits
//!
//! `traits` contains Alsacoin's storage traits.

use crate::result::Result;

/// `Store` is the trait implemented by `Alsacoin` stores.
pub trait Store {
    /// `keys_size` returns the size of the store keys.
    fn keys_size(&self) -> u32;

    /// `values_size` returns the size of the store values.
    fn values_size(&self) -> u32;

    /// `size` returns the size of the store items.
    fn size(&self) -> u32;

    /// `set_max_value_size` sets the maximum size of a store value.
    fn set_max_value_size(&mut self, size: u32);

    /// `max_value_size` gets the maximum size of a store value.
    fn get_max_value_size(&self) -> u32;

    /// `set_max_size` sets the maximum size of a store size.
    fn set_max_size(&mut self, size: u32) -> Result<()>;

    /// `max_size` gets the maximum size of a store size.
    fn get_max_size(&self) -> u32;

    /// `lookup` looks up a `Store` value by key.
    fn lookup(&self, key: &[u8]) -> Result<bool>;

    /// `get` returns a `Store` value by key.
    fn get(&self, key: &[u8]) -> Result<Vec<u8>>;

    // TODO: de-lame query: use streams
    /// `query` queries the `Store` for values.
    fn query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Vec<u8>>>;

    /// `sample` samples `Store` values.
    fn sample(&self, from: Option<&[u8]>, to: Option<&[u8]>, count: u32) -> Result<Vec<Vec<u8>>>;

    /// `count` counts `Store` items matching a specific query.
    fn count(&self, from: Option<&[u8]>, to: Option<&[u8]>, skip: Option<u32>) -> Result<u32>;

    /// `insert` inserts an item in the `Store`.
    fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<()>;

    /// `create` creates a previously not existing item in the `Store`.
    fn create(&mut self, key: &[u8], value: &[u8]) -> Result<()>;

    /// `update` updates a previously existing item in the `Store`.
    fn update(&mut self, key: &[u8], value: &[u8]) -> Result<()>;

    /// `insert_batch` inserts one or more items in the `Store`.
    fn insert_batch(&mut self, items: &[(&[u8], &[u8])]) -> Result<()>;

    /// `remove` removes an item from the `Store`.
    fn remove(&mut self, key: &[u8]) -> Result<()>;

    /// `remove_range` removes a range of items from the `Store`.
    fn remove_range(
        &mut self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        skip: Option<u32>,
    ) -> Result<()>;

    /// `remove_batch` removes one or more items from the `Store`.
    fn remove_batch(&mut self, keys: &[&[u8]]) -> Result<()>;

    /// `clear` clears the `Store`.
    fn clear(&mut self) -> Result<()>;
}

/// `MemoryStore` is the trait implemented by in-memory `Store`s.
pub trait MemoryStore: Store {}

/// `TemporaryStore` is the trait implemented by temporary `Store`s.
pub trait TemporaryStore: Store {}

/// `PersistentStore` is the trait implemented by persistent `Store`s.
pub trait PersistentStore: Store {}
