//! # Random
//!
//! `random` is the module containing the random functions used in Alsacoin.

use rand_core::RngCore;
use rand_os::OsRng;

/// `Random` is the type implemeting random functions.
pub struct Random;

impl Random {
    /// `u32_from_rng` returns a random `u32` using a given RNG.
    pub fn u32_from_rng<R>(rng: &mut R) -> u32
    where
        R: RngCore,
    {
        rng.next_u32()
    }

    /// `u32` returns a random `u32`.
    #[allow(dead_code)]
    fn u32() -> u32 {
        Random::u32_from_rng(&mut OsRng)
    }

    /// `u64_from_rng` returns a random `u64` using a given RNG.
    pub fn u64_from_rng<R>(rng: &mut R) -> u64
    where
        R: RngCore,
    {
        rng.next_u64()
    }

    /// `u64` returns a random `u64`.
    #[allow(dead_code)]
    fn u64() -> u64 {
        Random::u64_from_rng(&mut OsRng)
    }

    /// `fill_bytes_from_rng` fills a slice with random bytes using a given RNG.
    pub fn fill_bytes_from_rng<R>(rng: &mut R, buf: &mut [u8])
    where
        R: RngCore,
    {
        rng.fill_bytes(buf);
    }

    /// `fill_bytes` fills a slice with random bytes.
    pub fn fill_bytes(buf: &mut [u8]) {
        Random::fill_bytes_from_rng(&mut OsRng, buf)
    }

    /// `bytes_from_rng` creates a vector of random bytes using a given RNG.
    pub fn bytes_from_rng<R>(rng: &mut R, len: usize) -> Vec<u8>
    where
        R: RngCore,
    {
        let mut buf = Vec::new();
        buf.resize(len, 0);

        rng.fill_bytes(&mut buf);

        let mut res = Vec::new();
        res.extend_from_slice(&buf[..]);
        res
    }

    /// `bytes` creates a vector of random bytes.
    pub fn bytes(len: usize) -> Vec<u8> {
        Random::bytes_from_rng(&mut OsRng, len)
    }
}
