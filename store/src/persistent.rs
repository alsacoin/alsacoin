//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use futures::future::{self, BoxFuture};
use rkv::{Manager, Rkv, SingleStore, StoreOptions, Value};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

pub struct PersistentStore {
    name: String,
    path: PathBuf,
    manager: Arc<RwLock<Rkv>>,
    keys_size: u32,
    values_size: u32,
}

impl PersistentStore {
    /// `new` creates a new `PersistentStore`.
    pub fn new(name: &str, path: &Path) -> Result<PersistentStore> {
        let manager = Manager::singleton()
            .write()
            .unwrap()
            .get_or_create(path, Rkv::new)
            .unwrap();

        let mut store = PersistentStore {
            name: name.into(),
            path: path.into(),
            manager,
            keys_size: 0,
            values_size: 0,
        };

        store.update_size()?;

        Ok(store)
    }

    /// `open` returns a `PersistentStore` store handle.
    fn open(&self) -> Result<SingleStore> {
        let env = self.manager.read()?;
        env.open_single(self.name.as_str(), StoreOptions::create())
            .map_err(|e| e.into())
    }

    /// `path` returns the `PersistentStore` path.
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    /// `update_size` udpates the `PersistentStore` cached sizes.
    pub fn update_size(&mut self) -> Result<()> {
        Ok(()) // TODO
    }

    /// `_lookup` looks up a key-value pair from the `PersistentStore`.
    fn _lookup(&self, key: &[u8]) -> Result<bool> {
        let env = self.manager.read()?;
        let reader = env.read()?;
        let found = self.open()?.get(&reader, key)?.is_some();
        Ok(found)
    }

    /// `_get` gets a key-value pair from the `PersistentStore`.
    fn _get(&self, key: &[u8]) -> Result<Vec<u8>> {
        let env = self.manager.read()?;
        let reader = env.read()?;
        if let Some(value) = self.open()?.get(&reader, key)? {
            value.to_bytes().map_err(|e| e.into())
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `_count` returns the count of a list of values from the `PersistentStore`.
    fn _count(&self, from: &[u8], to: &[u8], skip: u32) -> Result<u32> {
        let env = self.manager.read()?;
        let store = env.open_single(self.name.as_str(), StoreOptions::create())?;
        let reader = env.read()?;
        let mut skipped = 0;
        let mut count = 0;

        let store_iter = store.iter_start(&reader)?;

        for res in store_iter {
            let (k, v) = res?;

            if (from <= k) && (to > k) {
                if skipped >= skip {
                    if v.is_some() {
                        count += 1;
                    } else {
                        break;
                    }
                } else {
                    skipped += 1;
                }
            }
        }

        Ok(count)
    }

    /// `_query` returns a list of values from the `PersistentStore`.
    fn _query(&self, from: &[u8], to: &[u8], count: u32, skip: u32) -> Result<Vec<Vec<u8>>> {
        let env = self.manager.read()?;
        let reader = env.read()?;
        let mut values = Vec::new();
        let mut skipped = 0;
        let mut counted = 0;

        for res in self.open()?.iter_start(&reader)? {
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
                } else {
                    skipped += 1;
                }
            }
        }

        Ok(values)
    }

    /// `_insert` inserts a binary key-value pair in the `PersistentStore`.
    fn _insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        let env = self.manager.read()?;
        let mut writer = env.write()?;
        self.open()?.put(&mut writer, key, &Value::Blob(value))?;
        writer.commit()?;

        self.keys_size += key.len() as u32;
        self.values_size += value.len() as u32;

        Ok(())
    }

    /// `_remove` removes a key-value pair from the `PersistentStore`.
    fn _remove(&mut self, key: &[u8]) -> Result<()> {
        let env = self.manager.read()?;
        let reader = env.read()?;

        if let Some(value) = self.open()?.get(&reader, key)? {
            let value_size = value.to_bytes()?.len();

            let mut writer = env.write()?;
            self.open()?.delete(&mut writer, key)?;
            writer.commit()?;

            self.keys_size -= key.len() as u32;
            self.values_size -= value_size as u32;

            Ok(())
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `clear` clears the `PersistentStore`.
    pub fn clear(&mut self) -> Result<()> {
        let env = self.manager.read()?;
        let mut writer = env.write()?;
        self.open()?.clear(&mut writer)?;
        writer.commit()?;

        self.keys_size = 0;
        self.values_size = 0;

        Ok(())
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

    fn lookup(&self, key: &Self::Key) -> BoxFuture<Result<bool>> {
        let res = self._lookup(key);
        Box::pin(future::ready(res))
    }

    fn get(&self, key: &Self::Key) -> BoxFuture<Result<Self::Value>> {
        let res = self._get(key);
        Box::pin(future::ready(res))
    }

    fn query(
        &self,
        from: &Self::Key,
        to: &Self::Key,
        count: u32,
        skip: u32,
    ) -> BoxFuture<Result<Vec<Self::Value>>> {
        let res = self._query(from, to, count, skip);
        Box::pin(future::ready(res))
    }

    fn count(&self, from: &Self::Key, to: &Self::Key, skip: u32) -> BoxFuture<Result<u32>> {
        let res = self._count(from, to, skip);
        Box::pin(future::ready(res))
    }

    fn insert(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>> {
        let res = self._insert(key, value);
        Box::pin(future::ready(res))
    }

    fn insert_batch(&mut self, _items: &[(Self::Key, Self::Value)]) -> BoxFuture<Result<()>> {
        let err = Error::NotImplemented;
        Box::pin(future::err(err))
    }

    fn remove(&mut self, key: &Self::Key) -> BoxFuture<Result<()>> {
        let res = self._remove(key);
        Box::pin(future::ready(res))
    }

    fn remove_batch(&mut self, _keys: &[Self::Key]) -> BoxFuture<Result<()>> {
        let err = Error::NotImplemented;
        Box::pin(future::err(err))
    }
}

#[test]
fn test_persistent_store_sync_ops() {
    /*
    use crypto::random::Random;
    use tempfile::Builder;

    let name = "test";
    let path_root = Builder::new().prefix("test_db").tempdir().unwrap();
    let path = path_root.path();
    let res = PersistentStore::new(name, &path);
    assert!(res.is_ok());
    let mut store = res.unwrap();

    let key_len = 100;
    let value_len = 1000;
    let mut expected_size = 0;

    let items: Vec<(Vec<u8>, Vec<u8>)> = (0..10)
        .map(|_| {
            (
                Random::bytes(key_len).unwrap(),
                Random::bytes(value_len).unwrap(),
            )
        })
        .collect();

    for (key, value) in &items {
        let size = store.size();
        assert_eq!(size, expected_size);

        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store._lookup(&key);
        assert!(res.is_ok());
        assert!(!res.unwrap());

        let res = store._get(&key);
        assert!(res.is_err());

        let res = store._insert(&key, &value);
        assert!(res.is_ok());

        expected_size += (key.len() + value.len()) as u32;

        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = store._query(&key, &key, 0, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = store._lookup(&key);
        assert!(res.is_ok());
        assert!(res.unwrap());

        let res = store._get(&key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = store._remove(&key);
        assert!(res.is_ok());

        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store._query(&key, &key, 0, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);

        let res = store._lookup(&key);
        assert!(res.is_ok());
        assert!(!res.unwrap());

        let res = store._get(&key);
        assert!(res.is_err());
    }
    */
}

#[test]
fn test_persistent_store_async_ops() {
    /*
    use crypto::random::Random;
    use std::sync::{Arc, Mutex};
    use tempfile::Builder;

    let name = "test";
    let path_root = Builder::new().prefix("test_db").tempdir().unwrap();
    let path = path_root.path();
    let res = PersistentStore::new(name, &path);
    assert!(res.is_ok());
    let inner_store = res.unwrap();
    let store = Arc::new(Mutex::new(inner_store));

    let key_len = 100;
    let value_len = 1000;
    let expected_size = Arc::new(Mutex::new(0));;

    let items: Vec<(Vec<u8>, Vec<u8>)> = (0..10)
        .map(|_| {
            (
                Random::bytes(key_len).unwrap(),
                Random::bytes(value_len).unwrap(),
            )
        })
        .collect();

    for (key, value) in &items {
        let store = Arc::clone(&store);
        let expected_size = Arc::clone(&expected_size);

        let _ = async move {
            let mut store = store.lock().unwrap();
            let mut expected_size = expected_size.lock().unwrap();

            let size = store.size();
            assert_eq!(size, *expected_size);

            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());

            let res = store.insert_batch(&[]).await;
            assert!(res.is_err());

            let res = store.insert(&key, &value).await;
            assert!(res.is_ok());

            *expected_size += (key.len() + value.len()) as u32;

            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 1);

            let res = store.query(&key, &key, 0, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap().len(), 0);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_ok());
            assert_eq!(&res.unwrap(), value);

            let res = store.remove(&key).await;
            assert!(res.is_ok());

            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.query(&key, &key, 0, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), vec![value.to_owned()]);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());
        };
    }
    */
}
