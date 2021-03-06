//! # Digest
//!
//! `digest` is the module containing the `Digest` type returned by hashing
//! algorithms.

use crate::error::Error;
use crate::random::Random;
use crate::result::Result;
use rand_core::RngCore;
use rand_os::OsRng;
use serde::de;
use serde::{Deserialize, Deserializer};
use serde::{Serialize, Serializer};
use std::cmp;
use std::convert::From;
use std::fmt;
use std::ops::{Index, IndexMut};
use std::result;
use subtle::ConstantTimeEq;

/// `DIGEST_LEN` is the length of a `Digest`.
pub const DIGEST_LEN: usize = 64;

/// `Digest` is the type returned by hashing algorithms.
#[derive(Copy, Clone)]
pub struct Digest([u8; DIGEST_LEN]);

impl Digest {
    /// `new` creates a new `Digest` from an array of bytes. Alias of `from_bytes`.
    pub fn new(d: [u8; DIGEST_LEN]) -> Digest {
        Digest(d)
    }

    /// `random` creates a random `Digest`.
    pub fn random() -> Result<Digest> {
        let mut rng = OsRng::new()?;
        Digest::from_rng(&mut rng)
    }

    /// `from_rng` creates a new random `Digest`, but requires
    /// to specify a random generator.
    pub fn from_rng<R: RngCore>(rng: &mut R) -> Result<Digest> {
        let bytes = Random::bytes_from_rng(rng, DIGEST_LEN);
        Digest::from_slice(&bytes)
    }

    /// `from_bytes` creates a new `Digest` from an array of bytes.
    pub fn from_bytes(d: [u8; DIGEST_LEN]) -> Digest {
        Digest(d)
    }

    /// `to_bytes` converts the `Digest` into an array of bytes.
    pub fn to_bytes(&self) -> [u8; DIGEST_LEN] {
        self.0
    }

    /// `from_slice` creates a new `Digest` from a slice of bytes.
    pub fn from_slice(buf: &[u8]) -> Result<Digest> {
        let len = buf.len();
        if len != DIGEST_LEN {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut d = [0u8; DIGEST_LEN];
        d.copy_from_slice(buf);

        let digest = Digest::from_bytes(d);
        Ok(digest)
    }

    /// `to_vec` converts the `Digest` into a vector of bytes.
    pub fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let bytes = self.to_bytes();
        buf.extend_from_slice(bytes.as_ref());
        buf
    }

    /// `from_str` creates a new `Digest` from an hex string.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Digest> {
        let len = s.len();
        if len != DIGEST_LEN * 2 {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let mut buf = Vec::new();
        base16::decode_buf(s.as_bytes(), &mut buf)?;

        Digest::from_slice(&buf)
    }

    /// `to_string` returns a `Digest` hex string.
    #[allow(clippy::inherent_to_string_shadow_display)]
    pub fn to_string(&self) -> String {
        base16::encode_lower(self.0.as_ref())
    }

    /// `leading_zeros` returns the `Digest` leading zeros.
    pub fn leading_zeros(&self) -> u64 {
        let mut zeros = 0;

        for b in self.0.iter() {
            if *b == 0 {
                zeros += 8;
            } else {
                zeros += u64::from(b.leading_zeros());
                break;
            }
        }

        zeros
    }

    /// `from_leading_zeros` creates a `Digest` with a specific
    /// number of leading zeros.
    pub fn from_leading_zeros(zeros: u64) -> Result<Digest> {
        if zeros > 512 {
            let msg = Error::OutOfBound;
            return Err(msg);
        }

        let mut ds = [255u8; DIGEST_LEN];
        let zb = (zeros / 8) as usize;

        #[allow(clippy::needless_range_loop)]
        for i in 0..zb {
            ds[i] = 0;
        }

        let rem = zeros % 8;
        if rem != 0 {
            ds[zb] >>= rem;
        }

        let d = Digest::from_bytes(ds);
        Ok(d)
    }
}

impl Default for Digest {
    fn default() -> Digest {
        Digest([0u8; DIGEST_LEN])
    }
}

impl fmt::Debug for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Digest({:?})", self.to_bytes().as_ref())
    }
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl PartialEq for Digest {
    fn eq(&self, other: &Digest) -> bool {
        (&self.to_bytes()).ct_eq(&other.to_bytes()).unwrap_u8() == 1u8
    }
}

impl Eq for Digest {}

impl PartialOrd for Digest {
    fn partial_cmp(&self, other: &Digest) -> Option<cmp::Ordering> {
        // NB: not constant-time
        Some(self.to_bytes().cmp(&other.to_bytes()))
    }
}

impl Ord for Digest {
    fn cmp(&self, other: &Digest) -> cmp::Ordering {
        // NB: not constant-time
        self.to_bytes().cmp(&other.to_bytes())
    }
}

impl Index<usize> for Digest {
    type Output = u8;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.0.index(idx)
    }
}

impl IndexMut<usize> for Digest {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        self.0.index_mut(idx)
    }
}

impl From<[u8; DIGEST_LEN]> for Digest {
    fn from(t: [u8; DIGEST_LEN]) -> Digest {
        Digest(t)
    }
}

// TODO: de-lame (de)serialization

impl Serialize for Digest {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = self.to_string();
        serializer.serialize_str(&hex)
    }
}

struct DigestVisitor;

impl<'de> de::Visitor<'de> for DigestVisitor {
    type Value = Digest;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string of length DIGEST_LEN*2")
    }

    fn visit_str<E>(self, value: &str) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Digest::from_str(value).map_err(|e| E::custom(format!("{}", e)))
    }

    fn visit_string<E>(self, value: String) -> result::Result<Self::Value, E>
    where
        E: de::Error,
    {
        Digest::from_str(&value).map_err(|e| E::custom(format!("{}", e)))
    }
}

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D>(deserializer: D) -> result::Result<Digest, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DigestVisitor)
    }
}

#[test]
fn test_digest_serialize() {
    use crate::random::Random;

    let buf = Random::bytes(DIGEST_LEN).unwrap();

    let res = Digest::from_slice(&buf);
    assert!(res.is_ok());
    let digest_a = res.unwrap();

    let hex = digest_a.to_string();

    let res = Digest::from_str(&hex);
    assert!(res.is_ok());

    let digest_b = res.unwrap();
    assert_eq!(digest_a, digest_b);
}
