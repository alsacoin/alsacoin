//! # Coinbase
//!
//! The `coinbase` module contains the conbaise generation functions.

use crate::common::riemmann_zeta_2;
use crate::error::Error;
use crate::result::Result;

/// `COINBASE_BASE_AMOUNT` is the coinbase base amount.
pub const COINBASE_BASE_AMOUNT: u64 = 1_000_000_000;

/// `coinbase_amount` returns the coinbase amount given the distance
/// from the eve transaction and the difficulty.
pub fn coinbase_amount(h: u64, d: u64) -> Result<u64> {
    if (h == 0) || (d == 0) || (d > 512) {
        let err = Error::OutOfBound;
        return Err(err);
    }

    let epoch = 1 + (h as f64 / 1000f64) as u64;
    let res = ((COINBASE_BASE_AMOUNT as f64) * riemmann_zeta_2(epoch)? / riemmann_zeta_2(d)?)
        .floor() as u64;
    Ok(res)
}

#[test]
fn test_coinbase_amount() {
    let hs = [1, 1000, 1_000_000];
    let ds = [1, 255, 512];
    let expected = [
        [COINBASE_BASE_AMOUNT, 609377028, 608649080],
        [1250000000, 761721286, 760811350],
        [1643935564, 1001776569, 1000579870],
    ];

    let res = coinbase_amount(0, 1);
    assert!(res.is_err());

    let res = coinbase_amount(1, 0);
    assert!(res.is_err());

    for (i, h) in hs.iter().enumerate() {
        for (j, d) in ds.iter().enumerate() {
            let res = coinbase_amount(*h, *d);
            assert!(res.is_ok());

            let ca = res.unwrap();
            assert_eq!(ca, expected[i][j])
        }
    }
}
