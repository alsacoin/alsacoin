//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use futures::future::BoxFuture;
use rkv::{Manager, Rkv, SingleStore, StoreOptions, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct PersistentStore {
    path: PathBuf,
    manager: Arc<RwLock<Rkv>>,
    keys_size: u32,
    values_size: u32,
}

impl PersistentStore {
    /// `new` creates a new `PersistentStore`.
    pub fn new(path: &Path) -> Result<PersistentStore> {
        let manager = Manager::singleton()
            .write()
            .unwrap()
            .get_or_create(path, Rkv::new)
            .unwrap();

        let mut store = PersistentStore {
            path: path.into(),
            manager,
            keys_size: 0,
            values_size: 0,
        };

        store.update_size()?;

        Ok(store)
    }

    /// `open` returns a `PersistentStore` store handle.
    fn open(&self, name: &str) -> Result<SingleStore> {
        let env = self.manager.read()?;
        env.open_single(name, StoreOptions::create())
            .map_err(|e| e.into())
    }

    /// `path` returns the `PersistentStore` path.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// `update_size` udpates the `PersistentStore` cached sizes.
    pub fn update_size(&mut self) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `get` gets a key-value pair from the `PersistentStore`.
    pub fn get(&self, name: &str, key: &[u8]) -> Result<Vec<u8>> {
        let env = self.manager.read()?;
        let reader = env.read()?;
        if let Some(value) = self.open(name)?.get(&reader, key)? {
            value.to_bytes().map_err(|e| e.into())
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `iter` returns the `PersistentStore` iterator over the key-value pairs.
    pub fn iter(
        &self,
        name: &str,
        from: &[u8],
        to: &[u8],
        count: u32,
        skip: u32,
    ) -> Result<Vec<Vec<u8>>> {
        let env = self.manager.read()?;
        let reader = env.read()?;
        let mut values = Vec::new();
        let mut skipped = 0;
        let mut counted = 0;

        for res in self.open(name)?.iter_start(&reader)? {
            let (k, v) = res?;

            if (from <= k) && (to > k) {
                if skipped >= skip {
                    if counted <= count {
                        if let Some(value) = v {
                            values.push(value.to_bytes()?);
                            counted += 1;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                skipped += 1;
            }
        }

        Ok(values)
    }

    /// `put` puts a binary key-value pair in the `PersistentStore`.
    pub fn put(&mut self, name: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let env = self.manager.read()?;
        let mut writer = env.write()?;
        self.open(name)?
            .put(&mut writer, key, &Value::Blob(value))?;
        writer.commit().map_err(|e| e.into())
    }

    /// `delete` deletes a key-value pair from the `PersistentStore`.
    pub fn delete(&mut self, name: &str, key: &[u8]) -> Result<()> {
        let env = self.manager.read()?;
        let mut writer = env.write()?;
        self.open(name)?.delete(&mut writer, key)?;
        writer.commit().map_err(|e| e.into())
    }

    /// `clear` clears the `PersistentStore`.
    pub fn clear(&mut self, name: &str) -> Result<()> {
        let env = self.manager.read()?;
        let mut writer = env.write()?;
        self.open(name)?.clear(&mut writer)?;
        writer.commit().map_err(|e| e.into())
    }
}

impl Store for PersistentStore {
    type Key = Vec<u8>;
    type Value = Vec<u8>;

    fn keys_size(&self) -> u32 {
        self.keys_size
    }

    fn values_size(&self) -> u32 {
        self.values_size
    }

    fn size(&self) -> u32 {
        self.keys_size + self.values_size
    }

    fn lookup(&self, _key: &Self::Key) -> BoxFuture<Result<bool>> {
        // TODO
        unreachable!()
    }

    fn get(&self, _key: &Self::Key) -> BoxFuture<Result<Self::Value>> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _from: &Self::Key,
        _to: &Self::Key,
        _count: u32,
        _skip: u32,
    ) -> BoxFuture<Result<Vec<Self::Value>>> {
        // TODO
        unreachable!()
    }

    fn count(&self, _from: &Self::Key, _to: &Self::Key, _skip: u32) -> BoxFuture<Result<u32>> {
        // TODO
        unreachable!()
    }

    fn insert(&mut self, _key: &Self::Key, _value: &Self::Value) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }

    fn insert_batch(&mut self, _items: &[(Self::Key, Self::Value)]) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _key: &Self::Key) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> BoxFuture<Result<()>> {
        // TODO
        unreachable!()
    }
}
