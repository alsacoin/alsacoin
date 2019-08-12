//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::backend::UnQLiteStore;
use crate::result::Result;

/// `PersistentStore` is a factory for temporary stores.
pub struct PersistentStore;

impl PersistentStore {
    /// `new_unqlite` creates a new persistent `UnQLiteStore`.
    pub fn new_unqlite(path: &str) -> Result<UnQLiteStore> {
        UnQLiteStore::new_persistent(path)
    }
}
