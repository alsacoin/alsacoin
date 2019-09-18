//! # Traits
//!
//! `traits` contains traits used throughout the crate.

use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use std::collections::BTreeSet;
use store::traits::Store;

/// `Storable` is the trait implemented by storable models.
pub trait Storable<S: Store>: Sized {
    /// `KEY_PREFIX` is the prefix added to keys of this model.
    const KEY_PREFIX: u8;

    /// `Key` is the type used to identify the model's instances in the `Store`.
    type Key: Default;

    /// `key` returns the model instance key.
    fn key(&self) -> Self::Key;

    /// `key_to_bytes` converts a key to a binary vector.
    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>>;

    /// `validate_single` validates the model instance against the store.
    fn validate_single(store: &S, stage: Stage, value: &Self) -> Result<()>;

    /// `validate_all` validates all the models instances in the store.
    fn validate_all(store: &S, stage: Stage) -> Result<()>;

    /// `lookup` looks up a model instance in the `Store` by key.
    fn lookup(store: &S, stage: Stage, key: &Self::Key) -> Result<bool>;

    /// `get` returns a model instance from the `Store`.
    fn get(store: &S, stage: Stage, key: &Self::Key) -> Result<Self>;

    /// `query` queries the `Store` for model instances.
    fn query(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<BTreeSet<Self>>;

    /// `sample` samples model instances from the `Store`.
    fn sample(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: u32,
    ) -> Result<BTreeSet<Self>>;

    /// `count` counts `Store` model instances matching a specific query.
    fn count(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32>;

    /// `insert` inserts a model instance in the `Store`.
    fn insert(store: &mut S, stage: Stage, value: &Self) -> Result<()>;

    /// `create` creates a previously not existing model instance in the `Store`.
    fn create(store: &mut S, stage: Stage, value: &Self) -> Result<()>;

    /// `update` updates a previously existing model instance in the `Store`.
    fn update(store: &mut S, stage: Stage, value: &Self) -> Result<()>;

    /// `insert_batch` inserts one or more model instances in the `Store`.
    fn insert_batch(store: &mut S, stage: Stage, values: &BTreeSet<Self>) -> Result<()>;

    /// `remove` removes a mode instance from the `Store`.
    fn remove(store: &mut S, stage: Stage, key: &Self::Key) -> Result<()>;

    /// `remove_batch` removes one or more model instances from the `Store`.
    fn remove_batch(store: &mut S, stage: Stage, keys: &BTreeSet<Self::Key>) -> Result<()>;

    /// `cleanup` clean ups the `Store` model instances.
    fn cleanup(store: &mut S, stage: Stage, min_time: Option<Timestamp>) -> Result<()>;

    /// `clear` clears the `Store` from the model instances.
    fn clear(store: &mut S, stage: Stage) -> Result<()>;
}
