//! # Account
//!
//! `account` contains the `Account` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crate::signers::Signers;
use crate::traits::Storable;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use store::traits::Store;

/// `Account` is the type used to represent an Alsacoin account
/// of a user, account which is identified by an `Address`.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub signers: Signers,
    pub value: u64, // NB: gonna be confidential
    pub counter: u64,
}

impl Account {
    /// `new` creates a new `Account`.
    pub fn new(signers: &Signers, value: u64) -> Result<Account> {
        signers.validate()?;

        let account = Account {
            address: signers.address,
            signers: signers.to_owned(),
            value,
            counter: 0,
        };

        Ok(account)
    }

    /// `update` updates the `Account` with a new value.
    pub fn update(&mut self, value: u64) {
        self.value = value;
        self.counter += 1;
    }

    /// `validate` validates the `Account`.
    pub fn validate(&self) -> Result<()> {
        self.signers.validate()?;

        if self.address != self.signers.address {
            let err = Error::InvalidAddress;
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

    fn key_to_bytes(key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key.to_bytes());
        Ok(buf)
    }

    fn lookup(store: &S, key: &Self::Key) -> Result<bool> {
        store.lookup(&key.to_bytes()).map_err(|e| e.into())
    }

    fn get(store: &S, key: &Self::Key) -> Result<Self> {
        let buf = store.get(&key.to_bytes())?;
        Self::from_bytes(&buf)
    }

    fn query(
        store: &S,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        count: Option<u32>,
        skip: Option<u32>,
    ) -> Result<Vec<Self>> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        let values = store.query(from, to, count, skip)?;
        let mut items = Vec::new();

        for value in values {
            let item = Self::from_bytes(&value)?;
            items.push(item);
        }

        Ok(items)
    }

    fn count(
        store: &S,
        from: Option<Self::Key>,
        to: Option<Self::Key>,
        skip: Option<u32>,
    ) -> Result<u32> {
        let from = if let Some(ref key) = from {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let to = if let Some(ref key) = to {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            Some(key)
        } else {
            None
        };

        let from = from.as_ref().map(|from| from.as_slice());
        let to = to.as_ref().map(|to| to.as_slice());
        store.count(from, to, skip).map_err(|e| e.into())
    }

    fn insert(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let value = value.to_bytes()?;
        store.insert(&key.to_bytes(), &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let value = value.to_bytes()?;
        store.create(&key.to_bytes(), &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let value = value.to_bytes()?;
        store.update(&key.to_bytes(), &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = Vec::new();

        for (k, v) in items {
            let mut key = Vec::new();
            key.extend_from_slice(&k.to_bytes());

            let value = v.to_bytes()?;
            let item = (key, value);
            _items.push(item);
        }

        let items: Vec<(&[u8], &[u8])> = _items
            .iter()
            .map(|(k, v)| (k.as_slice(), v.as_slice()))
            .collect();

        store.insert_batch(&items).map_err(|e| e.into())
    }

    fn remove(store: &mut S, key: &Self::Key) -> Result<()> {
        store.remove(&key.to_bytes()).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, keys: &[Self::Key]) -> Result<()> {
        let mut _keys: Vec<Vec<u8>> = Vec::new();

        for key in keys {
            let mut k = Vec::new();
            k.extend_from_slice(&key.to_bytes()[..]);
            _keys.push(k);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(_store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn clear(_store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
    }
}

#[test]
fn test_account_new() {
    use crate::signer::Signer;
    use crypto::ecc::ed25519::PublicKey;
    use crypto::random::Random;

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

    let res = Account::new(&valid_signers, value);
    assert!(res.is_ok());

    let res = Account::new(&invalid_signers, value);
    assert!(res.is_err());
}

#[test]
fn test_account_validate() {
    use crate::signer::Signer;
    use crypto::ecc::ed25519::PublicKey;
    use crypto::random::Random;

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

    let mut account = Account::new(&valid_signers, value).unwrap();

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
        let signers = Signers::new().unwrap();
        let value = Random::u64().unwrap();
        let account_a = Account::new(&signers, value).unwrap();

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
        let signers = Signers::new().unwrap();
        let value = Random::u64().unwrap();
        let account_a = Account::new(&signers, value).unwrap();

        let res = account_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Account::from_json(&json);
        assert!(res.is_ok());
        let account_b = res.unwrap();

        assert_eq!(account_a, account_b)
    }
}
