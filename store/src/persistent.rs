//! # Persistent
//
// `persistent` contains the persistent store type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use futures::future::{self, BoxFuture};
use std::io::Cursor;
use unqlite::Cursor as StoreCursor;
use unqlite::{Config, Direction, UnQLite, KV};

pub struct PersistentStore {
    db: UnQLite,
    keys_size: u32,
    values_size: u32,
}

impl PersistentStore {
    /// `new` creates a new `PersistentStore`.
    pub fn new(path: &str) -> Result<PersistentStore> {
        let db = UnQLite::create(path);

        let mut store = PersistentStore {
            db,
            keys_size: 0,
            values_size: 0,
        };

        store.init_size()?;

        Ok(store)
    }

    /// `new_from_db` creates a new `PersistentStore` from an UnQlite database.
    pub fn new_from_db(db: UnQLite) -> Result<PersistentStore> {
        let mut store = PersistentStore {
            db,
            keys_size: 0,
            values_size: 0,
        };

        store.init_size()?;

        Ok(store)
    }

    /// `sizes_to_bytes` returns the binary representation of the store sizes.
    pub fn sizes_to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.write_u32::<BigEndian>(self.keys_size)?;
        buf.write_u32::<BigEndian>(self.values_size)?;
        Ok(buf)
    }

    /// `sizes_from_bytes` returns the store sizes from a binary representation.
    pub fn sizes_from_bytes(buf: &[u8]) -> Result<(u32, u32)> {
        if buf.len() != 8 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut reader = Cursor::new(buf);
        let keys_size = reader.read_u32::<BigEndian>()?;
        let values_size = reader.read_u32::<BigEndian>()?;

        Ok((keys_size, values_size))
    }

    /// `init_size` initializes the `PersistentStore` cached sizes.
    fn init_size(&mut self) -> Result<()> {
        if !self._lookup(b"sizes") {
            self.db.kv_store(b"sizes", &[0u8; 4]).map_err(|e| e.into())
        } else {
            let buf = self._get(b"sizes")?;
            let (keys_size, values_size) = Self::sizes_from_bytes(&buf)?;

            self.keys_size = keys_size;
            self.values_size = values_size;
            Ok(())
        }
    }

    /// `update_size` updates the `PersistentStore` cached sizes.
    fn update_size(&mut self) -> Result<()> {
        let buf = self.sizes_to_bytes()?;
        self.db.kv_store(b"sizes", &buf).map_err(|e| e.into())
    }

    /// `log_errors` logs the `PersistentStore` errors.
    pub fn log_errors(&self) -> Option<String> {
        self.db.err_log()
    }

    /// `_lookup` looks up a key-value pair from the `PersistentStore`.
    fn _lookup(&self, key: &[u8]) -> bool {
        self.db.kv_contains(key)
    }

    /// `_get` gets a key-value pair from the `PersistentStore`.
    fn _get(&self, key: &[u8]) -> Result<Vec<u8>> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.db.kv_fetch(key).map_err(|e| e.into())
    }

    /// `_count` returns the count of a list of values from the `PersistentStore`.
    fn _count(&self, from: &[u8], to: &[u8], skip: u32) -> Result<u32> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut skipped = 0;
        let mut count = 0;

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                if skipped >= skip {
                    count += 1;
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(count)
    }

    /// `_query` returns a list of values from the `PersistentStore`.
    fn _query(&self, from: &[u8], to: &[u8], count: u32, skip: u32) -> Result<Vec<Vec<u8>>> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut skipped = 0;
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                if skipped >= skip {
                    if counted <= count {
                        values.push(item.value());
                        counted += 1;
                    } else {
                        break;
                    }
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(values)
    }

    /// `_insert` inserts a binary key-value pair in the `PersistentStore`.
    fn _insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.kv_store(key, value)?;
        self.keys_size += key.len() as u32;
        self.values_size += value.len() as u32;
        self.update_size()?;

        Ok(())
    }

    /// `_create` inserts a non-existing binary key-value pair in the `PersistentStore`.
    fn _create(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if self._lookup(key) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_update` updates an existing key-value pair in the `PersistentStore`.
    pub fn _update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_remove` removes a key-value pair from the `PersistentStore`.
    fn _remove(&mut self, key: &[u8]) -> Result<()> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        let value_len = self.db.kv_fetch_length(key)?;

        self.db.kv_delete(key)?;
        self.keys_size -= key.len() as u32;
        self.values_size -= value_len as u32;
        self.update_size()?;

        Ok(())
    }

    /// `clear` clears the `PersistentStore`.
    pub fn clear(&mut self) -> Result<()> {
        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();
            let value_len = item.value().len();

            self.db.kv_delete(&key)?;
            self.keys_size -= key.len() as u32;
            self.values_size -= value_len as u32;

            entry = item.next();
        }

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
        Box::pin(future::ok(res))
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
    use crypto::random::Random;

    let temp_db = UnQLite::create_temp();
    let res = PersistentStore::new_from_db(temp_db);
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

        let found = store._lookup(&key);
        assert!(!found);

        let res = store._get(&key);
        assert!(res.is_err());

        let res = store._insert(&key, &value);
        assert!(res.is_ok());

        expected_size += (key.len() + value.len()) as u32;

        let res = store.sizes_to_bytes();
        assert!(res.is_ok());
        let sizes_buf = res.unwrap();
        let res = PersistentStore::sizes_from_bytes(&sizes_buf);
        assert!(res.is_ok());
        let (keys_size, values_size) = res.unwrap();
        assert_eq!(store.keys_size(), keys_size);
        assert_eq!(store.values_size(), values_size);

        /*
        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = store._query(&key, &key, 0, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);
        */

        let found = store._lookup(&key);
        assert!(found);

        let res = store._get(&key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = store._remove(&key);
        assert!(res.is_ok());

        expected_size -= (key.len() + value.len()) as u32;

        let res = store.sizes_to_bytes();
        assert!(res.is_ok());
        let sizes_buf = res.unwrap();
        let res = PersistentStore::sizes_from_bytes(&sizes_buf);
        assert!(res.is_ok());
        let (keys_size, values_size) = res.unwrap();
        assert_eq!(store.keys_size(), keys_size);
        assert_eq!(store.values_size(), values_size);

        /*
        let res = store._count(&key, &key, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store._query(&key, &key, 0, 0);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);
        */

        let found = store._lookup(&key);
        assert!(!found);

        let res = store._get(&key);
        assert!(res.is_err());
    }
}

#[test]
fn test_persistent_store_async_ops() {
    use crypto::random::Random;
    use std::sync::{Arc, Mutex};

    let temp_db = UnQLite::create_temp();
    let res = PersistentStore::new_from_db(temp_db);
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

            let res = store.sizes_to_bytes();
            assert!(res.is_ok());
            let sizes_buf = res.unwrap();
            let res = PersistentStore::sizes_from_bytes(&sizes_buf);
            assert!(res.is_ok());
            let (keys_size, values_size) = res.unwrap();
            assert_eq!(store.keys_size(), keys_size);
            assert_eq!(store.values_size(), values_size);

            /*
            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 1);

            let res = store.query(&key, &key, 0, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap().len(), 0);
            */

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_ok());
            assert_eq!(&res.unwrap(), value);

            let res = store.remove(&key).await;
            assert!(res.is_ok());

            *expected_size -= (key.len() + value.len()) as u32;

            let res = store.sizes_to_bytes();
            assert!(res.is_ok());
            let sizes_buf = res.unwrap();
            let res = PersistentStore::sizes_from_bytes(&sizes_buf);
            assert!(res.is_ok());
            let (keys_size, values_size) = res.unwrap();
            assert_eq!(store.keys_size(), keys_size);
            assert_eq!(store.values_size(), values_size);

            /*
            let res = store.count(&key, &key, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.query(&key, &key, 0, 0).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), vec![value.to_owned()]);
            */

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());
        };
    }
}
