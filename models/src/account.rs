//! # Account
//!
//! `account` contains the `Account` type and functions.

use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crypto::hash::Blake512Hasher;
use crypto::hash::Digest;
use crypto::random::Random;
use crate::signers::Signers;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;

/// `Account` is the type used to represent an Alsacoin account
/// of a user, account which is identified by an `Address`.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Account {
    pub address: Address,
    pub prev_id: Option<Digest>,
    pub counter: u64,
    pub signers: Signers,
    pub value: u64, // NB: gonna be confidential
}

impl Account {
    /// `new` creates a new `Account`.
    pub fn new(address: Address, value: u64) -> Result<Account> {
        let mut account = Account {
            id: Digest::default(),
            prev_id: None,
            counter: 0,
            address,
            value,
        };

        account.id = account.calc_id()?;

        Ok(account)
    }

    /// `from_prev` creates a new `Account` that updates
    /// a previous one.
    pub fn from_prev(prev: &Account, value: u64) -> Result<Account> {
        prev.validate()?;

        let mut account = Account {
            id: Digest::default(),
            prev_id: Some(prev.id),
            counter: prev.counter + 1,
            address: prev.address,
            value,
        };

        account.id = account.calc_id()?;

        Ok(account)
    }

    /// `random` creates a new random `Account`.
    pub fn random() -> Result<Account> {
        let (prev_id, counter) = if Random::u32_range(0, 2)? == 1 {
            let prev_id = Some(Digest::random()?);
            let mut counter = Random::u64()?;
            if counter == 0 {
                counter += 1;
            }

            (prev_id, counter)
        } else {
            (None, 0)
        };

        let mut account = Account {
            id: Digest::default(),
            prev_id,
            counter,
            address: Address::random()?,
            value: Random::u64()?,
        };

        account.id = account.calc_id()?;

        Ok(account)
    }

    /// `update` creates a new `Account` by updating this `Account`.
    pub fn update(&self, value: u64) -> Result<Account> {
        Account::from_prev(self, value)
    }

    /// `validate` validates the `Account`.
    pub fn validate(&self) -> Result<()> {
        if let Some(prev_id) = self.prev_id {
            if self.id == prev_id {
                let err = Error::InvalidId;
                return Err(err);
            }
        }

        if self.id != self.calc_id()? {
            let err = Error::InvalidId;
            return Err(err);
        }

        Ok(())
    }

    /// `calc_id` calculates the id of the `Account`.
    pub fn calc_id(&self) -> Result<Digest> {
        let mut copy = *self;
        copy.id = Digest::default();

        let buf = copy.to_bytes()?;
        let id = Blake512Hasher::hash(&buf);
        Ok(id)
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

#[test]
fn test_account_new() {
    let address = Address::random().unwrap();
    let value = Random::u64().unwrap();
    let res = Account::new(address, value);
    assert!(res.is_ok());

    let account = res.unwrap();
    let res = account.validate();
    assert!(res.is_ok());
}

#[test]
fn test_account_from_prev() {
    for _ in 0..10 {
        let res = Account::random();
        assert!(res.is_ok());

        let valid_prev = res.unwrap();
        let value = Random::u64().unwrap();
        let res = Account::from_prev(&valid_prev, value);
        assert!(res.is_ok());

        let mut invalid_prev = valid_prev;

        if let Some(prev_id) = valid_prev.prev_id {
            invalid_prev.id = prev_id;
            let res = Account::from_prev(&invalid_prev, value);
            assert!(res.is_err());
        }

        while invalid_prev.id == valid_prev.id {
            invalid_prev.id = Digest::random().unwrap();
        }

        let res = Account::from_prev(&invalid_prev, value);
        assert!(res.is_err());
    }
}

#[test]
fn test_account_validate() {
    for _ in 0..10 {
        let res = Account::random();
        assert!(res.is_ok());

        let valid_account = res.unwrap();
        let res = valid_account.validate();
        assert!(res.is_ok());

        let mut invalid_account = valid_account;

        if let Some(prev_id) = valid_account.prev_id {
            invalid_account.id = prev_id;
            let res = invalid_account.validate();
            assert!(res.is_err());
        }

        while invalid_account.id == valid_account.id {
            invalid_account.id = Digest::random().unwrap();
        }

        let res = invalid_account.validate();
        assert!(res.is_err());
    }
}

#[test]
fn test_account_serialize_bytes() {
    for _ in 0..10 {
        let account_a = Account::random().unwrap();

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
    for _ in 0..10 {
        let account_a = Account::random().unwrap();

        let res = account_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Account::from_json(&json);
        assert!(res.is_ok());
        let account_b = res.unwrap();

        assert_eq!(account_a, account_b)
    }
}
