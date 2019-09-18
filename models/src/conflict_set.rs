//! # Conflict Set
//!
//! `conflict_set` is the module containing the type used to register mutually conflicting
//! transactions.

use crate::account::Account;
use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crate::transaction::Transaction;
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeSet;
use store::traits::Store;

/// `ConflictSet` is the set used to represent a set of mutually conflicting transactions.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Default, Serialize, Deserialize)]
pub struct ConflictSet {
    pub address: Address,
    pub stage: Stage,
    pub transactions: BTreeSet<Digest>,
    pub last: Option<Digest>,
    pub preferred: Option<Digest>,
    pub count: u32,
}

impl ConflictSet {
    /// `new` creates a new `ConflictSet`.
    pub fn new(address: Address, stage: Stage) -> ConflictSet {
        let mut set = ConflictSet::default();
        set.address = address;
        set.stage = stage;
        set
    }

    /// `lookup_transaction` looks up a `Transaction` id in the transactions set of the `ConflictSet`.
    pub fn lookup_transaction(&self, tx_id: &Digest) -> bool {
        self.transactions.contains(tx_id)
    }

    /// `add_transaction` adds a new `Transaction` id in the transactions set of the `ConflictSet`.
    pub fn add_transaction(&mut self, tx_id: Digest) {
        if !self.lookup_transaction(&tx_id) {
            self.transactions.insert(tx_id);

            self.last = Some(tx_id);

            if self.preferred.is_none() {
                self.preferred = Some(tx_id);
            }
        }
    }

    /// `remove_transaction` removes a `Transaction` from the transaction set of the `ConflictSet`.
    pub fn remove_transaction(&mut self, tx_id: &Digest) -> Result<()> {
        if !self.lookup_transaction(tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.transactions.remove(tx_id);

        if let Some(ref last) = self.last {
            if last == tx_id {
                self.last = None;
            }
        }

        if let Some(ref preferred) = self.preferred {
            if preferred == tx_id {
                self.preferred = None;
            }
        }

        Ok(())
    }

    /// `set_last` sets the last transaction in the `ConflictSet`.
    pub fn set_last(&mut self, tx_id: Digest) -> Result<()> {
        if !self.lookup_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.last = Some(tx_id);

        Ok(())
    }

    /// `set_preferred` sets the preferred transaction in the `ConflictSet`.
    pub fn set_preferred(&mut self, tx_id: Digest) -> Result<()> {
        if !self.lookup_transaction(&tx_id) {
            let err = Error::NotFound;
            return Err(err);
        }

        self.preferred = Some(tx_id);

        Ok(())
    }

    /// `validate` validates the `ConflictSet`.
    pub fn validate(&self) -> Result<()> {
        if let Some(last) = self.last {
            if !self.lookup_transaction(&last) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if let Some(preferred) = self.preferred {
            if !self.lookup_transaction(&preferred) {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if self.last.is_some() ^ self.preferred.is_some() {
            let err = Error::NotFound;
            return Err(err);
        }

        Ok(())
    }

    /// `clear` clears the `ConflictSet`.
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.last = None;
        self.preferred = None;
        self.count = 0;
    }

    /// `to_bytes` converts the `ConflictSet` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `ConflictSet`.
    pub fn from_bytes(b: &[u8]) -> Result<ConflictSet> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `ConflictSet` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `ConflictSet`.
    pub fn from_json(s: &str) -> Result<ConflictSet> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for ConflictSet {
    const KEY_PREFIX: u8 = 5;

    type Key = Address;

    fn key(&self) -> Self::Key {
        self.address
    }

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key.to_bytes());
        Ok(buf)
    }

    fn validate_single(store: &S, stage: Stage, value: &Self) -> Result<()> {
        if value.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        value.validate()?;

        if !Account::lookup(store, stage, &value.address)? {
            let err = Error::NotFound;
            return Err(err);
        }

        for id in &value.transactions {
            if !Transaction::lookup(store, stage, &id)? {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if let Some(id) = value.last {
            if !Transaction::lookup(store, stage, &id)? {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        if let Some(id) = value.preferred {
            if !Transaction::lookup(store, stage, &id)? {
                let err = Error::NotFound;
                return Err(err);
            }
        }

        Ok(())
    }

    fn validate_all(store: &S, stage: Stage) -> Result<()> {
        for value in Self::query(store, stage, None, None, None, None)? {
            Self::validate_single(store, stage, &value)?;
        }

        Ok(())
    }

    fn lookup(store: &S, stage: Stage, key: &Self::Key) -> Result<bool> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.lookup(&key).map_err(|e| e.into())
    }

    fn get(store: &S, stage: Stage, key: &Self::Key) -> Result<Self> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let buf = store.get(&key)?;
        Self::from_bytes(&buf)
    }

    fn query(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<BTreeSet<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _from = Digest::default();
            _from[0] = stage as u8;
            _from[1] = <Self as Storable<S>>::KEY_PREFIX;
            Some(_from.to_vec())
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _to = Digest::default();
            _to[0] = stage as u8;
            _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
            Some(_to.to_vec())
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.query(from, to, count, skip)?;
        let mut items = BTreeSet::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.insert(item);
        }

        Ok(items)
    }

    fn sample(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: u32,
    ) -> Result<BTreeSet<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _from = Digest::default();
            _from[0] = stage as u8;
            _from[1] = <Self as Storable<S>>::KEY_PREFIX;
            Some(_from.to_vec())
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _to = Digest::default();
            _to[0] = stage as u8;
            _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
            Some(_to.to_vec())
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.sample(from, to, count)?;
        let mut items = BTreeSet::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.insert(item);
        }

        Ok(items)
    }

    fn count(
        store: &S,
        stage: Stage,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _from = Digest::default();
            _from[0] = stage as u8;
            _from[1] = <Self as Storable<S>>::KEY_PREFIX;
            Some(_from.to_vec())
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            let mut _to = Digest::default();
            _to[0] = stage as u8;
            _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
            Some(_to.to_vec())
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.insert(&store_key, &store_value).map_err(|e| e.into())
    }

    fn create(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.create(&store_key, &store_value).map_err(|e| e.into())
    }

    fn update(store: &mut S, stage: Stage, value: &Self) -> Result<()> {
        Self::validate_single(store, stage, value)?;

        let key = <Self as Storable<S>>::key(value);
        let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
        let store_value = value.to_bytes()?;
        store.update(&store_key, &store_value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, stage: Stage, values: &BTreeSet<Self>) -> Result<()> {
        let mut items = BTreeSet::new();

        for value in values {
            Self::validate_single(store, stage, value)?;

            let key = <Self as Storable<S>>::key(value);
            let store_key = <Self as Storable<S>>::key_to_bytes(stage, &key)?;
            let store_value = value.to_bytes()?;
            let item = (store_key, store_value);
            items.insert(item);
        }

        let items: Vec<(&[u8], &[u8])> = items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, stage: Stage, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, stage: Stage, keys: &BTreeSet<Self::Key>) -> Result<()> {
        let mut _keys = BTreeSet::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            _keys.insert(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(_store: &mut S, _stage: Stage, _min_time: Option<Timestamp>) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn clear(store: &mut S, stage: Stage) -> Result<()> {
        let from = Some(vec![stage as u8, <Self as Storable<S>>::KEY_PREFIX]);
        let from = from.as_ref().map(|from| from.as_slice());

        let to = Some(vec![stage as u8, <Self as Storable<S>>::KEY_PREFIX + 1]);
        let to = to.as_ref().map(|to| to.as_slice());

        store.remove_range(from, to, None).map_err(|e| e.into())
    }
}

#[test]
fn test_conflict_set_ops() {
    let addr = Address::random().unwrap();
    let stage = Stage::random().unwrap();
    let mut conflict_set = ConflictSet::new(addr, stage);

    let res = conflict_set.validate();
    assert!(res.is_ok());

    let tx_id_1 = Digest::random().unwrap();

    let found = conflict_set.lookup_transaction(&tx_id_1);
    assert!(!found);

    let res = conflict_set.remove_transaction(&tx_id_1);
    assert!(res.is_err());

    conflict_set.add_transaction(tx_id_1);

    assert_eq!(conflict_set.last, Some(tx_id_1));
    assert_eq!(conflict_set.preferred, Some(tx_id_1));

    let found = conflict_set.lookup_transaction(&tx_id_1);
    assert!(found);

    let res = conflict_set.remove_transaction(&tx_id_1);
    assert!(res.is_ok());

    assert_eq!(conflict_set.last, None);
    assert_eq!(conflict_set.preferred, None);

    let res = conflict_set.set_last(tx_id_1);
    assert!(res.is_err());

    let res = conflict_set.set_preferred(tx_id_1);
    assert!(res.is_err());

    conflict_set.add_transaction(tx_id_1);

    let tx_id_2 = Digest::random().unwrap();

    conflict_set.add_transaction(tx_id_2);

    assert_eq!(conflict_set.last, Some(tx_id_2));
    assert_eq!(conflict_set.preferred, Some(tx_id_1));

    let tx_id_3 = Digest::random().unwrap();

    let res = conflict_set.set_last(tx_id_3);
    assert!(res.is_err());

    let res = conflict_set.set_preferred(tx_id_3);
    assert!(res.is_err());

    let res = conflict_set.set_last(tx_id_1);
    assert!(res.is_ok());
    assert_eq!(conflict_set.last, Some(tx_id_1));

    let res = conflict_set.set_preferred(tx_id_2);
    assert!(res.is_ok());
    assert_eq!(conflict_set.preferred, Some(tx_id_2));

    conflict_set.clear();
    assert!(conflict_set.transactions.is_empty());
    assert_eq!(conflict_set.last, None);
    assert_eq!(conflict_set.preferred, None);
    assert_eq!(conflict_set.count, 0);
}

#[test]
fn test_conflict_set_serialize_bytes() {
    let conflict_set_a = ConflictSet::default();

    let res = conflict_set_a.to_bytes();
    assert!(res.is_ok());
    let cbor = res.unwrap();

    let res = ConflictSet::from_bytes(&cbor);
    assert!(res.is_ok());
    let conflict_set_b = res.unwrap();

    assert_eq!(conflict_set_a, conflict_set_b)
}

#[test]
fn test_conflict_set_serialize_json() {
    let conflict_set_a = ConflictSet::default();

    let res = conflict_set_a.to_json();
    assert!(res.is_ok());
    let json = res.unwrap();

    let res = ConflictSet::from_json(&json);
    assert!(res.is_ok());
    let conflict_set_b = res.unwrap();

    assert_eq!(conflict_set_a, conflict_set_b)
}

#[test]
fn test_conflict_set_storable() {
    use crate::signers::Signers;
    use crate::wallet::Wallet;
    use store::backend::BTreeStore;
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_btree(max_value_size, max_size).unwrap();

    let stage = Stage::random().unwrap();

    let wallet = Wallet::new(stage).unwrap();
    let weight = 1;
    let signer = wallet.to_signer(weight).unwrap();
    let mut signers = Signers::new().unwrap();
    signers.set_threshold(weight).unwrap();
    signers.add(&signer).unwrap();

    let mut account = Account::new_eve(stage, &signers).unwrap();
    let transaction = Transaction::new_eve(stage, &account.address()).unwrap();

    Transaction::create(&mut store, stage, &transaction).unwrap();

    account.transaction_id = Some(transaction.id);
    account.counter += 1;

    Account::create(&mut store, stage, &account).unwrap();

    let address = account.address();
    let cs = ConflictSet::new(address, stage);

    let res = ConflictSet::count(&store, stage, Some(address), None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 0);

    let res = ConflictSet::query(&store, stage, Some(address), None, None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 0);

    let res = ConflictSet::lookup(&store, stage, &address);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(!found);

    let res = ConflictSet::get(&store, stage, &address);
    assert!(res.is_err());

    let res = ConflictSet::insert(&mut store, stage, &cs);
    assert!(res.is_ok());

    let res = ConflictSet::count(&store, stage, Some(address), None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 1);

    let res = ConflictSet::query(&store, stage, Some(address), None, None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().iter().next(), Some(&cs));

    let res = ConflictSet::lookup(&store, stage, &address);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(found);

    let res = ConflictSet::get(&store, stage, &address);
    assert!(res.is_ok());
    assert_eq!(&res.unwrap(), &cs);

    let res = <ConflictSet as Storable<BTreeStore>>::remove(&mut store, stage, &address);
    assert!(res.is_ok());

    let res = ConflictSet::count(&store, stage, Some(address), None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 0);

    let res = ConflictSet::query(&store, stage, Some(address), None, None, None);
    assert!(res.is_ok());
    assert_eq!(res.unwrap().len(), 0);

    let res = ConflictSet::lookup(&store, stage, &address);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(!found);

    let res = ConflictSet::get(&store, stage, &address);
    assert!(res.is_err());

    let res = ConflictSet::insert(&mut store, stage, &cs);
    assert!(res.is_ok());

    let res = <ConflictSet as Storable<BTreeStore>>::clear(&mut store, stage);
    assert!(res.is_ok());

    let res = ConflictSet::lookup(&store, stage, &address);
    assert!(res.is_ok());
    let found = res.unwrap();
    assert!(!found);
}
