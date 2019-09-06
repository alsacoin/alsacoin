//! # Protocol Server
//!
//! `server` is the module containing the protocol server type and functions.

use crate::error::Error;
use crate::network::{serve_avalanche, serve_incoming};
use crate::result::Result;
use crate::state::ProtocolState;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use std::thread;
use store::traits::Store;

/// `ProtocolServer` is the protocol main server type.
pub struct ProtocolServer<
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
> {
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
}

impl<S: Store + Send + 'static, P: Store + Send + 'static, T: Transport + Send + 'static>
    ProtocolServer<S, P, T>
{
    /// `new` creates a new `ProtocolServer`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
    ) -> Result<ProtocolServer<S, P, T>> {
        state.lock().unwrap().validate()?;

        let server = ProtocolServer { state, transport };

        Ok(server)
    }

    /// `validate` validates the `ProtocolServer`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()
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
