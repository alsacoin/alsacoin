//! # ElGamal
//!
//! `elgamal` is the module containint ElGamal encryption types
//! and functionalities.

use crate::error::Error;
use crate::result::Result;
use base16;
use curve25519_dalek::constants::{BASEPOINT_ORDER, RISTRETTO_BASEPOINT_TABLE};
use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use curve25519_dalek::scalar::Scalar;
use digest::Digest;
use rand_core::{CryptoRng, RngCore};
use rand_os::OsRng;
use std::fmt;
use std::ops::{Add, Mul};
use subtle::ConstantTimeEq;
use typenum::consts::U64;

/// `MESSAGE_LEN` is the length of a `Message`.
pub const MESSAGE_LEN: usize = 32;

/// `SECRET_KEY_LEN` is the length of a `SecretKey`.
pub const SECRET_KEY_LEN: usize = 32;

/// `PUBLIC_KEY_LEN` is the length of a `PublicKey`.
pub const PUBLIC_KEY_LEN: usize = 32;

/// `KEYPAIR_LEN` is the length of a `CypherText`.
pub const KEYPAIR_LEN: usize = 64;

/// `CYPHERTEXT_LEN` is the length of a `CypherText`.
pub const CYPHERTEXT_LEN: usize = 64;

/// `GAMMA_LEN` is the length of the gamma member of a `CypherText`.
pub const GAMMA_LEN: usize = 32;

/// `DELTA_LEN` is the length of the delta member of a `CypherText`.
pub const DELTA_LEN: usize = 32;

/// `Message` is an ElGamal message.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct Message([u8; MESSAGE_LEN]);

impl Message {
    /// `new` creates a new `Message` from a slice of bytes.
    pub fn new(msg: [u8; MESSAGE_LEN]) -> Message {
        Self::from_bytes(msg)
    }

    /// `random` creates a new random `Message`.
    pub fn random() -> Result<Message> {
        let mut rng = OsRng::new()?;
        let msg = Message::from_rng(&mut rng);
        Ok(msg)
    }

    /// `from_rng` creates a new random `Message`, but requires
    /// to specify a random generator.
    pub fn from_rng<R: RngCore + CryptoRng>(rng: &mut R) -> Message {
        let point = RistrettoPoint::random(rng).compress();
        Message::from_point(&point)
    }

    /// `from_hash` creates a new `Message` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> Message
    where
        D: Digest<OutputSize = U64> + Default,
    {
        let point = RistrettoPoint::from_hash(digest).compress();
        Message::from_point(&point)
    }

    /// `from_point` creates a new `Message` from a `CompressedRistretto`.
    pub fn from_point(point: &CompressedRistretto) -> Message {
        Message(point.to_bytes())
    }

    /// `to_point` returns the inner `CompressedRistretto` of the `Message`.
    pub fn to_point(&self) -> CompressedRistretto {
        CompressedRistretto::from_slice(&self.0[..])
    }

    /// `from_bytes` creates a new `Message` from an array of bytes.
    pub fn from_bytes(buf: [u8; MESSAGE_LEN]) -> Message {
        Message(buf)
    }

    /// `to_bytes` returns the `Message` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; MESSAGE_LEN] {
        self.0
    }

    /// `from_slice` creates a new `Message` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<Message> {
        let len = buf.len();
        if len != MESSAGE_LEN {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut m = [0u8; MESSAGE_LEN];
        m.copy_from_slice(buf);

        let msg = Message::from_bytes(m);
        Ok(msg)
    }

    /// `to_vec` converts the `Message` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `Message` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Message> {
        let len = s.len();
        if len != MESSAGE_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        Message::from_slice(&buf)
    }

    /// `to_string` returns a `Message` hex string.
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.0.as_ref())
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Mul<SecretKey> for Message {
    type Output = Option<Message>;

    fn mul(self, sk: SecretKey) -> Option<Message> {
        if let Some(point) = self.to_point().decompress() {
            let scalar = sk.to_scalar();
            let point = (scalar * point).compress();
            let msg = Message::from_point(&point);
            Some(msg)
        } else {
            None
        }
    }
}

impl Add<Message> for Message {
    type Output = Option<Message>;

    fn add(self, other: Message) -> Option<Message> {
        if let Some(point) = self.to_point().decompress() {
            if let Some(other_point) = other.to_point().decompress() {
                let point = (point + other_point).compress();
                let msg = Message::from_point(&point);
                Some(msg)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// `SecretKey` is an ElGamal secret key. It's just a
/// wrapper around `Scalar`. The key is just an integer
/// between 1 and q-1, where q is the order of the group
/// G.
#[derive(Copy, Clone, Debug)]
pub struct SecretKey(Scalar);

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

        let secret = SecretKey(scalar);
        Ok(secret)
    }

    /// `from_hash` creates a new `SecretKey` from a 64 bytes hash.
    pub fn from_hash<D>(digest: D) -> SecretKey
    where
        D: Digest<OutputSize = U64>,
    {
        let scalar = Scalar::from_hash(digest).reduce();
        SecretKey(scalar)
    }

    /// `from_scalar` creates a new `SecretKey` from a `Scalar`.
    /// The `Scalar` value cannot be 0.
    pub fn from_scalar(scalar: Scalar) -> Result<SecretKey> {
        if scalar.ct_eq(&Scalar::zero()).unwrap_u8() == 1u8 {
            let msg = "scalar is 0".into();
            let err = Error::Scalar { msg };
            return Err(err);
        }

        let secret = SecretKey(scalar);
        Ok(secret)
    }

    /// `to_scalar` returns the inner `Scalar` of the `SecretKey`.
    pub fn to_scalar(&self) -> Scalar {
        self.0
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
        let point = &RISTRETTO_BASEPOINT_TABLE * &self.0;
        PublicKey(point.compress())
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

impl Add<SecretKey> for SecretKey {
    type Output = Option<SecretKey>;

    fn add(self, other: SecretKey) -> Option<SecretKey> {
        let scalar = self.to_scalar();
        let other_scalar = other.to_scalar();

        let scalar = scalar + other_scalar;

        match SecretKey::from_scalar(scalar) {
            Ok(sk) => Some(sk),
            Err(_) => None,
        }
    }
}

/// `PublicKey` is an ElGamal public key. It's just a
/// wrapper around `CompressedRistretto`.
/// The key is computed as g^x, where g is the generator
/// of the group G of order q, and x a `SecretKey`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct PublicKey(CompressedRistretto);

impl PublicKey {
    /// `new` creates a new `PublicKey` from a `SecretKey`.
    pub fn new(secret: SecretKey) -> PublicKey {
        PublicKey::from_secret(secret)
    }

    /// `from_secret` creates a new `PublicKey` from a `SecretKey`.
    pub fn from_secret(secret: SecretKey) -> PublicKey {
        secret.to_public()
    }

    /// `random` creates a new random `PublicKey`.
    pub fn random() -> Result<PublicKey> {
        let sk = SecretKey::random()?;
        let pk = PublicKey::from_secret(sk);
        Ok(pk)
    }

    /// `from_point` creates a new `PublicKey` from a `CompressedRistretto`.
    pub fn from_point(point: CompressedRistretto) -> PublicKey {
        PublicKey(point)
    }

    /// `to_point` returns the inner `CompressedRistretto` of the `PublicKey`.
    pub fn to_point(&self) -> CompressedRistretto {
        self.0
    }

    /// `from_bytes` creates a new `PublicKey` from a slice of bytes.
    pub fn from_bytes(buf: [u8; PUBLIC_KEY_LEN]) -> PublicKey {
        let point = CompressedRistretto::from_slice(&buf[..]);
        PublicKey(point)
    }

    /// `to_bytes` returns the `PublicKey` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; PUBLIC_KEY_LEN] {
        self.0.to_bytes()
    }

    /// `from_slice` creates a new `PublicKey` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<PublicKey> {
        let len = buf.len();
        if len != PUBLIC_KEY_LEN {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut pk = [0u8; PUBLIC_KEY_LEN];
        pk.copy_from_slice(buf);

        let public_key = PublicKey::from_bytes(pk);
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
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Mul<SecretKey> for PublicKey {
    type Output = Option<PublicKey>;

    fn mul(self, sk: SecretKey) -> Option<PublicKey> {
        if let Some(point) = self.to_point().decompress() {
            let scalar = sk.to_scalar();
            let point = (scalar * point).compress();
            let pk = PublicKey::from_point(point);
            Some(pk)
        } else {
            None
        }
    }
}

impl Add<PublicKey> for PublicKey {
    type Output = Option<PublicKey>;

    fn add(self, other: PublicKey) -> Option<PublicKey> {
        if let Some(point) = self.to_point().decompress() {
            if let Some(other_point) = other.to_point().decompress() {
                let point = (point + other_point).compress();
                let pk = PublicKey::from_point(point);
                Some(pk)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// `KeyPair` is a pair of ElGamal `PublicKey` and `SecretKey`.
#[derive(Copy, Clone, Debug)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub secret_key: SecretKey,
}

impl KeyPair {
    /// `new` creates a new random `KeyPair`.
    pub fn new() -> Result<KeyPair> {
        let secret_key = SecretKey::new()?;
        let public_key = secret_key.to_public();

        let keys = KeyPair {
            public_key,
            secret_key,
        };

        Ok(keys)
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
    pub fn from_hash<D>(digest: D) -> KeyPair
    where
        D: Digest<OutputSize = U64>,
    {
        let secret_key = SecretKey::from_hash(digest);
        let public_key = secret_key.to_public();

        KeyPair {
            public_key,
            secret_key,
        }
    }

    /// `from_scalar` creates a new `KeyPair` from a `Scalar`.
    /// The `Scalar` value cannot be 0.
    pub fn from_scalar(scalar: Scalar) -> Result<KeyPair> {
        let secret_key = SecretKey::from_scalar(scalar)?;
        let public_key = secret_key.to_public();

        let keys = KeyPair {
            public_key,
            secret_key,
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
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// `CypherText` is the cyphertext generated by ElGamal encryption.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CypherText {
    gamma: PublicKey,
    delta: CompressedRistretto,
}

impl CypherText {
    /// `random` creates a new random `CypherText`.
    pub fn random() -> Result<CypherText> {
        let gamma = PublicKey::random()?;
        let delta = PublicKey::random()?.to_point();

        let cyph = CypherText { gamma, delta };
        Ok(cyph)
    }

    /// `from_bytes` creates a new `CypherText` from an array of bytes.
    pub fn from_bytes(buf: [u8; 64]) -> Result<CypherText> {
        let mut gamma_buf = [0u8; GAMMA_LEN];
        for (i, v) in buf[0..GAMMA_LEN].iter().enumerate() {
            gamma_buf[i] = *v;
        }

        let mut delta_buf = [0u8; DELTA_LEN];
        for (i, v) in buf[GAMMA_LEN..].iter().enumerate() {
            delta_buf[i] = *v;
        }

        let gamma = PublicKey::from_bytes(gamma_buf);
        let delta = CompressedRistretto::from_slice(&delta_buf);

        let cyph = CypherText { gamma, delta };
        Ok(cyph)
    }

    /// `to_bytes` returns the `CypherText` as an array of bytes.
    pub fn to_bytes(&self) -> [u8; CYPHERTEXT_LEN] {
        let mut buf = [0u8; CYPHERTEXT_LEN];
        for (i, v) in self.gamma.to_bytes().iter().enumerate() {
            buf[i] = *v;
        }

        for (i, v) in self.delta.to_bytes().iter().enumerate() {
            buf[i + GAMMA_LEN] = *v;
        }

        buf
    }

    /// `from_slice` creates a new `CypherText` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<CypherText> {
        let len = buf.len();
        if len != CYPHERTEXT_LEN {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut ct = [0u8; CYPHERTEXT_LEN];
        ct.copy_from_slice(buf);

        CypherText::from_bytes(ct)
    }

    /// `to_vec` converts the `CypherText` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `CypherText` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<CypherText> {
        let len = s.len();
        if len != CYPHERTEXT_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        CypherText::from_slice(&buf)
    }

    /// `to_string` returns a `CypherText` hex string.
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.to_bytes().as_ref())
    }
}

impl fmt::Display for CypherText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Add<CypherText> for CypherText {
    type Output = Option<CypherText>;

    fn add(self, other: CypherText) -> Option<CypherText> {
        if let Some(gamma) = self.gamma + other.gamma {
            if let Some(delta_point) = self.delta.decompress() {
                if let Some(other_delta_point) = other.delta.decompress() {
                    let delta = (delta_point + other_delta_point).compress();
                    let cyph = CypherText { gamma, delta };
                    Some(cyph)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// `shared` returns the shared key between a `PublicKey` and a `SecretKey` of different `KeyPair`s.
fn shared(pk: PublicKey, sk: SecretKey) -> Result<CompressedRistretto> {
    if sk.to_public().to_point().ct_eq(&pk.to_point()).unwrap_u8() == 1u8 {
        let msg = "same secret keys".into();
        let err = Error::Keys { msg };
        return Err(err);
    }

    let shared = pk * sk;

    if let Some(shared) = shared {
        Ok(shared.to_point())
    } else {
        let msg = "invalid public key".into();
        let err = Error::PublicKey { msg };
        Err(err)
    }
}

/// `inverse_shared` returns the inverse of the shared point by using the Lagrange's Theorem.
fn inverse_shared(pk: PublicKey, sk: SecretKey) -> Result<CompressedRistretto> {
    if sk.to_public().to_point().ct_eq(&pk.to_point()).unwrap_u8() == 1u8 {
        let msg = "same secret keys".into();
        let err = Error::Keys { msg };
        return Err(err);
    }

    if let Some(pk_point) = pk.to_point().decompress() {
        let sk_scalar = sk.to_scalar();

        let inv_shared = pk_point * (BASEPOINT_ORDER - sk_scalar);
        Ok(inv_shared.compress())
    } else {
        let msg = "invalid public key".into();
        let err = Error::PublicKey { msg };
        Err(err)
    }
}

/// `encrypt` encrypts a `Message` into a `CypherText`.
pub fn encrypt(msg: Message, pk: PublicKey, sk: SecretKey) -> Result<CypherText> {
    if sk.to_public().to_point().ct_eq(&pk.to_point()).unwrap_u8() == 1u8 {
        let msg = "same secret keys".into();
        let err = Error::Keys { msg };
        return Err(err);
    }

    if let Some(msg_point) = msg.to_point().decompress() {
        if let Some(shared_point) = shared(pk, sk)?.decompress() {
            let delta = (msg_point + shared_point).compress();
            let gamma = sk.to_public();

            let cyph = CypherText { gamma, delta };
            Ok(cyph)
        } else {
            let msg = "invalid shared secret".into();
            let err = Error::SharedSecret { msg };
            Err(err)
        }
    } else {
        let msg = "same message".into();
        let err = Error::Message { msg };
        Err(err)
    }
}

/// `decrypt` decrypts a `CypherText` into a `Message`.
pub fn decrypt(cyph: CypherText, sk: SecretKey) -> Result<Message> {
    if sk
        .to_public()
        .to_point()
        .ct_eq(&cyph.gamma.to_point())
        .unwrap_u8()
        == 1u8
    {
        let msg = "same secret keys".into();
        let err = Error::Keys { msg };
        return Err(err);
    }

    if let Some(delta_point) = cyph.delta.decompress() {
        if let Some(inv_shared_point) = inverse_shared(cyph.gamma, sk)?.decompress() {
            let msg_point = (delta_point + inv_shared_point).compress();

            let msg = Message::from_point(&msg_point);
            Ok(msg)
        } else {
            let msg = "invalid shared secret".into();
            let err = Error::SharedSecret { msg };
            Err(err)
        }
    } else {
        let msg = "invalid cyphertext".into();
        let err = Error::CypherText { msg };
        Err(err)
    }
}

#[test]
fn message_serialize() {
    use crate::random::Random;

    let buf = Random::bytes(MESSAGE_LEN).unwrap();

    let res = Message::from_slice(&buf);
    assert!(res.is_ok());
    let message_a = res.unwrap();

    let hex = message_a.to_string();

    let res = Message::from_str(&hex);
    assert!(res.is_ok());

    let message_b = res.unwrap();
    assert_eq!(message_a, message_b)
}

#[test]
fn secret_key_serialize() {
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
fn public_key_serialize() {
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
fn cyphertext_serialize() {
    use crate::random::Random;

    let buf = Random::bytes(CYPHERTEXT_LEN).unwrap();

    let res = CypherText::from_slice(&buf);
    assert!(res.is_ok());
    let cyphertext_a = res.unwrap();

    let hex = cyphertext_a.to_string();

    let res = CypherText::from_str(&hex);
    assert!(res.is_ok());

    let cyphertext_b = res.unwrap();
    assert_eq!(cyphertext_a, cyphertext_b)
}

#[test]
fn test_shared() {
    let sk1 = SecretKey::new().unwrap();
    let pk1 = PublicKey::new(sk1);
    let sk2 = SecretKey::new().unwrap();
    let pk2 = PublicKey::new(sk2);

    let s1 = shared(pk2, sk1).unwrap();
    let s2 = shared(pk1, sk2).unwrap();

    assert_eq!(s1, s2)
}

#[test]
fn test_inverse_shared() {
    use curve25519_dalek::traits::Identity;

    let sk1 = SecretKey::new().unwrap();
    let pk1 = PublicKey::new(sk1);
    let sk2 = SecretKey::new().unwrap();
    let pk2 = PublicKey::new(sk2);

    let s = shared(pk2, sk1).unwrap();
    let inv_s1 = inverse_shared(pk2, sk1).unwrap();
    let inv_s2 = inverse_shared(pk1, sk2).unwrap();

    assert_eq!(inv_s1, inv_s2);

    let s_point = s.decompress().unwrap();
    let inv_s1_point = inv_s1.decompress().unwrap();
    let inv_s2_point = inv_s2.decompress().unwrap();

    let id = RistrettoPoint::identity();
    let id1 = s_point + inv_s1_point;
    let id2 = s_point + inv_s2_point;

    assert_eq!(id, id1);
    assert_eq!(id, id2);
    assert_eq!(id1, id2);
}

#[test]
fn test_encryption() {
    for _ in 0..10 {
        let msg1 = Message::random().unwrap();
        let sk1 = SecretKey::new().unwrap();
        let sk2 = SecretKey::new().unwrap();
        let pk2 = PublicKey::new(sk2);

        let cyph = encrypt(msg1, pk2, sk1).unwrap();
        let msg2 = decrypt(cyph, sk2).unwrap();

        assert_eq!(msg1, msg2)
    }
}

#[test]
fn test_message_add() {
    let msg1 = Message::random().unwrap();
    let msg2 = Message::random().unwrap();
    let msg3 = (msg1 + msg2).unwrap();

    let msg1_point = msg1.to_point().decompress().unwrap();
    let msg2_point = msg2.to_point().decompress().unwrap();
    let msg3_point = msg3.to_point().decompress().unwrap();

    assert_eq!(msg3_point, msg1_point + msg2_point);
}

#[test]
fn test_cyphertext_add() {
    let cyph1 = CypherText::random().unwrap();
    let cyph2 = CypherText::random().unwrap();
    let cyph3 = (cyph1 + cyph2).unwrap();
    let cyph3_gamma = (cyph1.gamma + cyph2.gamma).unwrap();
    let cyph3_delta =
        (cyph1.delta.decompress().unwrap() + cyph2.delta.decompress().unwrap()).compress();

    assert_eq!(cyph3.gamma, cyph3_gamma);
    assert_eq!(cyph3.delta, cyph3_delta);
}

#[test]
fn test_encryption_message_sum() {
    for _ in 0..10 {
        let msg1 = Message::random().unwrap();
        let msg2 = Message::random().unwrap();
        let msg3_from_sum = (msg1 + msg2).unwrap();

        let sk1 = SecretKey::new().unwrap();
        let sk2 = SecretKey::new().unwrap();
        let pk2 = PublicKey::new(sk2);

        let cyph3 = encrypt(msg3_from_sum, pk2, sk1).unwrap();

        let msg3_from_decrypt = decrypt(cyph3, sk2).unwrap();

        assert_eq!(msg3_from_sum, msg3_from_decrypt)
    }
}

#[test]
fn test_encryption_cyphertext_sum() {
    for _ in 0..10 {
        let msg1 = Message::random().unwrap();
        let msg2 = Message::random().unwrap();
        let msg3_from_sum = (msg1 + msg2).unwrap();

        let sk1 = SecretKey::new().unwrap();
        let sk2 = SecretKey::new().unwrap();
        let pk2 = PublicKey::new(sk2);

        let cyph1 = encrypt(msg1, pk2, sk1).unwrap();
        let cyph2 = encrypt(msg2, pk2, sk1).unwrap();
        let cyph3_from_sum = (cyph1 + cyph2).unwrap();

        let msg3_from_decrypt = decrypt(cyph3_from_sum, sk2).unwrap();

        assert_eq!(msg3_from_sum, msg3_from_decrypt)
    }
}

#[test]
fn test_encryption_cyphertext_sum_2() {
    for _ in 0..10 {
        let msg1 = Message::random().unwrap();
        let msg2 = Message::random().unwrap();
        let msg3 = Message::random().unwrap();
        let msg4_from_sum = ((msg1 + msg2).unwrap() + msg3).unwrap();

        let sk1 = SecretKey::new().unwrap();
        let sk2 = SecretKey::new().unwrap();
        let pk2 = PublicKey::new(sk2);

        let cyph1 = encrypt(msg1, pk2, sk1).unwrap();
        let cyph2 = encrypt(msg2, pk2, sk1).unwrap();
        let cyph3 = encrypt(msg3, pk2, sk1).unwrap();
        let cyph4_from_sum = ((cyph1 + cyph2).unwrap() + cyph3).unwrap();

        let msg4_from_decrypt = decrypt(cyph4_from_sum, sk2).unwrap();

        assert_eq!(msg4_from_sum, msg4_from_decrypt)
    }
}
