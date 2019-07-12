//! `traits` contains Alsacoin's storage traits.

use futures::Future;
use futures::Stream;

/// `Store` is the trait implemented by `Alsacoin` stores.
pub trait Store {
    /// The type of a `Store` key.
    type Key;
    /// The type of a `Store` value.
    type Value;
    /// The type of the `Store` `query` method parameters.
    type QueryParams;
    /// The type of the `Store` `exec` method parameters.
    type ExecParams;
    /// The type of the `Store` `exec` method results.
    type ExecResult;

    /// `get` returns a `Store` value by key.
    fn get(&self, k: &Self::Key) -> dyn Future<Output = Self::Value>;

    /// `query` queries the `Store` for values.
    fn query(&self, p: &Self::QueryParams) -> dyn Stream<Item = Self::Value>;

    /// `count` counts `Store` items matching a specific query.
    fn count(&self, p: &Self::QueryParams) -> dyn Stream<Item = u32>;

    /// `insert` inserts an item in the `Store`.
    fn insert(&mut self, k: &Self::Key, v: &Self::Value) -> dyn Future<Output = ()>;

    /// `insert_batch` inserts one or more items in the `Store`.
    fn insert_batch(&mut self, kvs: &[(Self::Key, Self::Value)]) -> dyn Future<Output = ()>;

    /// `remove` removes an item from the `Store`.
    fn remove(&mut self, k: &Self::Key) -> dyn Future<Output = ()>;

    /// `remove_batch` removes one or more items from the `Store`.
    fn remove_batch(&mut self, k: &[Self::Key]) -> dyn Future<Output = ()>;

    /// `exec` execs an operation in the `Store`.
    fn exec(&mut self, p: &Self::ExecParams) -> dyn Future<Output = Self::ExecResult>;
}
