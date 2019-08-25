//! # Memory
//
// `memory` contains the memory store type and functions.

use crate::backend::{BTreeStore, UnQLiteStore};
use crate::result::Result;

/// `MemoryStoreFactory` is a factory for in-memory stores.
pub struct MemoryStoreFactory;

impl MemoryStoreFactory {
    /// `new_btree` creates a new `BTreeStore`.
    pub fn new_btree() -> BTreeStore {
        BTreeStore::new()
    }

    /// `new_unqlite` creates a new in-memory `UnQLiteStore`.
    pub fn new_unqlite() -> Result<UnQLiteStore> {
        UnQLiteStore::new_memory()
    }
}
