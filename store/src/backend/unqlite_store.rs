//! # UnQLiteStore
//
// `unqlite_store` contains the `UnQLite` store backend type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::traits::{MemoryStore, PersistentStore, Store, TemporaryStore};
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

    fn _remove_range_complete(&mut self, from: &[u8], to: &[u8], skip: u32) -> Result<()> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut skipped = 0;

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                if skipped >= skip {
                    self._remove(&key)?;
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_no_skip(&mut self, from: &[u8], to: &[u8]) -> Result<()> {
        if from < to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                self._remove(&key)?;
            }

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_no_from(&mut self, to: &[u8], skip: u32) -> Result<()> {
        let mut skipped = 0;

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                if skipped >= skip {
                    self._remove(&key)?;
                } else {
                    skipped += 1;
                }
            }

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_no_from_no_skip(&mut self, to: &[u8]) -> Result<()> {
        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();
            let key = item.key();

            if to > key.as_slice() {
                self._remove(&key)?;
            }

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_no_to(&mut self, from: &[u8], skip: u32) -> Result<()> {
        let mut skipped = 0;

        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if skipped >= skip {
                let key = item.key();
                self._remove(&key)?;
            } else {
                skipped += 1;
            }

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_no_to_no_skip(&mut self, from: &[u8]) -> Result<()> {
        let mut entry = self.db.seek(from, Direction::Ge);

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            let key = item.key();
            self._remove(&key)?;

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_no_from_no_to(&mut self, skip: u32) -> Result<()> {
        let mut skipped = 0;

        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            if skipped >= skip {
                let key = item.key();
                self._remove(&key)?;
            } else {
                skipped += 1;
            }

            entry = item.next();
        }

        Ok(())
    }

    fn _remove_range_none(&mut self) -> Result<()> {
        let mut entry = self.db.first();

        loop {
            if entry.is_none() {
                break;
            }

            let item = entry.unwrap();

            let key = item.key();
            self._remove(&key)?;

            entry = item.next();
        }

        Ok(())
    }

    /// `_remove_range` removes a range of items from the `UnQLiteStore`.
    fn _remove_range(
        &mut self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        skip: Option<u32>,
    ) -> Result<()> {
        if let Some(from) = from {
            if let Some(to) = to {
                if from < to {
                    let err = Error::InvalidRange;
                    return Err(err);
                }

                if let Some(skip) = skip {
                    self._remove_range_complete(from, to, skip)
                } else {
                    self._remove_range_no_skip(from, to)
                }
            } else if let Some(skip) = skip {
                self._remove_range_no_to(from, skip)
            } else {
                self._remove_range_no_to_no_skip(from)
            }
        } else if let Some(to) = to {
            if let Some(skip) = skip {
                self._remove_range_no_from(to, skip)
            } else {
                self._remove_range_no_from_no_skip(to)
            }
        } else if let Some(skip) = skip {
            self._remove_range_no_from_no_to(skip)
        } else {
            self._remove_range_none()
        }
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
    fn keys_size(&self) -> u32 {
        self.keys_size
    }

    fn values_size(&self) -> u32 {
        self.values_size
    }

    fn size(&self) -> u32 {
        self.keys_size + self.values_size
    }

    fn lookup(&self, key: &[u8]) -> Result<bool> {
        Ok(self._lookup(key))
    }

    fn get(&self, key: &[u8]) -> Result<Vec<u8>> {
        self._get(key)
    }

    fn query(
        &self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Vec<u8>>> {
        self._query(from, to, count, skip)
    }

    fn count(&self, from: Option<&[u8]>, to: Option<&[u8]>, skip: Option<u32>) -> Result<u32> {
        self._count(from, to, skip)
    }

    fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self._insert(key, value)
    }

    fn create(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self._create(key, value)
    }

    fn update(&mut self, key: &[u8], value: &[u8]) -> Result<()> {
        self._update(key, value)
    }

    fn insert_batch(&mut self, _items: &[(&[u8], &[u8])]) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn remove(&mut self, key: &[u8]) -> Result<()> {
        self._remove(key)
    }

    fn remove_batch(&mut self, _keys: &[&[u8]]) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn remove_range(
        &mut self,
        from: Option<&[u8]>,
        to: Option<&[u8]>,
        skip: Option<u32>,
    ) -> Result<()> {
        self._remove_range(from, to, skip)
    }

    fn clear(&mut self) -> Result<()> {
        self._clear()
    }
}

impl MemoryStore for UnQLiteStore {}

impl TemporaryStore for UnQLiteStore {}

impl PersistentStore for UnQLiteStore {}

#[test]
fn test_persistent_store_ops() {
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

        let res = store.count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store.lookup(&key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = store.get(&key);
        assert!(res.is_err());

        let res = store.insert(&key, &value);
        assert!(res.is_ok());

        expected_size += (key.len() + value.len()) as u32;

        let res = store.count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = store.query(Some(&key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);

        let res = store.lookup(&key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = store.get(&key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = store.remove(&key);
        assert!(res.is_ok());

        expected_size -= (key.len() + value.len()) as u32;

        let res = store.count(Some(&key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = store.query(Some(&key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![] as Vec<Vec<u8>>);

        let res = store.lookup(&key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = store.get(&key);
        assert!(res.is_err());

        let res = store.insert(&key, &value);
        assert!(res.is_ok());

        let res = store.clear();
        assert!(res.is_ok());

        assert_eq!(store.keys_size(), 0);
        assert_eq!(store.values_size(), 0);
    }
}
