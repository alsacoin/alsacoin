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
    const KEY_PREFIX: u32 = 2;

    type Key = Address;

    fn lookup(&self, _store: &S, _key: &Self::Key) -> Result<bool> {
        // TODO
        unreachable!()
    }

    fn get(&self, _store: &S, _key: &Self::Key) -> Result<Self> {
        // TODO
        unreachable!()
    }

    fn query(
        &self,
        _store: &S,
        _from: Option<&Self::Key>,
        _to: Option<&Self::Key>,
        _count: Option<u32>,
        _skip: Option<u32>,
    ) -> Result<Vec<Self>> {
        // TODO
        unreachable!()
    }

    fn count(
        &self,
        _store: &S,
        _from: Option<&Self::Key>,
        _to: Option<&Self::Key>,
        _skip: Option<u32>,
    ) -> Result<u32> {
        // TODO
        unreachable!()
    }

    fn insert(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn create(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn update(&mut self, _store: &mut S, _key: &Self::Key, _value: &Self) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn insert_batch(&mut self, _store: &mut S, _items: &[(Self::Key, Self)]) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn remove(&mut self, _store: &mut S, _key: &Self::Key) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn remove_batch(&mut self, _store: &mut S, _keys: &[Self::Key]) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn cleanup(&mut self, _store: &mut S) -> Result<()> {
        // TODO
        unreachable!()
    }

    fn clear(&mut self, _store: &mut S) -> Result<()> {
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
