//! # Memory
//
// `memory` contains the memory store type and functions.

use crate::backend::{BTreeMapStore, UnQLiteStore};
use crate::result::Result;

/// `MemoryStore` is a factory for in-memory stores.
pub struct MemoryStore;

impl MemoryStore {
    /// `new_btree_map` creates a new `BTreeMapStore`.
    pub fn new_btree_map() -> BTreeMapStore {
        BTreeMapStore::new()
    }

    /// `new_unqlite` creates a new in-memory `UnQLiteStore`.
    pub fn new_unqlite() -> Result<UnQLiteStore> {
        UnQLiteStore::new_memory()
    }
}
