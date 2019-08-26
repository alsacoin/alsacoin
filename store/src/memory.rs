//! # Memory
//
// `memory` contains the memory store type and functions.

use crate::backend::{BTreeStore, UnQLiteStore};
use crate::result::Result;

/// `MemoryStoreFactory` is a factory for in-memory stores.
pub struct MemoryStoreFactory;

impl MemoryStoreFactory {
    /// `new_btree` creates a new `BTreeStore`.
    pub fn new_btree(max_value_size: u32) -> BTreeStore {
        BTreeStore::new(max_value_size)
    }

    /// `new_unqlite` creates a new in-memory `UnQLiteStore`.
    pub fn new_unqlite(max_value_size: u32) -> Result<UnQLiteStore> {
        UnQLiteStore::new_memory(max_value_size)
    }
}
