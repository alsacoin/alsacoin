//! # Input
//!
//! `input` contains the `Input` type and functions.

use crate::account::Account;
use crate::address::Address;
use crate::error::Error;
use crate::result::Result;
use crate::transaction::Transaction;
use crypto::ecc::ed25519::{KeyPair, PublicKey, SecretKey, Signature};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// `Input` is an input in an Alsacoin `Transaction`.
#[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
pub struct Input {
    pub account: Account,
    pub signatures: BTreeMap<PublicKey, Signature>,
    pub amount: u64,
    pub distance: u64,
}

impl Input {
    /// `new` creates a new unsigned `Input`.
    pub fn new(account: &Account, distance: u64, amount: u64) -> Result<Input> {
        account.validate()?;

        if account.amount < amount {
            let err = Error::InvalidAmount;
            return Err(err);
        }

        if distance == 0 {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        let mut input = Input::default();
        input.account = account.to_owned();
        input.amount = amount;
        input.distance = distance;

        Ok(input)
    }

    /// `address` returns the `Input` address.
    pub fn address(&self) -> Address {
        self.account.address()
    }

    /// `from_transaction` creates a new `Input` from a `Transaction`.
    pub fn from_transaction(account: &Account, transaction: &Transaction) -> Result<Input> {
        account.validate()?;
        transaction.validate()?;

        if account.stage != transaction.stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        if account.time > transaction.time {
            let err = Error::InvalidTimestamp;
            return Err(err);
        }

        let output = transaction.get_output(&account.address())?;
        let distance = transaction.distance;
        let amount = output.amount;

        let account = account.update(transaction.locktime, amount, transaction.id)?;

        Input::new(&account, distance, amount)
    }

    /// `signature_message` returns the binary message to sign from a binary seed.
    pub fn signature_message(&self, seed: &[u8]) -> Result<Vec<u8>> {
        let mut clone = self.clone();
        clone.signatures = BTreeMap::default();

        let mut msg = Vec::new();
        msg.extend_from_slice(seed);
        msg.extend_from_slice(&clone.to_bytes()?);

        Ok(msg)
    }

    /// `calc_signature` calculates the input signature given a binary seed.
    pub fn calc_signature(&self, secret_key: &SecretKey, seed: &[u8]) -> Result<Signature> {
        let kp = KeyPair::from_secret(secret_key)?;

        if !self.account.signers.lookup(&kp.public_key) {
            let err = Error::NotFound;
            return Err(err);
        }

        let msg = self.signature_message(seed)?;

        kp.sign(&msg).map_err(|e| e.into())
    }

    /// `sign` signs the `Input` and update its id.
    pub fn sign(&mut self, secret_key: &SecretKey, msg: &[u8]) -> Result<()> {
        let public_key = secret_key.to_public();

        if !self.account.signers.lookup(&public_key) {
            let err = Error::NotFound;
            return Err(err);
        }

        let signature = self.calc_signature(secret_key, msg)?;

        self.signatures.insert(public_key, signature);

        Ok(())
    }

    /// `verify_signature` verifies the `Input` signature.
    pub fn verify_signature(&self, public_key: &PublicKey, seed: &[u8]) -> Result<()> {
        if !self.account.signers.lookup(&public_key) {
            let err = Error::NotFound;
            return Err(err);
        }

        let signature = self.signatures.get(public_key).unwrap();

        let msg = self.signature_message(seed)?;

        public_key.verify(&signature, &msg).map_err(|e| e.into())
    }

    /// `is_signed` returns if the `Input` has been signed by someone.
    pub fn is_signed(&self) -> bool {
        let signatures_len = self.signatures.len();
        let pks_len = self
            .signatures
            .keys()
            .filter(|pk| self.account.signers.lookup(&pk))
            .count();

        signatures_len != 0 && pks_len == signatures_len
    }

    /// `is_fully_signed` returns if the `Input` has been fully signed.
    pub fn is_fully_signed(&self) -> Result<bool> {
        let res = self.is_signed() && self.signatures_weight()? >= self.account.signers.threshold;
        Ok(res)
    }

    /// `validate` validates the `Input`.
    pub fn validate(&self) -> Result<()> {
        self.account.validate()?;

        if self.account.amount < self.amount {
            let err = Error::InvalidAmount;
            return Err(err);
        }

        if self.distance == 0 {
            let err = Error::InvalidDistance;
            return Err(err);
        }

        for pk in self.signatures.keys() {
            if !self.account.signers.lookup(&pk) {
                let err = Error::InvalidPublicKey;
                return Err(err);
            }
        }

        Ok(())
    }

    /// `verify_signatures` verifies `Input` signatures. It does not
    /// expect it to be fully signed.
    pub fn verify_signatures(&self, seed: &[u8]) -> Result<()> {
        if !self.is_signed() {
            let err = Error::NotSigned;
            return Err(err);
        }

        for pk in self.signatures.keys() {
            if !self.account.signers.lookup(&pk) {
                let err = Error::InvalidPublicKey;
                return Err(err);
            }

            self.verify_signature(&pk, seed)?;
        }

        Ok(())
    }

    /// `verify_fully_signed` verifies `Input` signatures expecting
    /// it to be fully signed.
    pub fn verify_fully_signed(&self, seed: &[u8]) -> Result<()> {
        if !self.is_fully_signed()? {
            let err = Error::NotFullySigned;
            return Err(err);
        }

        self.verify_signatures(seed)
    }

    /// `signatures_weight` returns the sum of the weights of
    /// the actual signers.
    pub fn signatures_weight(&self) -> Result<u64> {
        self.validate()?;

        let mut sigs_weight = 0;

        for pk in self.signatures.keys() {
            let signer = self.account.signers.get(&pk)?;
            sigs_weight += signer.weight;
        }

        Ok(sigs_weight)
    }

    /// `to_bytes` converts the `Input` into a CBOR binary.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        serde_cbor::to_vec(self).map_err(|e| e.into())
    }

    /// `from_bytes` converts a CBOR binary into an `Input`.
    pub fn from_bytes(b: &[u8]) -> Result<Input> {
        serde_cbor::from_slice(b).map_err(|e| e.into())
    }

    /// `to_json` converts the `Input` into a JSON string.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| e.into())
    }

    /// `from_json` converts a JSON string into an `Input`.
    pub fn from_json(s: &str) -> Result<Input> {
        serde_json::from_str(s).map_err(|e| e.into())
    }
}

#[test]
fn test_input_new() {
    use crate::signers::Signers;
    use crate::stage::Stage;
    use crypto::hash::Digest;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();
    let signers = Signers::new().unwrap();
    let amount = Random::u64().unwrap();
    let tx_id = Digest::random().unwrap();
    let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

    let mut distance = Random::u64().unwrap();
    while distance == 0 {
        distance = Random::u64().unwrap();
    }

    let res = Input::new(&account, distance, amount);
    assert!(res.is_ok());

    let input = res.unwrap();

    let res = input.validate();
    assert!(res.is_ok());
}

#[test]
fn test_input_sign() {
    use crate::signer::Signer;
    use crate::signers::Signers;
    use crate::stage::Stage;
    use crypto::hash::Digest;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();

    let secret_key_a = SecretKey::random().unwrap();
    let public_key_a = secret_key_a.to_public();
    let secret_key_b = SecretKey::random().unwrap();
    let public_key_b = secret_key_b.to_public();
    let msg_len = 1000;
    let msg = Random::bytes(msg_len).unwrap();
    let weight = 10;
    let signer_a = Signer {
        public_key: public_key_a,
        weight,
    };
    let signer_b = Signer {
        public_key: public_key_b,
        weight,
    };

    let mut signers = Signers::new().unwrap();
    signers.threshold = 20;

    signers.add(&signer_a).unwrap();
    signers.add(&signer_b).unwrap();

    let amount = Random::u64().unwrap();
    let tx_id = Digest::random().unwrap();
    let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

    let mut distance = Random::u64().unwrap();
    while distance == 0 {
        distance = Random::u64().unwrap();
    }
    let mut input = Input::new(&account, distance, amount).unwrap();

    let is_signed = input.is_signed();
    assert!(!is_signed);

    let res = input.sign(&secret_key_a, &msg);
    assert!(res.is_ok());

    let is_signed = input.is_signed();
    assert!(is_signed);

    let res = input.verify_signature(&public_key_a, &msg);
    assert!(res.is_ok());

    let res = input.signatures_weight();
    assert!(res.is_ok());

    let sigs_weight = res.unwrap();
    assert_eq!(sigs_weight, signer_a.weight);

    let res = input.is_fully_signed();
    assert!(res.is_ok());
    assert!(!res.unwrap());

    let res = input.sign(&secret_key_b, &msg);
    assert!(res.is_ok());

    let sigs_weight = input.signatures_weight().unwrap();
    assert_eq!(sigs_weight, input.account.signers.threshold);

    let res = input.is_fully_signed();
    assert!(res.is_ok());
    assert!(res.unwrap());
}

#[test]
fn test_input_validate() {
    use crate::signers::Signers;
    use crate::stage::Stage;
    use crypto::hash::Digest;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();
    let signers = Signers::new().unwrap();
    let amount = Random::u64().unwrap();
    let tx_id = Digest::random().unwrap();
    let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

    let mut distance = Random::u64().unwrap();
    while distance == 0 {
        distance = Random::u64().unwrap();
    }

    let mut input = Input::new(&account, distance, amount).unwrap();

    let res = input.validate();
    assert!(res.is_ok());

    input.distance = 0;
    let res = input.validate();
    assert!(res.is_err());

    input.distance += 1;

    let mut invalid_public_key = PublicKey::random().unwrap();
    while input.account.signers.lookup(&invalid_public_key) {
        invalid_public_key = PublicKey::random().unwrap();
    }

    let invalid_signature = Signature::default();

    input
        .signatures
        .insert(invalid_public_key, invalid_signature);
    let res = input.validate();
    assert!(res.is_err());
}

#[test]
fn test_input_serialize_bytes() {
    use crate::signers::Signers;
    use crate::stage::Stage;
    use crypto::hash::Digest;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();

    for _ in 0..10 {
        let signers = Signers::new().unwrap();
        let amount = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let input_a = Input::new(&account, distance, amount).unwrap();

        let res = input_a.to_bytes();
        assert!(res.is_ok());
        let cbor = res.unwrap();

        let res = Input::from_bytes(&cbor);
        assert!(res.is_ok());
        let input_b = res.unwrap();

        assert_eq!(input_a, input_b)
    }
}

#[test]
fn test_input_serialize_json() {
    use crate::signers::Signers;
    use crate::stage::Stage;
    use crypto::hash::Digest;
    use crypto::random::Random;

    let stage = Stage::random().unwrap();

    for _ in 0..10 {
        let signers = Signers::new().unwrap();
        let amount = Random::u64().unwrap();
        let tx_id = Digest::random().unwrap();
        let account = Account::new(stage, &signers, amount, Some(tx_id)).unwrap();

        let mut distance = Random::u64().unwrap();
        while distance == 0 {
            distance = Random::u64().unwrap();
        }

        let input_a = Input::new(&account, distance, amount).unwrap();

        let res = input_a.to_json();
        assert!(res.is_ok());
        let json = res.unwrap();

        let res = Input::from_json(&json);
        assert!(res.is_ok());
        let input_b = res.unwrap();

        assert_eq!(input_a, input_b)
    }
}
