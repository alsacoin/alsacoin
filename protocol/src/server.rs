//! # Protocol Server
//!
//! `server` is the module containing the protocol server type and functions.

use crate::error::Error;
use crate::network::{serve_avalanche, serve_incoming};
use crate::result::Result;
use crate::state::ProtocolState;
use log::logger::Logger;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use std::thread;
use store::traits::Store;

/// `ProtocolServer` is the protocol main server type.
pub struct ProtocolServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
    pub logger: Arc<Logger>,
}

impl<S, P, T> ProtocolServer<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    /// `new` creates a new `ProtocolServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolServer<S, P, T>> {
        state.lock().unwrap().validate()?;
        logger.validate()?;

        let server = ProtocolServer {
            state,
            transport,
            logger,
        };

        Ok(server)
    }

    /// `validate` validates the `ProtocolServer`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()?;
        self.logger.validate().map_err(|e| e.into())
    }

    /// `serve` serves the main server operations.
    pub fn serve(&mut self) -> Result<()> {
        {
            let state = self.state.clone();
            let transport = self.transport.clone();

            let res = thread::spawn(move || serve_incoming(state, transport))
                .join()
                .map_err(|e| Error::Thread {
                    msg: format!("{:?}", e),
                })?;

            if res.is_err() {
                return res;
            }
        }

        {
            let state = self.state.clone();
            let transport = self.transport.clone();

            let res = thread::spawn(move || serve_avalanche(state, transport))
                .join()
                .map_err(|e| Error::Thread {
                    msg: format!("{:?}", e),
                })?;

            if res.is_err() {
                return res;
            }
        }

        Ok(())
    }
}
