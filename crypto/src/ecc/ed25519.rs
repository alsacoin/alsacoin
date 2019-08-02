//! # Ed25519
//!
//! `ed25519` is the module containint Ed25519 signature types
//! and functionalities.

use crate::error::Error;
use crate::result::Result;
use base16;
use curve25519_dalek::scalar::Scalar;
use digest::Digest;
use ed25519_dalek as ed25519;
use rand_core::{CryptoRng, RngCore};
use rand_os::OsRng;
use serde::de;
use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer};
use std::cmp;
use std::fmt;
use std::result;
use subtle::ConstantTimeEq;
use typenum::consts::U64;

// TODO: de-lame (de)serialization

/// `SECRET_KEY_LEN` is the length of a `SecretKey`.
pub const SECRET_KEY_LEN: usize = 32;

/// `PUBLIC_KEY_LEN` is the length of a `PublicKey`.
pub const PUBLIC_KEY_LEN: usize = 32;

/// `KEYPAIR_LEN` is the length of a `KeyPair`.
pub const KEYPAIR_LEN: usize = 64;

/// `SIGNATURE_LEN` is the length of a `Signature`.
pub const SIGNATURE_LEN: usize = 64;

/// `SecretKey` is an Ed25519 secret key.
#[derive(Debug)]
pub struct SecretKey(ed25519::SecretKey);

impl SecretKey {
    /// `new` creates a new random `SecretKey`.
    pub fn new() -> Result<SecretKey> {
        SecretKey::random()
    }

    /// `random` creates a random `SecretKey`.
    pub fn random() -> Result<SecretKey> {
        let mut rng = OsRng::new()?;
        SecretKey::from_rng(&mut rng)
    }

    /// `from_rng` creates a new random `SecretKey`, but requires
    /// to specify a random generator.
    pub fn from_rng<R: RngCore + CryptoRng>(rng: &mut R) -> Result<SecretKey> {
        let mut scalar = Scalar::random(rng).reduce();
        while scalar.ct_eq(&Scalar::zero()).unwrap_u8() == 1u8 {
            scalar = Scalar::random(rng).reduce();
        }

        let buf = scalar.to_bytes();
        let sk = ed25519::SecretKey::from_bytes(buf.as_ref())?;
        let secret_key = SecretKey(sk);
        Ok(secret_key)
    }

    /// `from_hash` creates a new `SecretKey` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> Result<SecretKey>
    where
        D: Digest<OutputSize = U64>,
    {
        let scalar = Scalar::from_hash(digest);
        SecretKey::from_scalar(scalar)
    }

    /// `from_scalar` creates a new `SecretKey` from a `Scalar`.
    /// The `Scalar` value cannot be 0.
    pub fn from_scalar(scalar: Scalar) -> Result<SecretKey> {
        if scalar.ct_eq(&Scalar::zero()).unwrap_u8() == 1u8 {
            let msg = "scalar is 0".into();
            let err = Error::Scalar { msg };
            return Err(err);
        }

        if !scalar.is_canonical() {
            let msg = "not canonical bytes".into();
            let err = Error::Scalar { msg };
            return Err(err);
        }

        let buf = scalar.to_bytes();
        let sk = ed25519::SecretKey::from_bytes(buf.as_ref())?;
        let secret_key = SecretKey(sk);
        Ok(secret_key)
    }

    /// `to_scalar` returns the inner `Scalar` of the `SecretKey`.
    pub fn to_scalar(&self) -> Result<Scalar> {
        let buf = self.0.to_bytes();
        if let Some(scalar) = Scalar::from_canonical_bytes(buf) {
            Ok(scalar)
        } else {
            let msg = "not canonical bytes".into();
            let err = Error::Scalar { msg };
            Err(err)
        }
    }

    /// `from_bytes` creates a new `SecretKey` from a slice of bytes.
    pub fn from_bytes(buf: [u8; SECRET_KEY_LEN]) -> Result<SecretKey> {
        if let Some(scalar) = Scalar::from_canonical_bytes(buf) {
            let secret = SecretKey::from_scalar(scalar)?;
            Ok(secret)
        } else {
            let msg = "not canonical bytes".into();
            let err = Error::Scalar { msg };
            Err(err)
        }
    }

    /// `to_bytes` returns the `SecretKey` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; SECRET_KEY_LEN] {
        self.0.to_bytes()
    }

    /// `from_slice` creates a new `SecretKey` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<SecretKey> {
        let len = buf.len();
        if len != SECRET_KEY_LEN {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut sk = [0u8; SECRET_KEY_LEN];
        sk.copy_from_slice(buf);

        SecretKey::from_bytes(sk)
    }

    /// `to_vec` converts the `Digest` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `SecretKey` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<SecretKey> {
        let len = s.len();
        if len != SECRET_KEY_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        SecretKey::from_slice(&buf)
    }

    /// `to_string` returns a `SecretKey` hex string.
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.to_bytes().as_ref())
    }

    /// `to_public` returns the `PublicKey` of the `SecretKey`.
    pub fn to_public(&self) -> PublicKey {
        let pk: ed25519::PublicKey = (&self.0).into();
        PublicKey(pk)
    }

    /// `validate` validates the `SecretKey`.
    pub fn validate(&self) -> Result<()> {
        let scalar = self.to_scalar()?;

        if !scalar.is_canonical() {
            let msg = "not canonical bytes".into();
            let err = Error::Scalar { msg };
            return Err(err);
        }

        if scalar.ct_eq(&Scalar::zero()).unwrap_u8() == 1u8 {
            let msg = "scalar is 0".into();
            let err = Error::Scalar { msg };
            return Err(err);
        }

        Ok(())
    }

    /// `sign` signs a binary message returning a `Signature`.
    /// NB: the function does not validate the `SecretKey`.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        let pk = self.to_public();
        let expanded: ed25519::ExpandedSecretKey = (&self.0).into();
        let sig = expanded.sign(msg, &pk.0);
        Signature(sig)
    }
}

impl Clone for SecretKey {
    fn clone(&self) -> SecretKey {
        // TODO: make it less lame if possible
        SecretKey::from_bytes(self.to_bytes()).unwrap()
    }
}

impl fmt::Display for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for SecretKey {
    fn eq(&self, other: &SecretKey) -> bool {
        (&self.to_bytes()).ct_eq(&other.to_bytes()).unwrap_u8() == 1u8
    }
}

impl Eq for SecretKey {}

impl PartialOrd for SecretKey {
    fn partial_cmp(&self, other: &SecretKey) -> Option<cmp::Ordering> {
        // NB: not constant-time
        Some(self.to_bytes().cmp(&other.to_bytes()))
    }
}

impl Ord for SecretKey {
    fn cmp(&self, other: &SecretKey) -> cmp::Ordering {
        // NB: not constant-time
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl Serialize for SecretKey {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = self.to_string();
        serializer.serialize_str(&hex)
    }
}

struct SecretKeyVisitor;

impl<'de> de::Visitor<'de> for SecretKeyVisitor {
    type Value = SecretKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string of length DIGEST_LEN*2")
    }

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        SecretKey::from_str(value).map_err(|e| E::custom(format!("{}", e)))
    }

    fn visit_string<E>(self, value: String) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        SecretKey::from_str(&value).map_err(|e| E::custom(format!("{}", e)))
    }
}

impl<'de> Deserialize<'de> for SecretKey {
    fn deserialize<D>(deserializer: D) -> result::Result<SecretKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SecretKeyVisitor)
    }
}

/// `PublicKey` is an Ed25519 public key.
#[derive(Copy, Clone, Debug, Default)]
pub struct PublicKey(ed25519::PublicKey);

impl PublicKey {
    /// `new` creates a new `PublicKey` from a `SecretKey`.
    pub fn new(secret: &SecretKey) -> Result<PublicKey> {
        PublicKey::from_secret(secret)
    }

    /// `from_secret` creates a new `PublicKey` from a `SecretKey`.
    pub fn from_secret(secret: &SecretKey) -> Result<PublicKey> {
        secret.validate()?;
        let pk = secret.to_public();
        Ok(pk)
    }

    /// `random` creates a new random `PublicKey`.
    pub fn random() -> Result<PublicKey> {
        let sk = SecretKey::random()?;
        PublicKey::from_secret(&sk)
    }

    /// `from_bytes` creates a new `PublicKey` from a slice of bytes.
    pub fn from_bytes(buf: [u8; PUBLIC_KEY_LEN]) -> Result<PublicKey> {
        let pk = ed25519::PublicKey::from_bytes(buf.as_ref())?;
        let public_key = PublicKey(pk);
        Ok(public_key)
    }

    /// `to_bytes` returns the `PublicKey` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LEN] {
        self.0.to_bytes()
    }

    /// `from_slice` creates a new `PublicKey` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<PublicKey> {
        let pk = ed25519::PublicKey::from_bytes(buf)?;
        let public_key = PublicKey(pk);
        Ok(public_key)
    }

    /// `to_vec` converts the `PublicKey` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `PublicKey` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<PublicKey> {
        let len = s.len();
        if len != PUBLIC_KEY_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        PublicKey::from_slice(&buf)
    }

    /// `to_string` returns a `PublicKey` hex string.
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.to_bytes().as_ref())
    }

    /// `verify` verifies a `Signature` against a binary message.
    pub fn verify(&self, sig: &Signature, msg: &[u8]) -> Result<()> {
        self.0.verify(msg, &sig.0).map_err(|e| e.into())
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for PublicKey {
    fn eq(&self, other: &PublicKey) -> bool {
        (&self.to_bytes()).ct_eq(&other.to_bytes()).unwrap_u8() == 1u8
    }
}

impl Eq for PublicKey {}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &PublicKey) -> Option<cmp::Ordering> {
        // NB: not constant-time
        Some(self.to_bytes().cmp(&other.to_bytes()))
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &PublicKey) -> cmp::Ordering {
        // NB: not constant-time
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = self.to_string();
        serializer.serialize_str(&hex)
    }
}

struct PublicKeyVisitor;

impl<'de> de::Visitor<'de> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string of length DIGEST_LEN*2")
    }

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        PublicKey::from_str(value).map_err(|e| E::custom(format!("{}", e)))
    }

    fn visit_string<E>(self, value: String) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        PublicKey::from_str(&value).map_err(|e| E::custom(format!("{}", e)))
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> result::Result<PublicKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyVisitor)
    }
}

/// `KeyPair` is a pair of Ed25519 `PublicKey` and `SecretKey`.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl KeyPair {
    /// `new` creates a new random `KeyPair`.
    pub fn new() -> Result<KeyPair> {
        let mut rng = OsRng::new()?;
        KeyPair::from_rng(&mut rng)
    }

    /// `from_rng` creates a new random `KeyPair`, but requires
    /// to specify a random generator.
    pub fn from_rng<R>(mut rng: &mut R) -> Result<KeyPair>
    where
        R: RngCore + CryptoRng,
    {
        let secret_key = SecretKey::from_rng(&mut rng)?;
        let public_key = secret_key.to_public();

        let keys = KeyPair {
            public_key,
            secret_key,
        };

        Ok(keys)
    }

    /// `from_hash` creates a new `KeyPair` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> Result<KeyPair>
    where
        D: Digest<OutputSize = U64>,
    {
        let secret_key = SecretKey::from_hash(digest)?;
        let public_key = secret_key.to_public();

        let kp = KeyPair {
            public_key,
            secret_key,
        };

        Ok(kp)
    }

    /// `from_scalar` creates a new `KeyPair` from a `Scalar`.
    pub fn from_scalar(scalar: Scalar) -> Result<KeyPair> {
        let secret_key = SecretKey::from_scalar(scalar)?;
        let public_key = secret_key.to_public();

        let keys = KeyPair {
            public_key,
            secret_key,
        };

        Ok(keys)
    }

    /// `from_secret` creates a new `KeyPair` from a `SecretKey`.
    pub fn from_secret(secret_key: &SecretKey) -> Result<KeyPair> {
        secret_key.validate()?;
        let public_key = secret_key.to_public();

        let keys = KeyPair {
            public_key,
            secret_key: secret_key.clone(),
        };

        Ok(keys)
    }

    /// `from_bytes` creates a new `KeyPair` from a bytes of bytes.
    pub fn from_bytes(buf: [u8; KEYPAIR_LEN]) -> Result<KeyPair> {
        let public_key = PublicKey::from_slice(&buf[0..PUBLIC_KEY_LEN])?;
        let secret_key = SecretKey::from_slice(&buf[SECRET_KEY_LEN..])?;

        if public_key != secret_key.to_public() {
            let msg = "keys not related".into();
            let err = Error::Keys { msg };
            return Err(err);
        }

        let keys = KeyPair {
            public_key,
            secret_key,
        };

        Ok(keys)
    }

    /// `to_bytes` returns the `KeyPair` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; KEYPAIR_LEN] {
        let mut buf = [0u8; KEYPAIR_LEN];

        for (i, b) in self.public_key.to_bytes().iter().enumerate() {
            buf[i] = *b;
        }

        for (i, b) in self.secret_key.to_bytes().iter().enumerate() {
            buf[i + PUBLIC_KEY_LEN] = *b;
        }

        buf
    }

    /// `from_slice` creates a new `KeyPair` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<KeyPair> {
        let len = buf.len();
        if len != KEYPAIR_LEN {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut kp = [0u8; KEYPAIR_LEN];
        kp.copy_from_slice(buf);

        KeyPair::from_bytes(kp)
    }

    /// `to_vec` converts the `KeyPair` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `KeyPair` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<KeyPair> {
        let len = s.len();
        if len != KEYPAIR_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        KeyPair::from_slice(&buf)
    }

    /// `to_string` returns a `KeyPair` hex string.
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.to_bytes().as_ref())
    }

    /// `validate` validates the `KeyPair`.
    pub fn validate(&self) -> Result<()> {
        if self.public_key != self.secret_key.to_public() {
            let msg = "keys not related".into();
            let err = Error::Keys { msg };
            return Err(err);
        }

        Ok(())
    }

    /// `sign` signs a binary message returning a `Signature`.
    pub fn sign(&self, msg: &[u8]) -> Result<Signature> {
        self.validate()?;

        let signature = self.secret_key.sign(msg);
        Ok(signature)
    }

    /// `verify` verifies a `Signature` against a binary message.
    pub fn verify(&self, sig: &Signature, msg: &[u8]) -> Result<()> {
        self.validate()?;

        self.public_key.verify(sig, msg)
    }
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// `Signature` is an Ed25519 signature.
#[derive(Copy, Clone, Debug)]
pub struct Signature(ed25519::Signature);

impl Signature {
    /// `new` creates a new `Signature` from a `SecretKey`.
    pub fn new(buf: [u8; SIGNATURE_LEN]) -> Result<Signature> {
        Signature::from_bytes(buf)
    }

    /// `from_bytes` creates a new `Signature` from a slice of bytes.
    pub fn from_bytes(buf: [u8; SIGNATURE_LEN]) -> Result<Signature> {
        let pk = ed25519::Signature::from_bytes(buf.as_ref())?;
        let public_key = Signature(pk);
        Ok(public_key)
    }

    /// `to_bytes` returns the `Signature` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; SIGNATURE_LEN] {
        self.0.to_bytes()
    }

    /// `from_slice` creates a new `Signature` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<Signature> {
        let pk = ed25519::Signature::from_bytes(buf)?;
        let public_key = Signature(pk);
        Ok(public_key)
    }

    /// `to_vec` converts the `Signature` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `Signature` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Signature> {
        let len = s.len();
        if len != SIGNATURE_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        Signature::from_slice(&buf)
    }

    /// `to_string` returns a `Signature` hex string.
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.to_bytes().as_ref())
    }
}

impl Default for Signature {
    fn default() -> Signature {
        Signature::new([0u8; SIGNATURE_LEN]).unwrap()
    }
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Signature {
    fn eq(&self, other: &Signature) -> bool {
        (&self.to_bytes()).ct_eq(&other.to_bytes()).unwrap_u8() == 1u8
    }
}

impl Eq for Signature {}

impl PartialOrd for Signature {
    fn partial_cmp(&self, other: &Signature) -> Option<cmp::Ordering> {
        // NB: not constant-time
        Some(self.to_bytes().cmp(&other.to_bytes()))
    }
}

impl Ord for Signature {
    fn cmp(&self, other: &Signature) -> cmp::Ordering {
        // NB: not constant-time
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = self.to_string();
        serializer.serialize_str(&hex)
    }
}

struct SignatureVisitor;

impl<'de> de::Visitor<'de> for SignatureVisitor {
    type Value = Signature;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string of length DIGEST_LEN*2")
    }

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Signature::from_str(value).map_err(|e| E::custom(format!("{}", e)))
    }

    fn visit_string<E>(self, value: String) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Signature::from_str(&value).map_err(|e| E::custom(format!("{}", e)))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> result::Result<Signature, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SignatureVisitor)
    }
}

#[test]
fn test_secret_key_serialize() {
    let res = SecretKey::random();
    assert!(res.is_ok());
    let secret_key_a = res.unwrap();

    let hex = secret_key_a.to_string();

    let res = SecretKey::from_str(&hex);
    assert!(res.is_ok());

    let secret_key_b = res.unwrap();
    assert_eq!(secret_key_a, secret_key_b)
}

#[test]
fn test_public_key_serialize() {
    let res = PublicKey::random();
    assert!(res.is_ok());
    let public_key_a = res.unwrap();

    let hex = public_key_a.to_string();

    let res = PublicKey::from_str(&hex);
    assert!(res.is_ok());

    let public_key_b = res.unwrap();
    assert_eq!(public_key_a, public_key_b)
}

#[test]
fn test_signature_serialize() {
    let buf = [0u8; SIGNATURE_LEN];

    let res = Signature::from_slice(&buf);
    assert!(res.is_ok());
    let signature_a = res.unwrap();

    let hex = signature_a.to_string();

    let res = Signature::from_str(&hex);
    assert!(res.is_ok());

    let signature_b = res.unwrap();
    assert_eq!(signature_a, signature_b)
}

#[test]
fn test_secret_key_validate() {
    use curve25519_dalek::scalar::Scalar;

    let mut rng = OsRng::new().unwrap();

    let scalar = Scalar::random(&mut rng);
    let zero = Scalar::zero();

    let res = SecretKey::from_scalar(scalar);
    if (scalar == zero) || !scalar.is_canonical() {
        assert!(res.is_err());
    } else {
        assert!(res.is_ok());
    }
}

#[test]
fn test_keypair_validate() {
    use curve25519_dalek::scalar::Scalar;

    let mut rng = OsRng::new().unwrap();

    let scalar = Scalar::random(&mut rng);
    let zero = Scalar::zero();

    let res = KeyPair::from_scalar(scalar);
    if (scalar == zero) || !scalar.is_canonical() {
        assert!(res.is_err());
    } else {
        assert!(res.is_ok());

        let secret_key = SecretKey::from_scalar(scalar).unwrap();
        let res = KeyPair::from_secret(&secret_key);
        assert!(res.is_ok());
    }
}

#[test]
fn test_keypair_sign() {
    use crate::random::Random;

    let msg_len = 1000;

    for _ in 0..10 {
        let msg = Random::bytes(msg_len).unwrap();

        let sk = SecretKey::random().unwrap();
        let pk = PublicKey::from_secret(&sk).unwrap();

        let sig = sk.sign(&msg);
        let res = pk.verify(&sig, &msg);
        assert!(res.is_ok());

        let kp = KeyPair::new().unwrap();

        let res = kp.sign(&msg);
        assert!(res.is_ok());

        let sig = res.unwrap();
        let res = kp.verify(&sig, &msg);
        assert!(res.is_ok());
    }
}
