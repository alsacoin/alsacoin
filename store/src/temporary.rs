//! # Temporary
//
// `temporary` contains the temporary store type and functions.

use crate::backend::UnQLiteStore;
use crate::result::Result;

/// `TemporaryStore` is a factory for temporary stores.
pub struct TemporaryStore;

impl TemporaryStore {
    /// `new_unqlite` creates a new in-memory `UnQLiteStore`.
    pub fn new_unqlite() -> Result<UnQLiteStore> {
        UnQLiteStore::new_temporary()
    }
}
