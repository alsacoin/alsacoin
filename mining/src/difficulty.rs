//! # Difficulty
//!
//! The `difficulty` module contains the difficulty calculation functions.

use crate::common::riemmann_zeta_2;
use crate::error::Error;
use crate::result::Result;

/// `difficulty` calculates the difficulty bits given a specific distance
/// from the eve transaction and a specific amount.
pub fn difficulty(d: u64, a: u64) -> Result<u64> {
    if (d == 0) || (a == 0) {
        let err = Error::OutOfBound;
        return Err(err);
    }

    let d = 1 + (d as f64 / 1000f64) as u64;
    let a = 1 + (a as f64 / 1000f64) as u64;
    let res = (64f64 * riemmann_zeta_2(d)? / riemmann_zeta_2(a)?).floor() as u64;
    Ok(res)
}

#[test]
fn test_difficulty() {
    let ds = [1, 1_000, 1_000_000];
    let ams = [1, 1_000, 1_000_000];
    let expected = [[64, 51, 38], [80, 64, 48], [105, 84, 64]];

    let res = difficulty(0, 1);
    assert!(res.is_err());

    let res = difficulty(1, 0);
    assert!(res.is_err());

    for (i, d) in ds.iter().enumerate() {
        for (j, a) in ams.iter().enumerate() {
            let res = difficulty(*d, *a);
            assert!(res.is_ok());

            let diff = res.unwrap();
            assert_eq!(diff, expected[i][j]);
        }
    }
}
