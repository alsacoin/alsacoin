//! # Traits
//!
//! `traits` contains traits used throughout the crate.

use crate::result::Result;
//use crate::error::Error;
use futures::future::BoxFuture;
use store::traits::Store;

/// `Storable` is the trait implemented by storable models.
pub trait Storable<S: Store>: Sized {
    /// `KEY_PREFIX` is the prefix added to keys of this model.
    const KEY_PREFIX: u32;

    /// `Key` is the type used to identify the model's instances in the `Store`.
    type Key;

    /// `store_key` returns the `Store` `Key` of the `Storable` instance.
    fn store_key(&self) -> Result<S::Key>;

    /// `store_value` returns the `Store` `Value` of the `Storable` instance.
    fn store_value(&self) -> Result<S::Value>;

    /// `key_from_store_key` returns a `Key` from a `Store` `Key`.
    fn key_from_store_key(key: S::Key) -> Result<Self::Key>;

    /// `from_store_value` returns a model instance from a `Store` `Value`.
    fn from_store_value(value: S::Value) -> Result<Self>;

    /// `lookup` looks up a model instance in the `Store` by key.
    fn lookup(&self, key: &Self::Key) -> BoxFuture<Result<bool>>;

    /// `get` returns a model instance from the `Store`.
    fn get(&self, key: &Self::Key) -> BoxFuture<Result<Self>>;

    // TODO: de-lame query: use streams
    /// `query` queries the `Store` for model instances.
    fn query(
        &self,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<Vec<Self>>>;

    /// `count` counts `Store` model instances matching a specific query.
    fn count(
        &self,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<u32>>;

    /// `insert` inserts a model instance in the `Store`.
    fn insert(&mut self, key: &Self::Key, value: &Self) -> BoxFuture<Result<()>>;

    /// `create` creates a previously not existing model instance in the `Store`.
    fn create(&mut self, key: &Self::Key, value: &Self) -> BoxFuture<Result<()>>;

    /// `update` updates a previously existing model instance in the `Store`.
    fn update(&mut self, key: &Self::Key, value: &Self) -> BoxFuture<Result<()>>;

    /// `insert_batch` inserts one or more model instances in the `Store`.
    fn insert_batch(&mut self, items: &[(Self::Key, Self)]) -> BoxFuture<Result<()>>;

    /// `remove` removes a mode instance from the `Store`.
    fn remove(&mut self, key: &Self::Key) -> BoxFuture<Result<()>>;

    /// `remove_batch` removes one or more model instances from the `Store`.
    fn remove_batch(&mut self, keys: &[Self::Key]) -> BoxFuture<Result<()>>;

    /// `cleanup` clean ups the `Store` model instances.
    fn cleanup(&mut self) -> BoxFuture<Result<()>>;

    /// `clear` clears the `Store` from the model instances.
    fn clear(&mut self) -> BoxFuture<Result<()>>;
}
