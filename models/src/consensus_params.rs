//! # Consensus Params
//!
//! `consensus_params` is the type used to manage the parameters of the Avalanche Consensus algorithm.

use crate::error::Error;
use crate::result::Result;
use crate::stage::Stage;
use crate::timestamp::Timestamp;
use crate::traits::Storable;
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use serde::{Deserialize, Serialize};
use serde_cbor;
use serde_json;
use store::traits::Store;

/// `ConsensusParams` represents the Avalanche Consensus parameters.
#[derive(Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct ConsensusParams {
    pub id: Digest,
    pub stage: Stage,
    pub alfa: u64,
    pub beta1: Option<u64>,
    pub beta2: Option<u64>,
}

impl ConsensusParams {
    /// `new` creates a new `ConsensusParams`.
    pub fn new(
        stage: Stage,
        alfa: u64,
        beta1: Option<u64>,
        beta2: Option<u64>,
    ) -> Result<ConsensusParams> {
        let mut params = ConsensusParams::default();
        params.stage = stage;
        params.alfa = alfa;
        params.beta1 = beta1;
        params.beta2 = beta2;

        params.id = params.calc_id()?;

        Ok(params)
    }

    /// `random` creates a random `ConsensusParams`.
    pub fn random() -> Result<ConsensusParams> {
        let stage = Stage::random()?;

        let alfa = Random::u64()?;

        let beta1 = if Random::u32_range(0, 2)? == 1 {
            Some(Random::u64()?)
        } else {
            None
        };

        let beta2 = if Random::u32_range(0, 2)? == 1 {
            Some(Random::u64()?)
        } else {
            None
        };

        ConsensusParams::new(stage, alfa, beta1, beta2)
    }

    /// `calc_id` calculates the id of the `ConsensusParams`.
    pub fn calc_id(&self) -> Result<Digest> {
        let mut clone = self.clone();
        clone.id = Digest::default();

        let buf = clone.to_bytes()?;
        let id = Blake512Hasher::hash(&buf);

        Ok(id)
    }

    /// `validate` validates the `ConsensusParams`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id()? {
            let err = Error::InvalidId;
            return Err(err);
        }

        Ok(())
    }

    /// `to_bytes` converts the `ConsensusParams` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `ConsensusParams`.
    pub fn from_bytes(b: &[u8]) -> Result<ConsensusParams> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `ConsensusParams` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `ConsensusParams`.
    pub fn from_json(s: &str) -> Result<ConsensusParams> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

impl<S: Store> Storable<S> for ConsensusParams {
    const KEY_PREFIX: u8 = 4;

    type Key = Digest;

    fn key_to_bytes(key: &Self::Key) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.push(<Self as Storable<S>>::KEY_PREFIX);
        buf.extend_from_slice(&key.to_bytes());
        Ok(buf)
    }

    fn lookup(store: &S, key: &Self::Key) -> Result<bool> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        store.lookup(&key).map_err(|e| e.into())
    }

    fn get(store: &S, key: &Self::Key) -> Result<Self> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let buf = store.get(&key)?;
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
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.insert(&key, &value).map_err(|e| e.into())
    }

    fn create(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.create(&key, &value).map_err(|e| e.into())
    }

    fn update(store: &mut S, key: &Self::Key, value: &Self) -> Result<()> {
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        let value = value.to_bytes()?;
        store.update(&key, &value).map_err(|e| e.into())
    }

    fn insert_batch(store: &mut S, items: &[(Self::Key, Self)]) -> Result<()> {
        let mut _items = Vec::new();

        for (k, v) in items {
            let key = <Self as Storable<S>>::key_to_bytes(k)?;
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
        let key = <Self as Storable<S>>::key_to_bytes(key)?;
        store.remove(&key).map_err(|e| e.into())
    }

    fn remove_batch(store: &mut S, keys: &[Self::Key]) -> Result<()> {
        let mut _keys = Vec::new();
        for key in keys {
            let key = <Self as Storable<S>>::key_to_bytes(key)?;
            _keys.push(key);
        }

        let keys: Vec<&[u8]> = _keys.iter().map(|k| k.as_slice()).collect();

        store.remove_batch(&keys).map_err(|e| e.into())
    }

    fn cleanup(_store: &mut S, _min_time: Timestamp) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn clear(store: &mut S) -> Result<()> {
        let from = Some(vec![<Self as Storable<S>>::KEY_PREFIX]);
        let from = from.as_ref().map(|from| from.as_slice());
        let to = Some(vec![<Self as Storable<S>>::KEY_PREFIX + 1]);
        let to = to.as_ref().map(|to| to.as_slice());
        store.remove_range(from, to, None).map_err(|e| e.into())
    }
}

#[test]
fn test_consensus_params_validate() {
    for _ in 0..10 {
        let mut params = ConsensusParams::random().unwrap();
        let res = params.validate();
        assert!(res.is_ok());

        params.id = Digest::default();
        let res = params.validate();
        assert!(res.is_err());
    }
}

#[test]
fn test_consensus_params_serialize_bytes() {
    for _ in 0..10 {
        let params_a = ConsensusParams::random().unwrap();

        let res = params_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = ConsensusParams::from_bytes(&cbor);
        assert!(res.is_ok());
        let params_b = res.unwrap();

        assert_eq!(params_a, params_b)
    }
}

#[test]
fn test_consensus_params_serialize_json() {
    for _ in 0..10 {
        let params_a = ConsensusParams::random().unwrap();

        let res = params_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = ConsensusParams::from_json(&json);
        assert!(res.is_ok());
        let params_b = res.unwrap();

        assert_eq!(params_a, params_b)
    }
}

#[test]
fn test_consensus_params_storable() {
    use store::memory::MemoryStoreFactory;

    let max_value_size = 1000;
    let mut store = MemoryStoreFactory::new_unqlite(max_value_size).unwrap();

    let items: Vec<(Digest, ConsensusParams)> = (0..10)
        .map(|_| {
            let params = ConsensusParams::random().unwrap();
            (params.id, params)
        })
        .collect();

    for (key, value) in &items {
        let res = ConsensusParams::count(&store, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConsensusParams::query(&store, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![]);

        let res = ConsensusParams::lookup(&store, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConsensusParams::get(&store, &key);
        assert!(res.is_err());

        let res = ConsensusParams::insert(&mut store, &key, &value);
        assert!(res.is_ok());

        let res = ConsensusParams::count(&store, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = ConsensusParams::query(&store, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![value.to_owned()]);

        let res = ConsensusParams::lookup(&store, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(found);

        let res = ConsensusParams::get(&store, &key);
        assert!(res.is_ok());
        assert_eq!(&res.unwrap(), value);

        let res = ConsensusParams::remove(&mut store, &key);
        assert!(res.is_ok());

        let res = ConsensusParams::count(&store, Some(*key), None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 0);

        let res = ConsensusParams::query(&store, Some(*key), None, None, None);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![]);

        let res = ConsensusParams::lookup(&store, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);

        let res = ConsensusParams::get(&store, &key);
        assert!(res.is_err());

        let res = ConsensusParams::insert(&mut store, &key, &value);
        assert!(res.is_ok());

        let res = ConsensusParams::clear(&mut store);
        assert!(res.is_ok());

        let res = ConsensusParams::lookup(&store, &key);
        assert!(res.is_ok());
        let found = res.unwrap();
        assert!(!found);
    }
}
