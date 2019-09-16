//! # Store
//!
//! `store` is the module containing the store type and functions.

use crate::error::Error;
use crate::persistent::PersistentStoreFactory;
use crate::result::Result;
use crate::temporary::TemporaryStoreFactory;
use crate::traits::Store;
use config::store::StoreConfig;

/// `StoreFactory` is the factory for store types.
pub struct StoreFactory {}

impl StoreFactory {
    /// `create` creates a new store from the configs.
    pub fn create(path: Option<String>, config: &StoreConfig) -> Result<Box<dyn Store>> {
        config.validate()?;

        let mut config = config.clone();
        config.populate();

        match config.kind.unwrap().as_str() {
            "temporary" => {
                let store = TemporaryStoreFactory::new_unqlite(
                    config.max_value_size.unwrap(),
                    config.max_size.unwrap(),
                )?;

                Ok(Box::new(store))
            }
            "persistent" => {
                if path.is_none() {
                    let err = Error::InvalidPath;
                    return Err(err);
                }

                let path = path.unwrap();

                let store = PersistentStoreFactory::new_unqlite(
                    &path,
                    config.max_value_size.unwrap(),
                    config.max_size.unwrap(),
                )?;

                Ok(Box::new(store))
            }
            _ => {
                let err = Error::InvalidKind;
                Err(err)
            }
        }
    }
}
