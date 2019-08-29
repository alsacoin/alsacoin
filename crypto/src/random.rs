//! # Random
//!
//! `random` is the module containing the random functions used in Alsacoin.

use crate::error::Error;
use crate::result::Result;
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
    pub fn u32() -> Result<u32> {
        let mut rng = OsRng::new()?;
        let res = Random::u32_from_rng(&mut rng);
        Ok(res)
    }

    /// `u32_range` returns a random `u32` between a specific inclusive range.
    pub fn u32_range(from: u32, to: u32) -> Result<u32> {
        let mut rng = OsRng::new()?;
        let res = Random::u32_range_from_rng(&mut rng, from, to)?;
        Ok(res)
    }

    /// `u32_range_from_rng` returns a random `u32` between a specific range
    /// using a given RNG.
    pub fn u32_range_from_rng<R>(rng: &mut R, from: u32, to: u32) -> Result<u32>
    where
        R: RngCore,
    {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        if from == to {
            return Ok(from);
        }

        let interval = to - from;
        let val = Random::u32_from_rng(rng) % interval;
        let res = from + val;

        Ok(res)
    }

    /// `u32_sample` returns a random sample of `u32` values.
    pub fn u32_sample(count: u32) -> Result<Vec<u32>> {
        let mut res = Vec::new();

        for _ in 0..count as usize {
            res.push(Random::u32()?);
        }

        Ok(res)
    }

    /// `u32_sample_range` returns a random sample of `u32` values between an range.
    pub fn u32_sample_range(from: u32, to: u32, count: u32) -> Result<Vec<u32>> {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut res = Vec::new();

        for _ in 0..count as usize {
            res.push(Random::u32_range(from, to)?);
        }

        Ok(res)
    }

    /// `u32_sample_unique` returns a random sample of `u32` unique values.
    pub fn u32_sample_unique(count: u32) -> Result<Vec<u32>> {
        let mut res = Vec::new();

        for _ in 0..count as usize {
            let value = Random::u32()?;
            if !res.contains(&value) {
                res.push(value);
            }
        }

        Ok(res)
    }

    /// `u32_sample_unique_range` returns a random sample of `u32` unique values between an range.
    pub fn u32_sample_unique_range(from: u32, to: u32, count: u32) -> Result<Vec<u32>> {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut res = Vec::new();

        for _ in 0..count as usize {
            let value = Random::u32_range(from, to)?;
            if !res.contains(&value) {
                res.push(value);
            }
        }

        Ok(res)
    }

    /// `u64_from_rng` returns a random `u64` using a given RNG.
    pub fn u64_from_rng<R>(rng: &mut R) -> u64
    where
        R: RngCore,
    {
        rng.next_u64()
    }

    /// `u64` returns a random `u64`.
    pub fn u64() -> Result<u64> {
        let mut rng = OsRng::new()?;
        let res = Random::u64_from_rng(&mut rng);
        Ok(res)
    }

    /// `u64_range` returns a random `u64` between a specific inclusive range.
    pub fn u64_range(from: u64, to: u64) -> Result<u64> {
        let mut rng = OsRng::new()?;
        let res = Random::u64_range_from_rng(&mut rng, from, to)?;
        Ok(res)
    }

    /// `u64_range_from_rng` returns a random `u64` between a specific range
    /// using a given RNG.
    pub fn u64_range_from_rng<R>(rng: &mut R, from: u64, to: u64) -> Result<u64>
    where
        R: RngCore,
    {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        if from == to {
            return Ok(from);
        }

        let interval = to - from;
        let val = Random::u64_from_rng(rng) % interval;
        let res = from + val;

        Ok(res)
    }

    /// `u64_sample` returns a random sample of `u64` values.
    pub fn u64_sample(count: u64) -> Result<Vec<u64>> {
        let mut res = Vec::new();

        for _ in 0..count as usize {
            res.push(Random::u64()?);
        }

        Ok(res)
    }

    /// `u64_sample_range` returns a random sample of `u64` values between an range.
    pub fn u64_sample_range(from: u64, to: u64, count: u64) -> Result<Vec<u64>> {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut res = Vec::new();

        for _ in 0..count as usize {
            res.push(Random::u64_range(from, to)?);
        }

        Ok(res)
    }

    /// `u64_sample_unique` returns a random sample of `u64` unique values.
    pub fn u64_sample_unique(count: u64) -> Result<Vec<u64>> {
        let mut res = Vec::new();

        for _ in 0..count as usize {
            let value = Random::u64()?;
            if !res.contains(&value) {
                res.push(value);
            }
        }

        Ok(res)
    }

    /// `u64_sample_unique_range` returns a random sample of `u64` unique values between an range.
    pub fn u64_sample_unique_range(from: u64, to: u64, count: u64) -> Result<Vec<u64>> {
        if from > to {
            let err = Error::InvalidRange;
            return Err(err);
        }

        let mut res = Vec::new();

        for _ in 0..count as usize {
            let value = Random::u64_range(from, to)?;
            if !res.contains(&value) {
                res.push(value);
            }
        }

        Ok(res)
    }

    /// `fill_bytes_from_rng` fills a slice with random bytes using a given RNG.
    pub fn fill_bytes_from_rng<R>(rng: &mut R, buf: &mut [u8])
    where
        R: RngCore,
    {
        rng.fill_bytes(buf);
    }

    /// `fill_bytes` fills a slice with random bytes.
    pub fn fill_bytes(buf: &mut [u8]) -> Result<()> {
        let mut rng = OsRng::new()?;
        Random::fill_bytes_from_rng(&mut rng, buf);
        Ok(())
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
    pub fn bytes(len: usize) -> Result<Vec<u8>> {
        let mut rng = OsRng::new()?;
        let res = Random::bytes_from_rng(&mut rng, len);
        Ok(res)
    }
}

#[test]
fn test_u32_range() {
    for _ in 0..10 {
        let valid_from = Random::u32().unwrap() % (u32::max_value() / 2);
        let valid_to = valid_from * 2;
        let invalid_from = valid_to;
        let invalid_to = valid_from;

        let res = Random::u32_range(invalid_from, invalid_to);
        assert!(res.is_err());

        let res = Random::u32_range(valid_from, valid_to);
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val >= valid_from && val < valid_to)
    }
}

#[test]
fn test_u64_range() {
    for _ in 0..10 {
        let valid_from = Random::u64().unwrap() % (u64::max_value() / 2);
        let valid_to = valid_from * 2;
        let invalid_from = valid_to;
        let invalid_to = valid_from;

        let res = Random::u64_range(invalid_from, invalid_to);
        assert!(res.is_err());

        let res = Random::u64_range(valid_from, valid_to);
        assert!(res.is_ok());
        let val = res.unwrap();
        assert!(val >= valid_from && val < valid_to)
    }
}
