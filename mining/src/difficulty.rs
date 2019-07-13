//! # Difficulty
//!
//! The `difficulty` module contains the difficulty calculation functions.

use crate::common::riemmann_zeta_2;
use std::f64::consts::PI;

/// `calc_k` calculates 6/pi^2.
pub fn calc_k() -> f64 {
    6.0 / (PI.powi(2))
}

/// `difficulty` calculates the difficulty bits given a specific distance
/// from the eve transaction.
pub fn difficulty(d: u64) -> Result<u64, String> {
    let k = calc_k();
    let res = (512f64 * k * riemmann_zeta_2(d)?).floor() as u64;
    Ok(res)
}

#[test]
fn test_difficulty() {
    let ds = [1, 10, 100, 1000, 1_000_000];
    let expected = [311, 482, 508, 511, 511];

    let res = difficulty(0);
    assert!(res.is_err());

    for (i, d) in ds.iter().enumerate() {
        let res = riemmann_zeta_2(*d);
        assert!(res.is_ok());

        let res = difficulty(*d);
        assert!(res.is_ok());

        let diff = res.unwrap();

        assert_eq!(diff, expected[i]);
    }
}
