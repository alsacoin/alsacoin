//! # Common
//!
//! The `common` module contains the common functionalities used by the `mining` crate.

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

#[test]
fn test_riemmann_zeta_2() {
    let ns = [1, 10, 100, 1000, 1_000_000];
    let expected = [0.0, 1.54976, 1.63498, 1.64393, 1.64493];

    let res = riemmann_zeta_2(0);
    assert!(res.is_err());

    for (i, d) in ns.iter().enumerate() {
        let res = riemmann_zeta_2(*d);
        assert!(res.is_ok());

        let rz = res.unwrap();
        assert!(rz >= expected[i])
    }
}
