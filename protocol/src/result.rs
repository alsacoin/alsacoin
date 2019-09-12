//! # Result
//!
//! `result` is the module containing the `Result` type of the crate.

use crate::error::Error;
use log::logger::Logger;
use std::result::Result as StdResult;
use std::sync::Arc;

pub type Result<T> = StdResult<T, Error>;

/// `handle_result` handles a `Result` in case of error.
pub fn handle_result<T>(logger: Arc<Logger>, res: Result<T>, ctx: &str) -> Result<T> {
    match res {
        Ok(val) => Ok(val),
        Err(err) => {
            let msg = format!("{}: {}", ctx, &err);
            logger.log_critical(&msg)?;
            Err(err)
        }
    }
}
