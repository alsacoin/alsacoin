//! # UnQLiteStore
//
// `unqlite_store` contains the `UnQLite` store backend type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::Store;
use futures::future::{self, BoxFuture};
use unqlite::Cursor as StoreCursor;
use unqlite::{Config, Direction, UnQLite, KV};

pub struct UnQLiteStore {
    db: UnQLite,
    keys_size: u32,
    values_size: u32,
}

impl UnQLiteStore {
    /// `new_from_db` creates a new `UnQLiteStore` from an UnQlite database.
    pub fn new_from_db(db: UnQLite) -> Result<UnQLiteStore> {
        let mut store = UnQLiteStore {
            db,
            keys_size: 0,
            values_size: 0,
        };

        store.fetch_sizes()?;

        Ok(store)
    }

    /// `new_memory` creates a new in-memory `UnQLiteStore`.
    pub fn new_memory() -> Result<UnQLiteStore> {
        let db = UnQLite::create_in_memory();
        Self::new_from_db(db)
    }

    /// `new_temporary` creates a new temporary `UnQLiteStore`.
    pub fn new_temporary() -> Result<UnQLiteStore> {
        let db = UnQLite::create_temp();
        Self::new_from_db(db)
    }

    /// `new_persistent` creates a new persistent `UnQLiteStore`.
    pub fn new_persistent(path: &str) -> Result<UnQLiteStore> {
        let db = UnQLite::create(path);
        Self::new_from_db(db)
    }

    /// `fetch_sizes` fetches the `UnQLiteStore` cached sizes.
    fn fetch_sizes(&mut self) -> Result<()> {
        let mut entry = self.db.first();
        let mut keys_size = 0;
        let mut values_size = 0;

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            keys_size += item.key().len() as u32;
            values_size += item.value().len() as u32;

            entry = item.next();
        }

        self.keys_size = keys_size;
        self.values_size = values_size;

        Ok(())
    }

    /// `log_errors` logs the `UnQLiteStore` errors.
    pub fn log_errors(&self) -> Option<String> {
        self.db.err_log()
    }

    /// `_lookup` looks up a key-value pair from the `UnQLiteStore`.
    fn _lookup(&self, key: &[u8]) -> bool {
        self.db.kv_contains(key)
    }

    /// `_get` gets a key-value pair from the `UnQLiteStore`.
    fn _get(&self, key: &[u8]) -> Result<Vec<u8>> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.db.kv_fetch(key).map_err(|e| e.into())
    }

    fn _count_complete(&self, from: &[u8], to: &[u8], skip: u32) -> Result<u32> {
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

    fn _count_no_skip(&self, from: &[u8], to: &[u8]) -> Result<u32> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut count = 0;

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                count += 1;
            }

            entry = item.next();
        }

        Ok(count)
    }

    fn _count_no_from(&self, to: &[u8], skip: u32) -> Result<u32> {
        let mut skipped = 0;
        let mut count = 0;

        let mut entry = self.db.first();

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

    fn _count_no_from_no_skip(&self, to: &[u8]) -> Result<u32> {
        let mut count = 0;

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                count += 1;
            }

            entry = item.next();
        }

        Ok(count)
    }

    fn _count_no_to(&self, from: &[u8], skip: u32) -> Result<u32> {
        let mut skipped = 0;
        let mut count = 0;

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            if skipped >= skip {
                count += 1;
            } else {
                skipped += 1;
            }

            entry = entry.unwrap().next();
        }

        Ok(count)
    }

    fn _count_no_to_no_skip(&self, from: &[u8]) -> Result<u32> {
        let mut count = 0;

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            count += 1;

            entry = entry.unwrap().next();
        }

        Ok(count)
    }

    fn _count_no_from_no_to(&self, skip: u32) -> Result<u32> {
        let mut skipped = 0;
        let mut count = 0;

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            if skipped >= skip {
                count += 1;
            } else {
                skipped += 1;
            }

            entry = entry.unwrap().next();
        }

        Ok(count)
    }

    fn _count_none(&self) -> Result<u32> {
        let mut count = 0;

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            count += 1;

            entry = entry.unwrap().next();
        }

        Ok(count)
    }

    /// `_count` returns the count of a list of values from the `UnQLiteStore`.
    fn _count(&self, from: Option<&[u8]>, to: Option<&[u8]>, skip: Option<u32>) -> Result<u32> {
        if let Some(from) = from {
            if let Some(to) = to {
                if from < to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }

                if let Some(skip) = skip {
                    self._count_complete(from, to, skip)
                } else {
                    self._count_no_skip(from, to)
                }
            } else if let Some(skip) = skip {
                self._count_no_to(from, skip)
            } else {
                self._count_no_to_no_skip(from)
            }
        } else if let Some(to) = to {
            if let Some(skip) = skip {
                self._count_no_from(to, skip)
            } else {
                self._count_no_from_no_skip(to)
            }
        } else if let Some(skip) = skip {
            self._count_no_from_no_to(skip)
        } else {
            self._count_none()
        }
    }

    fn _query_complete(
        &self,
        from: &[u8],
        to: &[u8],
        count: u32,
        skip: u32,
    ) -> Result<Vec<Vec<u8>>> {
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
                    }
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_count(&self, from: &[u8], to: &[u8], skip: u32) -> Result<Vec<Vec<u8>>> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut skipped = 0;
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
                    values.push(item.value());
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_skip(&self, from: &[u8], to: &[u8], count: u32) -> Result<Vec<Vec<u8>>> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() && counted <= count {
                values.push(item.value());
                counted += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_skip_no_count(&self, from: &[u8], to: &[u8]) -> Result<Vec<Vec<u8>>> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                values.push(item.value());
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from(&self, to: &[u8], count: u32, skip: u32) -> Result<Vec<Vec<u8>>> {
        let mut skipped = 0;
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.first();

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
                    }
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from_no_count(&self, to: &[u8], skip: u32) -> Result<Vec<Vec<u8>>> {
        let mut skipped = 0;
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                if skipped >= skip {
                    values.push(item.value());
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from_no_skip(&self, to: &[u8], count: u32) -> Result<Vec<Vec<u8>>> {
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() && counted <= count {
                values.push(item.value());
                counted += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from_no_skip_no_count(&self, to: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                values.push(item.value());
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_to(&self, from: &[u8], count: u32, skip: u32) -> Result<Vec<Vec<u8>>> {
        let mut skipped = 0;
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if skipped >= skip {
                if counted <= count {
                    values.push(item.value());
                    counted += 1;
                }
            } else {
                skipped += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_to_no_count(&self, from: &[u8], skip: u32) -> Result<Vec<Vec<u8>>> {
        let mut skipped = 0;
        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if skipped >= skip {
                values.push(item.value());
            } else {
                skipped += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_to_no_skip(&self, from: &[u8], count: u32) -> Result<Vec<Vec<u8>>> {
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if counted <= count {
                values.push(item.value());
                counted += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_to_no_skip_no_count(&self, from: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut values = Vec::new();

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            values.push(item.value());

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from_no_to(&self, count: u32, skip: u32) -> Result<Vec<Vec<u8>>> {
        let mut skipped = 0;
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if skipped >= skip {
                if counted <= count {
                    values.push(item.value());
                    counted += 1;
                }
            } else {
                skipped += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from_no_to_no_count(&self, skip: u32) -> Result<Vec<Vec<u8>>> {
        let mut skipped = 0;
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if skipped >= skip {
                values.push(item.value());
            } else {
                skipped += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_no_from_no_to_no_skip(&self, count: u32) -> Result<Vec<Vec<u8>>> {
        let mut counted = 0;
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if counted <= count {
                values.push(item.value());
                counted += 1;
            }

            entry = item.next();
        }

        Ok(values)
    }

    fn _query_none(&self) -> Result<Vec<Vec<u8>>> {
        let mut values = Vec::new();

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            values.push(item.value());

            entry = item.next();
        }

        Ok(values)
    }

    /// `_query` returns a list of values from the `UnQLiteStore`.
    fn _query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Vec<u8>>> {
        if let Some(from) = from {
            if let Some(to) = to {
                if from < to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }

                if let Some(skip) = skip {
                    if let Some(count) = count {
                        self._query_complete(from, to, skip, count)
                    } else {
                        self._query_no_count(from, to, skip)
                    }
                } else if let Some(count) = count {
                    self._query_no_skip(from, to, count)
                } else {
                    self._query_no_skip_no_count(from, to)
                }
            } else if let Some(skip) = skip {
                if let Some(count) = count {
                    self._query_no_to(from, skip, count)
                } else {
                    self._query_no_to_no_count(from, skip)
                }
            } else if let Some(count) = count {
                self._query_no_to_no_skip(from, count)
            } else {
                self._query_no_to_no_skip_no_count(from)
            }
        } else if let Some(to) = to {
            if let Some(skip) = skip {
                if let Some(count) = count {
                    self._query_no_from(to, skip, count)
                } else {
                    self._query_no_from_no_count(to, skip)
                }
            } else if let Some(count) = count {
                self._query_no_from_no_skip(to, count)
            } else {
                self._query_no_from_no_skip_no_count(to)
            }
        } else if let Some(skip) = skip {
            if let Some(count) = count {
                self._query_no_from_no_to(skip, count)
            } else {
                self._query_no_from_no_to_no_count(skip)
            }
        } else if let Some(count) = count {
            self._query_no_from_no_to_no_skip(count)
        } else {
            self._query_none()
        }
    }

    /// `_insert` inserts a binary key-value pair in the `UnQLiteStore`.
    fn _insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self.db.kv_store(key, value)?;
        self.keys_size += key.len() as u32;
        self.values_size += value.len() as u32;

        Ok(())
    }

    /// `_create` inserts a non-existing binary key-value pair in the `UnQLiteStore`.
    fn _create(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if self._lookup(key) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_update` updates an existing key-value pair in the `UnQLiteStore`.
    pub fn _update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        self._insert(key, value)
    }

    /// `_remove` removes a key-value pair from the `UnQLiteStore`.
    fn _remove(&mut self, key: &[u8]) -> Result<()> {
        if !self._lookup(key) {
            let err = Error::NotFound;
            return Err(err);
        }

        let value_len = self.db.kv_fetch_length(key)?;

        self.db.kv_delete(key)?;
        self.keys_size -= key.len() as u32;
        self.values_size -= value_len as u32;

        Ok(())
    }

    /// `_clear` clears the `UnQLiteStore`.
    fn _clear(&mut self) -> Result<()> {
        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            self.db.kv_delete(&key)?;

            let key_size = key.len() as u32;
            let value_size = item.value().len() as u32;

            if self.keys_size >= key_size {
                self.keys_size -= key_size;
            }

            if self.values_size >= value_size {
                self.values_size -= value_size;
            }

            entry = item.next();
        }

        Ok(())
    }
}

impl Store for UnQLiteStore {
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
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<Vec<Self::Value>>> {
        let from = from.map(|from| from.as_slice());
        let to = to.map(|to| to.as_slice());
        let res = self._query(from, to, count, skip);
        Box::pin(future::ready(res))
    }

    fn count(
        &self,
        from: Option<&Self::Key>,
        to: Option<&Self::Key>,
        skip: Option<u32>,
    ) -> BoxFuture<Result<u32>> {
        let from = from.map(|from| from.as_slice());
        let to = to.map(|to| to.as_slice());
        let res = self._count(from, to, skip);
        Box::pin(future::ready(res))
    }

    fn insert(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>> {
        let res = self._insert(key, value);
        Box::pin(future::ready(res))
    }

    fn create(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>> {
        let res = self._create(key, value);
        Box::pin(future::ready(res))
    }

    fn update(&mut self, key: &Self::Key, value: &Self::Value) -> BoxFuture<Result<()>> {
        let res = self._update(key, value);
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

    fn clear(&mut self) -> BoxFuture<Result<()>> {
        let res = self._clear();
        Box::pin(future::ready(res))
    }
}

#[test]
fn test_persistent_store_sync_ops() {
    use crypto::random::Random;

    let res = UnQLiteStore::new_temporary();
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

        let res = store._count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let found = store._lookup(&key);
        assert!(!found);

        let res = store._get(&key);
        assert!(res.is_err());

        let res = store._insert(&key, &value);
        assert!(res.is_ok());

        expected_size += (key.len() + value.len()) as u32;

        let res = store._count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = store._query(Some(&key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);

        let found = store._lookup(&key);
        assert!(found);

        let res = store._get(&key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = store._remove(&key);
        assert!(res.is_ok());

        expected_size -= (key.len() + value.len()) as u32;

        let res = store._count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store._query(Some(&key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![] as Vec<Vec<u8>>);

        let found = store._lookup(&key);
        assert!(!found);

        let res = store._get(&key);
        assert!(res.is_err());

        let res = store._insert(&key, &value);
        assert!(res.is_ok());

        let res = store._clear();
        assert!(res.is_ok());
        assert_eq!(store.keys_size(), 0);
        assert_eq!(store.values_size(), 0);
    }
}

#[test]
fn test_persistent_store_async_ops() {
    use crypto::random::Random;
    use std::sync::{Arc, Mutex};

    let res = UnQLiteStore::new_temporary();
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

            let res = store.count(Some(&key), None, None).await;
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

            let res = store.count(Some(&key), None, None).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 1);

            let res = store.query(Some(&key), None, None, None).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), vec![value.to_owned()]);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_ok());
            assert_eq!(&res.unwrap(), value);

            let res = store.remove(&key).await;
            assert!(res.is_ok());

            *expected_size -= (key.len() + value.len()) as u32;

            let res = store.count(Some(&key), None, None).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), 0);

            let res = store.query(Some(&key), None, None, None).await;
            assert!(res.is_ok());
            assert_eq!(res.unwrap(), vec![] as Vec<Vec<u8>>);

            let res = store.lookup(&key).await;
            assert!(res.is_ok());
            assert!(!res.unwrap());

            let res = store.get(&key).await;
            assert!(res.is_err());
        };
    }
}
