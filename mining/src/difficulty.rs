//! # Difficulty
//!
//! The `difficulty` crate contains the difficulty calculation functions.

use std::f64::consts::PI;

/// `calc_k` calculates 6/pi^2.
pub fn calc_k() -> f64 {
    6.0/(PI.powi(2))
}

/// `riemann_zeta_2` calculates the value of the Riemann Zeta function with s = 2
/// at a specific iteration.
pub fn riemmann_zeta_2(n: u64) -> Result<f64, String> {
    if n == 0 {
        let err = "out of bound".into();
        return Err(err);
    }

    let mut res = 0f64;

    for i in 1..=n {
        let i2 = i.pow(2) as f64;
        res += 1f64 / i2;
    }

    Ok(res)
}

/// `difficulty` calculates the difficulty bits given a specific distance
/// from the Eve transaction.
pub fn difficulty(d: u64) -> Result<u64, String> {
    let k = calc_k();
    let res = (512f64 * k * riemmann_zeta_2(d)?).floor() as u64;
    Ok(res)
}

#[test]
fn test_riemmann_zeta_2() {
    let tests = [1, 10, 100, 1000, 1_000_000];
    let expected = [0.60792, 0.94214, 0.99395, 0.99939, 0.99999];

    let res = riemmann_zeta_2(0);
    assert!(res.is_err());

    let k = calc_k();

    for (i, d) in tests.iter().enumerate() {
        let res = riemmann_zeta_2(*d);
        assert!(res.is_ok());

        let rz = res.unwrap();
        let krz = k*rz;

        assert!(krz >= expected[i])
    }
}

#[test]
fn test_difficulty() {
    let tests = [1, 10, 100, 1000, 1_000_000];
    let expected = [311, 482, 508, 511, 511];

    let res = difficulty(0);
    assert!(res.is_err());

    for (i, d) in tests.iter().enumerate() {
        let res = riemmann_zeta_2(*d);
        assert!(res.is_ok());

        let res = difficulty(*d);
        assert!(res.is_ok());

        let diff = res.unwrap();
        
        assert_eq!(diff, expected[i]);
    }
}
