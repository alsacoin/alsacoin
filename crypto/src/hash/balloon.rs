use byteorder::{BigEndian, WriteBytesExt};

use crate::error::Error;
use crate::hash::Blake512Hasher;
use crate::hash::Digest;
use crate::hash::CRH;
use crate::result::Result;

/// Params used in Balloon hashing.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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
    fn validate(&self) -> Result<()> {
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

        let hasher = Blake512Hasher;

        buf[0] = hasher.hash(&buf_0);

        for m in 1..self.params.s_cost as usize {
            let mut buf_m_1 = Vec::new();
            buf_m_1.write_u32::<BigEndian>(cnt)?;
            cnt += 1;
            buf_m_1.extend_from_slice(&buf[m - 1].to_bytes());

            buf[m] = hasher.hash(&buf_m_1);
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

                buf[m] = hasher.hash(&buf_m_2);

                for i in 0..(self.params.delta - 1) as usize {
                    // NB: block obtained by hashing
                    let mut buf_idx_block = Vec::new();
                    buf_idx_block.write_u32::<BigEndian>(t as u32)?;
                    buf_idx_block.write_u32::<BigEndian>(m as u32)?;
                    buf_idx_block.write_u32::<BigEndian>(i as u32)?;
                    let idx_block = hasher.hash(&buf_idx_block);

                    let mut buf_i_1 = Vec::new();
                    buf_i_1.write_u32::<BigEndian>(cnt)?;
                    cnt += 1;
                    buf_i_1.extend_from_slice(&self.salt.to_bytes());
                    buf_i_1.extend_from_slice(&idx_block.to_bytes());

                    // TODO: should we hear those guys even here?
                    let other_buf = hasher.hash(&buf_i_1).to_bytes();
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

                    buf[m] = hasher.hash(&buf_i_2);
                }
            }
        }

        Ok(buf[(self.params.s_cost - 1) as usize])
    }

    /// Validates the `BalloonHasher`.
    fn validate(&self) -> Result<()> {
        self.params.validate()
    }
}

impl CRH for BalloonHasher {
    fn hash(&self, msg: &[u8]) -> Digest {
        self.hash(msg).unwrap()
    }
}
