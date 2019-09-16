//! # Pool Factory
//!
//! `pool` is the module containing the PoolFactory type and functions.

use crate::backend::UnQLiteStore;
use crate::memory::MemoryStoreFactory;
use crate::result::Result;
use config::pool::PoolConfig;

/// `PoolFactory` is the factory for store types.
pub struct PoolFactory {}

impl PoolFactory {
    /// `create` creates a new pool from the configs.
    pub fn create(config: &PoolConfig) -> Result<UnQLiteStore> {
        let mut config = config.clone();
        config.populate();

        MemoryStoreFactory::new_unqlite(config.max_value_size.unwrap(), config.max_size.unwrap())
    }
}
