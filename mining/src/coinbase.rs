//! # Coinbase
//!
//! The `coinbase` module contains the conbaise generation functions.

use crate::common::riemmann_zeta_2;
use crate::error::Error;
use crate::result::Result;

/// `COINBASE_BASE` is the coinbase base amount.
pub const COINBASE_BASE: u64 = 1_000_000;

/// `coinbase_amount` returns the coinbase amount given the distance
/// from the eve transaction.
pub fn coinbase_amount(d: u64) -> Result<u64> {
    if d == 0 {
        let err = Error::OutOfBound;
        return Err(err);
    }

    let res = ((COINBASE_BASE as f64) * riemmann_zeta_2(d)?).floor() as u64;
    Ok(res)
}

#[test]
fn test_coinbase_amount() {
    let ds = [1, 10, 100, 1000, 1_000_000];
    let expected = [1_000_000, 1549767, 1634983, 1643934, 1644933];

    let res = coinbase_amount(0);
    assert!(res.is_err());

    for (i, d) in ds.iter().enumerate() {
        let res = coinbase_amount(*d);
        assert!(res.is_ok());

        let ca = res.unwrap();
        assert_eq!(ca, expected[i])
    }
}
