use byteorder::{BigEndian, WriteBytesExt};

use crate::error::Error;
use crate::hash::Blake512Hasher;
use crate::hash::Digest;
use crate::result::Result;
use serde::{Deserialize, Serialize};

/// Params used in Balloon hashing.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize)]
pub struct BalloonParams {
    /// The s_cost parameter used in Balloon hashing.
    pub s_cost: u32,
    /// The t_cost parameter used in Balloon hashing.
    pub t_cost: u32,
    /// The delta parameter used in Balloon hashing.
    pub delta: u32,
}

impl BalloonParams {
    /// Creates a new `BalloonParams`.
    pub fn new(s_cost: u32, t_cost: u32, delta: u32) -> Result<BalloonParams> {
        let params = BalloonParams {
            s_cost,
            t_cost,
            delta,
        };

        params.validate()?;

        Ok(params)
    }

    /// Validates the `BalloonParams`.
    pub fn validate(&self) -> Result<()> {
        if self.s_cost == 0 {
            let msg = "invalid s_cost argument".into();
            let err = Error::BalloonParams { msg };
            return Err(err);
        }

        if self.t_cost == 0 {
            let msg = "invalid t_cost argument".into();
            let err = Error::BalloonParams { msg };
            return Err(err);
        }

        if self.delta < 3 {
            let msg = "invalid delta argument".into();
            let err = Error::BalloonParams { msg };
            return Err(err);
        }

        Ok(())
    }
}

impl Default for BalloonParams {
    fn default() -> BalloonParams {
        BalloonParams {
            s_cost: 1,
            t_cost: 1,
            delta: 3,
        }
    }
}

/// Hasher implementing Balloon hashing.
pub struct BalloonHasher {
    pub salt: Digest,
    pub params: BalloonParams,
}

impl BalloonHasher {
    /// Creates a new `BalloonHasher`.
    pub fn new(salt: Digest, params: BalloonParams) -> Result<BalloonHasher> {
        params.validate()?;

        Ok(BalloonHasher { salt, params })
    }

    /// Hashes a binary message into a `Digest`.
    pub fn hash(&self, msg: &[u8]) -> Result<Digest> {
        self.validate()?;

        let mut cnt = 0u32;
        let mut buf = Vec::new();

        for _ in 0..self.params.s_cost {
            buf.push(Digest::default())
        }

        let mut buf_0 = Vec::new();
        buf_0.write_u32::<BigEndian>(cnt)?;
        cnt += 1;
        buf_0.extend_from_slice(msg);
        buf_0.extend_from_slice(&self.salt.to_bytes());

        buf[0] = Blake512Hasher::hash(&buf_0);

        for m in 1..self.params.s_cost as usize {
            let mut buf_m_1 = Vec::new();
            buf_m_1.write_u32::<BigEndian>(cnt)?;
            cnt += 1;
            buf_m_1.extend_from_slice(&buf[m - 1].to_bytes());

            buf[m] = Blake512Hasher::hash(&buf_m_1);
        }

        // TODO: fix the algo online, contact the guys (t > 0)
        for t in 0..(self.params.t_cost - 1) as usize {
            // TODO: fix the algo online, contact the guys
            for m in 1..(self.params.s_cost - 1) as usize {
                let prev = buf[(m - 1 as usize) % self.params.s_cost as usize];
                let mut buf_m_2 = Vec::new();
                buf_m_2.write_u32::<BigEndian>(cnt)?;
                cnt += 1;
                buf_m_2.extend_from_slice(&prev.to_bytes());
                buf_m_2.extend_from_slice(&buf[m].to_bytes());

                buf[m] = Blake512Hasher::hash(&buf_m_2);

                for i in 0..(self.params.delta - 1) as usize {
                    // NB: block obtained by hashing
                    let mut buf_idx_block = Vec::new();
                    buf_idx_block.write_u32::<BigEndian>(t as u32)?;
                    buf_idx_block.write_u32::<BigEndian>(m as u32)?;
                    buf_idx_block.write_u32::<BigEndian>(i as u32)?;
                    let idx_block = Blake512Hasher::hash(&buf_idx_block);

                    let mut buf_i_1 = Vec::new();
                    buf_i_1.write_u32::<BigEndian>(cnt)?;
                    cnt += 1;
                    buf_i_1.extend_from_slice(&self.salt.to_bytes());
                    buf_i_1.extend_from_slice(&idx_block.to_bytes());

                    // TODO: should we hear those guys even here?
                    let other_buf = Blake512Hasher::hash(&buf_i_1).to_bytes();
                    let mut other: u32 = 0;
                    for i in other_buf.iter().take(64) {
                        other += u32::from(*i);
                    }
                    other %= self.params.s_cost;

                    let mut buf_i_2 = Vec::new();
                    buf_i_2.write_u32::<BigEndian>(cnt)?;
                    cnt += 1;
                    buf_i_2.extend_from_slice(&buf[m].to_bytes());
                    buf_i_2.extend_from_slice(&buf[other as usize].to_bytes());

                    buf[m] = Blake512Hasher::hash(&buf_i_2);
                }
            }
        }

        Ok(buf[(self.params.s_cost - 1) as usize])
    }

    /// Validates the `BalloonHasher`.
    pub fn validate(&self) -> Result<()> {
        self.params.validate()
    }
}

#[test]
fn balloon_params_new() {
    use crate::random::Random;

    for _ in 0..10 {
        let s_cost = Random::u32().unwrap();
        let t_cost = Random::u32().unwrap();
        let delta = Random::u32().unwrap();

        let res = BalloonParams::new(s_cost, t_cost, delta);

        if s_cost == 0 || t_cost == 0 || delta < 3 {
            assert!(res.is_err());
        } else {
            assert!(res.is_ok())
        }
    }
}

#[test]
fn balloon_params_validate() {
    use crate::random::Random;

    for _ in 0..10 {
        let s_cost = Random::u32().unwrap();
        let t_cost = Random::u32().unwrap();
        let delta = Random::u32().unwrap();

        let params = BalloonParams {
            s_cost,
            t_cost,
            delta,
        };

        let res = params.validate();

        if s_cost == 0 || t_cost == 0 || delta < 3 {
            assert!(res.is_err());
        } else {
            assert!(res.is_ok())
        }
    }
}

#[test]
fn balloon_hasher_new() {
    use crate::random::Random;

    for _ in 0..10 {
        let s_cost = Random::u32().unwrap();
        let t_cost = Random::u32().unwrap();
        let delta = Random::u32().unwrap();

        let params = BalloonParams {
            s_cost,
            t_cost,
            delta,
        };

        let salt = Digest::random().unwrap();

        let res = BalloonHasher::new(salt, params);

        if s_cost == 0 || t_cost == 0 || delta < 3 {
            assert!(res.is_err());
        } else {
            assert!(res.is_ok())
        }
    }
}

#[test]
fn balloon_hasher_validate() {
    use crate::random::Random;

    for _ in 0..10 {
        let s_cost = Random::u32().unwrap();
        let t_cost = Random::u32().unwrap();
        let delta = Random::u32().unwrap();

        let params = BalloonParams {
            s_cost,
            t_cost,
            delta,
        };

        let salt = Digest::random().unwrap();

        let hasher = BalloonHasher { salt, params };

        let res = hasher.validate();

        if s_cost == 0 || t_cost == 0 || delta < 3 {
            assert!(res.is_err());
        } else {
            assert!(res.is_ok())
        }
    }
}

#[test]
fn balloon_hasher_hash() {
    use crate::random::Random;

    for _ in 0..10 {
        let s_cost = Random::u32_range(0, 10).unwrap();
        let t_cost = Random::u32_range(0, 10).unwrap();
        let delta = Random::u32_range(0, 10).unwrap();

        let params = BalloonParams {
            s_cost,
            t_cost,
            delta,
        };

        let salt = Digest::random().unwrap();

        let hasher = BalloonHasher { salt, params };

        let msg_len = 1000;
        let msg = Random::bytes(msg_len).unwrap();

        let res = hasher.hash(&msg);

        if s_cost == 0 || t_cost == 0 || delta < 3 {
            assert!(res.is_err());
        } else {
            assert!(res.is_ok())
        }
    }
}
