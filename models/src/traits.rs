//! # Traits
//!
//! `traits` contains traits used throughout the crate.

use crate::result::Result;
use store::traits::Store;

/// `Storable` is the trait implemented by storable models.
pub trait Storable<S: Store>: Sized {
    /// `KEY_PREFIX` is the prefix added to keys of this model.
    const KEY_PREFIX: u32;

    /// `Key` is the type used to identify the model's instances in the `Store`.
    type Key;

    /// `lookup` looks up a model instance in the `Store` by key.
    fn lookup(&self, store: &S, key: &Self::Key) -> Result<bool>;

    /// `get` returns a model instance from the `Store`.
    fn get(&self, store: &S, key: &Self::Key) -> Result<Self>;

    // TODO: de-lame query: use streams
    /// `query` queries the `Store` for model instances.
    fn query(
        &self,
        store: &S,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Self>>;

    /// `count` counts `Store` model instances matching a specific query.
    fn count(
        &self,
        store: &S,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32>;

    /// `insert` inserts a model instance in the `Store`.
    fn insert(&mut self, store: &mut S, key: &Self::Key, value: &Self) -> Result<()>;

    /// `create` creates a previously not existing model instance in the `Store`.
    fn create(&mut self, store: &mut S, key: &Self::Key, value: &Self) -> Result<()>;

    /// `update` updates a previously existing model instance in the `Store`.
    fn update(&mut self, store: &mut S, key: &Self::Key, value: &Self) -> Result<()>;

    /// `insert_batch` inserts one or more model instances in the `Store`.
    fn insert_batch(&mut self, store: &mut S, items: &[(Self::Key, Self)]) -> Result<()>;

    /// `remove` removes a mode instance from the `Store`.
    fn remove(&mut self, store: &mut S, key: &Self::Key) -> Result<()>;

    /// `remove_batch` removes one or more model instances from the `Store`.
    fn remove_batch(&mut self, store: &mut S, keys: &[Self::Key]) -> Result<()>;

    /// `cleanup` clean ups the `Store` model instances.
    fn cleanup(&mut self, store: &mut S) -> Result<()>;

    /// `clear` clears the `Store` from the model instances.
    fn clear(&mut self, store: &mut S) -> Result<()>;
}
