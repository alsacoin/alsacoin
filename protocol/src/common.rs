//! # Common
//!
//! `common` is the module containing crate's common functionalities.

use crate::result::Result;
use log::logger::Logger;
use std::sync::Arc;

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
