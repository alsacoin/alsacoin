//! # Wallet
//!
//! `wallet` contains the `Wallet` type and functions.

use crate::error::Error;
use crate::result::Result;
use crate::signer::Signer;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crypto::ecc::ed25519::{KeyPair, PublicKey, SecretKey, Signature, PUBLIC_KEY_LEN};
use crypto::hash::{Blake512Hasher, Digest};
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use std::collections::BTreeSet;
use store::traits::Store;

/// `Wallet` is the type used to represent an Alsacoin wallet
/// of a user, wallet which is identified by a `PublicKey`.
/// NB: ehrg on the use of vecs. It may not be a problem. It may.
/// NB: good for a PoC, should put more thinking after though.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct Wallet {
    pub public_key: Vec<u8>,
    pub secret_key: Vec<u8>,
    pub stage: Stage,
    pub time: Timestamp,
    pub checksum: Digest,
}

impl Wallet {
    /// `new` creates a new `Wallet`.
    pub fn new(stage: Stage) -> Result<Wallet> {
        let keypair = KeyPair::new()?;
        Wallet::from_keypair(stage, &keypair)
    }

    /// `from_secret` creates a new `Wallet` from a `SecretKey`.
    pub fn from_secret(stage: Stage, secret_key: SecretKey) -> Result<Wallet> {
        let keypair = KeyPair::from_secret(&secret_key)?;
        Wallet::from_keypair(stage, &keypair)
    }

    /// `from_keypair` creates a new `Wallet` from a `KeyPair`.
    pub fn from_keypair(stage: Stage, keypair: &KeyPair) -> Result<Wallet> {
        keypair.validate()?;

        let time = Timestamp::now();
        let checksum = Digest::default();

        let mut wallet = Wallet {
            public_key: keypair.public_key.to_vec(),
            secret_key: keypair.secret_key.to_vec(),
            stage,
            time,
            checksum,
        };

        wallet.update_checksum()?;

        Ok(wallet)
    }

    /// `update_checksum` updates the `Wallet` checksum.
    pub fn update_checksum(&mut self) -> Result<()> {
        self.checksum = self.calc_checksum()?;

        Ok(())
    }

    /// `calc_checksum` calculates the `Wallet` checksum.
    pub fn calc_checksum(&self) -> Result<Digest> {
        let mut clone = self.clone();
        clone.checksum = Digest::default();

        let buf = clone.to_bytes()?;
        let digest = Blake512Hasher::hash(&buf);

        Ok(digest)
    }

    /// `sign` signs a binary message with the `Wallet`.
    pub fn sign(&self, msg: &[u8]) -> Result<Signature> {
        let public_key = PublicKey::from_slice(&self.public_key)?;
        let secret_key = SecretKey::from_slice(&self.secret_key)?;

        let keypair = KeyPair {
            public_key,
            secret_key,
        };
        keypair.validate()?;

        keypair.sign(msg).map_err(|e| e.into())
    }

    /// `validate_signature` validates a `Signature` against the `Wallet` and a binary message.
    pub fn validate_signature(&self, sig: &Signature, msg: &[u8]) -> Result<()> {
        let public_key = PublicKey::from_slice(&self.public_key)?;
        let secret_key = SecretKey::from_slice(&self.secret_key)?;

        let keypair = KeyPair {
            public_key,
            secret_key,
        };
        keypair.validate()?;

        keypair.verify(sig, msg).map_err(|e| e.into())
    }

    /// `to_signer` returns a `Wallet` `Signer`.
    pub fn to_signer(&self, weight: u64) -> Result<Signer> {
        self.validate()?;

        let public_key = PublicKey::from_slice(&self.public_key)?;

        let signer = Signer { public_key, weight };

        Ok(signer)
    }

    /// `validate` validates the `Wallet`.
    pub fn validate(&self) -> Result<()> {
        let public_key = PublicKey::from_slice(&self.public_key)?;
        let secret_key = SecretKey::from_slice(&self.secret_key)?;

        let keypair = KeyPair {
            public_key,
            secret_key,
        };
        keypair.validate()?;

        self.time.validate()?;

        if self.checksum != self.calc_checksum()? {
            let err = Error::InvalidChecksum;
            return Err(err);
        }

        Ok(())
    }

    /// `to_bytes` converts the `Wallet` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Wallet`.
    pub fn from_bytes(b: &[u8]) -> Result<Wallet> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Wallet` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Wallet`.
    pub fn from_json(s: &str) -> Result<Wallet> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for Wallet {
    const KEY_PREFIX: u8 = 8;

    type Key = Vec<u8>;

    fn key_to_bytes(stage: Stage, key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(stage as u8);
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key);
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

        let mut _from = Vec::new();
        _from.extend_from_slice(&[0u8; PUBLIC_KEY_LEN]);
        _from[0] = stage as u8;
        _from[1] = <Self as Storable<S>>::KEY_PREFIX;
        let from = Some(_from.to_vec());
        let from = from.as_ref().map(|from| from.as_slice());

        let mut _to = Vec::new();
        _to.extend_from_slice(&[0u8; PUBLIC_KEY_LEN]);
        _to[0] = stage as u8;
        _to[1] = <Self as Storable<S>>::KEY_PREFIX + 1;
        let to = Some(_to.to_vec());
        let to = to.as_ref().map(|to| to.as_slice());

        for value in store.query(from, to, None, None)? {
            let wallet = Wallet::from_bytes(&value)?;
            if wallet.time < min_time {
                let key = <Self as Storable<S>>::key_to_bytes(stage, &wallet.public_key)?;
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
fn test_wallet_from_keypair() {
    let stage = Stage::default();

    let mut keypair = KeyPair::new().unwrap();
    let valid_secret = keypair.secret_key.clone();

    let res = Wallet::from_keypair(stage, &keypair);
    assert!(res.is_ok());

    while keypair.secret_key == valid_secret {
        keypair.secret_key = SecretKey::random().unwrap();
    }

    let res = Wallet::from_keypair(stage, &keypair);
    assert!(res.is_err());
}

#[test]
fn test_wallet_sign() {
    use crypto::random::Random;

    let stage = Stage::default();

    let mut wallet = Wallet::new(stage).unwrap();
    let valid_secret = wallet.secret_key.clone();

    let msg_len = 1000;
    let msg = Random::bytes(msg_len).unwrap();

    let res = wallet.sign(&msg);
    assert!(res.is_ok());

    let sig = res.unwrap();

    let res = wallet.validate_signature(&sig, &msg);
    assert!(res.is_ok());

    while wallet.secret_key == valid_secret {
        wallet.secret_key = SecretKey::random().unwrap().to_vec();
    }

    let res = wallet.sign(&msg);
    assert!(res.is_err());

    let mut other_wallet = Wallet::new(stage).unwrap();

    while other_wallet == wallet {
        other_wallet = Wallet::new(stage).unwrap();
    }

    let res = other_wallet.validate_signature(&sig, &msg);
    assert!(res.is_err());
}

#[test]
fn test_wallet_to_signer() {
    let stage = Stage::default();
    let weight = 10;

    let mut wallet = Wallet::new(stage).unwrap();
    let valid_secret = wallet.secret_key.clone();

    let res = wallet.to_signer(weight);
    assert!(res.is_ok());

    while wallet.secret_key == valid_secret {
        wallet.secret_key = SecretKey::random().unwrap().to_vec();
    }

    let res = wallet.to_signer(weight);
    assert!(res.is_err());
}

#[test]
fn test_wallet_validate() {
    let stage = Stage::default();

    let mut wallet = Wallet::new(stage).unwrap();
    let valid_secret = wallet.secret_key.clone();

    let res = wallet.validate();
    assert!(res.is_ok());

    while wallet.secret_key == valid_secret {
        wallet.secret_key = SecretKey::random().unwrap().to_vec();
    }

    let res = wallet.validate();
    assert!(res.is_err());
}

#[test]
fn test_wallet_serialize_bytes() {
    let stage = Stage::default();

    for _ in 0..10 {
        let wallet_a = Wallet::new(stage).unwrap();

        let res = wallet_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Wallet::from_bytes(&cbor);
        assert!(res.is_ok());
        let wallet_b = res.unwrap();

        assert_eq!(wallet_a, wallet_b)
    }
}

#[test]
fn test_wallet_serialize_json() {
    let stage = Stage::default();

    for _ in 0..10 {
        let wallet_a = Wallet::new(stage).unwrap();

        let res = wallet_a.to_json();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Wallet::from_json(&cbor);
        assert!(res.is_ok());
        let wallet_b = res.unwrap();

        assert_eq!(wallet_a, wallet_b)
    }
}

#[test]
fn test_wallet_storable() {
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1 << 10;
    let max_size = 1 << 30;

    let mut store = MemoryStoreFactory::new_unqlite(max_value_size, max_size).unwrap();

    let stage = Stage::random().unwrap();

    let items: Vec<(Vec<u8>, Wallet)> = (0..10)
        .map(|_| {
            let wallet = Wallet::new(stage).unwrap();
            (wallet.clone().public_key, wallet)
        })
        .collect();

    for (key, value) in &items {
        let res = Wallet::count(&store, stage, Some(key.to_vec()), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = Wallet::query(&store, stage, Some(key.to_vec()), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = Wallet::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = Wallet::get(&store, stage, &key);
        assert!(res.is_err());

        let res = Wallet::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = Wallet::count(&store, stage, Some(key.to_vec()), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = Wallet::query(&store, stage, Some(key.to_vec()), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().iter().next(), Some(value));

        let res = Wallet::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = Wallet::get(&store, stage, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = Wallet::remove(&mut store, stage, &key);
        assert!(res.is_ok());

        let res = Wallet::count(&store, stage, Some(key.to_vec()), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = Wallet::query(&store, stage, Some(key.to_vec()), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 0);

        let res = Wallet::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = Wallet::get(&store, stage, &key);
        assert!(res.is_err());

        let res = Wallet::insert(&mut store, stage, &key, &value);
        assert!(res.is_ok());

        let res = Wallet::clear(&mut store, stage);
        assert!(res.is_ok());

        let res = Wallet::lookup(&store, stage, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);
    }
}
