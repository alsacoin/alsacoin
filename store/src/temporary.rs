//! # Temporary
//
// `temporary` contains the temporary store type and functions.

use crate::backend::UnQLiteStore;
use crate::result::Result;

/// `TemporaryStoreFactory` is a factory for temporary stores.
pub struct TemporaryStoreFactory;

impl TemporaryStoreFactory {
    /// `new_unqlite` creates a new in-memory `UnQLiteStore`.
    pub fn new_unqlite(max_value_size: u32) -> Result<UnQLiteStore> {
        UnQLiteStore::new_temporary(max_value_size)
    }
}
