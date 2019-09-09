//! # Account
//!
//! `account` contains the `Account` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crate::signers::Signers;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crypto::hash::Digest;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeSet;
use store::traits::Store;

/// `Account` is the type used to represent an Alsacoin account
/// of a user, account which is identified by an `Address`.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub stage: Stage,
    pub timestamp: Timestamp,
    pub signers: Signers,
    pub value: u64, // NB: gonna be confidential
    pub counter: u64,
    pub transaction_id: Option<Digest>,
}

impl Account {
    /// `new` creates a new `Account`.
    pub fn new(
        stage: Stage,
        signers: &Signers,
        value: u64,
        tx_id: Option<Digest>,
    ) -> Result<Account> {
        signers.validate()?;

        let account = Account {
            address: signers.address,
            stage,
            timestamp: Timestamp::now(),
            signers: signers.to_owned(),
            value,
            counter: 0,
            transaction_id: tx_id,
        };

        Ok(account)
    }

    /// `new_eve` creates a new eve `Account`.
    pub fn new_eve(stage: Stage, signers: &Signers) -> Result<Account> {
        signers.validate()?;

        Account::new(stage, signers, 0, None)
    }

    /// `is_eve` returns if the `Account` is an eve `Account`.
    pub fn is_eve(&self) -> Result<bool> {
        self.signers.validate()?;

        let res = self.value == 0 && self.transaction_id.is_none() && self.counter == 0;

        Ok(res)
    }

    /// `update` updates the `Account`.
    pub fn update(&mut self, value: u64, tx_id: Digest) {
        self.value = value;
        self.transaction_id = Some(tx_id);
        self.counter += 1;
    }

    /// `validate` validates the `Account`.
    pub fn validate(&self) -> Result<()> {
        self.timestamp.validate()?;
        self.signers.validate()?;

        if self.address != self.signers.address {
            let err = Error::InvalidAddress;
            return Err(err);
        }

        if (self.value != 0 || self.counter != 0) && self.transaction_id.is_none() {
            let err = Error::InvalidAccount;
            return Err(err);
        }

        Ok(())
    }

    /// `to_bytes` converts the `Account` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Account`.
    pub fn from_bytes(b: &[u8]) -> Result<Account> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Account` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Account`.
    pub fn from_json(s: &str) -> Result<Account> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for Account {
    const KEY_PREFIX: u8 = 2;

    type Key = Address;

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key.to_bytes());
        Ok(buf)
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
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
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
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
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
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.insert(&key, &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.create(&key, &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, stage: Stage, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        let value = value.to_bytes()?;
        store.update(&key, &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, stage: Stage, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = BTreeSet::new();

        for (k, v) in items {
            let key = <Self as Storable<S>>::key_to_bytes(stage, k)?;
            let value = v.to_bytes()?;
            let item = (key, value);
            _items.insert(item);
        }

        let items: Vec<(&[u8], &[u8])> = _items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, stage: Stage, key: &Self::Key) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, stage: Stage, keys: &[Self::Key]) -> Result<()> {
        let mut _keys = BTreeSet::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(stage, key)?;
            _keys.insert(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(store: &mut S, stage: Stage, min_time: Option<Timestamp>) -> Result<()> {
        let min_time = min_time.unwrap_or_default();

        let mut _from = Address::default();
        _from[0] = stage as u8;
        _from[1] = <Self as Storable<S>>::KEY_PREFIX;
        let from = Some(_from.to_vec());
        let from = from.as_ref().map(|from| from.as_slice());

        let mut _to = Address::default();
        _to[0] = stage as u8;
        _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
        let to = Some(_to.to_vec());
        let to = to.as_ref().map(|to| to.as_slice());

        for value in store.query(from, to, None, None)? {
            let account = Account::from_bytes(&value)?;
            if account.timestamp < min_time {
                let key = <Self as Storable<S>>::key_to_bytes(stage, &account.address)?;
                store.remove(&key)?;
            }
        }

        Ok(())
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
fn test_account_new() {
    use crate::signer::Signer;
    use crypto::ecc::ed25519::PublicKey;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();
    let value = Random::u64().unwrap();
    let mut valid_signers = Signers::new().unwrap();

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64_range(1, 11).unwrap();
        let signer = Signer { public_key, weight };

        valid_signers.add(&signer).unwrap();
    }

    let mut invalid_signers = valid_signers.clone();
    invalid_signers.threshold = invalid_signers.total_weight() + 1;

    let tx_id = Digest::random().unwrap();

    let res = Account::new(stage, &valid_signers, value, Some(tx_id));
    assert!(res.is_ok());

    let res = Account::new(stage, &invalid_signers, value, Some(tx_id));
    assert!(res.is_err());
}

#[test]
fn test_account_new_eve() {
    use crate::signer::Signer;
    use crypto::ecc::ed25519::PublicKey;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();
    let mut valid_signers = Signers::new().unwrap();

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64_range(1, 11).unwrap();
        let signer = Signer { public_key, weight };

        valid_signers.add(&signer).unwrap();
    }

    let mut invalid_signers = valid_signers.clone();
    invalid_signers.threshold = invalid_signers.total_weight() + 1;

    let res = Account::new_eve(stage, &invalid_signers);
    assert!(res.is_err());

    let res = Account::new_eve(stage, &valid_signers);
    assert!(res.is_ok());

    let mut eve_account = res.unwrap();

    let res = eve_account.is_eve();
    assert!(res.is_ok());
    assert!(res.unwrap());

    let res = eve_account.validate();
    assert!(res.is_ok());

    eve_account.value = 1;

    let res = eve_account.is_eve();
    assert!(res.is_ok());
    assert!(!res.unwrap());

    let res = eve_account.validate();
    assert!(res.is_err());

    eve_account.value = 0;

    eve_account.counter = 1;

    let res = eve_account.is_eve();
    assert!(res.is_ok());
    assert!(!res.unwrap());

    let res = eve_account.validate();
    assert!(res.is_err());
}

#[test]
fn test_account_validate() {
    use crate::signer::Signer;
    use crypto::ecc::ed25519::PublicKey;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();
    let value = Random::u64().unwrap();
    let mut valid_signers = Signers::new().unwrap();

    let mut invalid_address = Address::random().unwrap();
    while invalid_address == valid_signers.address {
        invalid_address = Address::random().unwrap();
    }

    for _ in 0..10 {
        let public_key = PublicKey::random().unwrap();
        let weight = Random::u64_range(1, 11).unwrap();
        let signer = Signer { public_key, weight };

        valid_signers.add(&signer).unwrap();
    }

    let mut invalid_signers = valid_signers.clone();
    invalid_signers.threshold = invalid_signers.total_weight() + 1;
    let tx_id = Digest::random().unwrap();

    let mut account = Account::new(stage, &valid_signers, value, Some(tx_id)).unwrap();

    let res = account.validate();
    assert!(res.is_ok());

    account.address = invalid_address;
    let res = account.validate();
    assert!(res.is_err());

    account.address = valid_signers.address;

    account.signers = invalid_signers;
    let res = account.validate();
    assert!(res.is_err());
}

#[test]
fn test_account_serialize_bytes() {
    use crypto::random::Random;

    for _ in 0..10 {
        let stage = Stage::random().unwrap();
        let signers = Signers::new().unwrap();
        let value = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let account_a = Account::new(stage, &signers, value, Some(tx_id)).unwrap();

        let res = account_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Account::from_bytes(&cbor);
        assert!(res.is_ok());
        let account_b = res.unwrap();

        assert_eq!(account_a, account_b)
    }
}

#[test]
fn test_account_serialize_json() {
    use crypto::random::Random;

    for _ in 0..10 {
        let stage = Stage::random().unwrap();
        let signers = Signers::new().unwrap();
        let value = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let account_a = Account::new(stage, &signers, value, Some(tx_id)).unwrap();

        let res = account_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Account::from_json(&json);
        assert!(res.is_ok());
        let account_b = res.unwrap();

        assert_eq!(account_a, account_b)
    }
}

#[test]
fn test_account_storable() {
    use crypto::random::Random;
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_unqlite(max_value_size, max_size).unwrap();

    let stage = Stage::random().unwrap();

    let items: Vec<(Address, Account)> = (0..10)
        .map(|_| {
            let signers = Signers::new().unwrap();
            let value = Random::u64().unwrap();
            let tx_id = Digest::random().unwrap();
            let account = Account::new(stage, &signers, value, Some(tx_id)).unwrap();
            (account.address, account)
        })
        .collect();

    for (key, value) in &items {
        let res = Account::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = Account::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = Account::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = Account::get(&store, stage, &key);
        assert!(res.is_err());

        let res = Account::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = Account::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = Account::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().iter().next(), Some(value));

        let res = Account::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = Account::get(&store, stage, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = Account::remove(&mut store, stage, &key);
        assert!(res.is_ok());

        let res = Account::count(&store, stage, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = Account::query(&store, stage, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = Account::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = Account::get(&store, stage, &key);
        assert!(res.is_err());

        let res = Account::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = Account::clear(&mut store, stage);
        assert!(res.is_ok());

        let res = Account::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);
    }
}
