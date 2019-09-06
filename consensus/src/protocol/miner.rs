//! # Protocol Miner
//!
//! `miner` is the module containing the protocol miner type and functions.

use crate::error::Error;
use crate::result::Result;
use config::consensus_config::ConsensusConfig;
use crypto::hash::Digest;
use network::error::Error as NetworkError;
use models::account::Account;
use models::conflict_set::ConflictSet;
use models::consensus_state::ConsensusState;
use models::error::Error as ModelsError;
use models::node::Node;
use models::consensus_message::ConsensusMessage;
use models::stage::Stage;
use models::traits::Storable;
use models::transaction::Transaction;
use std::collections::BTreeSet;
use crate::protocol::ProtocolState;
use network::traits::Transport;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolMiner` is the protocol miner type.
pub struct ProtocolMiner<S: Store, P: Store, T: Transport> {
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
}

impl<S: Store, P: Store, T: Transport> ProtocolMiner<S, P, T> {
    /// `new` creates a new `ProtocolMiner`.
    pub fn new(state: Arc<Mutex<ProtocolState<S, P>>>, transport: Arc<Mutex<T>>) -> Result<ProtocolMiner<S, P, T>> {
        state.lock().unwrap().validate()?;

        let miner = ProtocolMiner {
            state,
            transport,
        };

        Ok(miner)
    }

    /// `validate` validates the `ProtocolMiner`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()
    }

    /// `handle_mine` handles a `Mine` `ConsensusMessage` request.
    pub fn handle_mine(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::Mine {
                id,
                address,
                node,
                transactions,
                ..
            } => {
                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                let node = Node::new(self.stage, &address);
                self.handle_node(&node)?;

                for transaction in &transactions {
                    transaction.validate()?;

                    if transaction.is_mined() {
                        let err = Error::AlreadyMined;
                        return Err(err);
                    }
                }

                let mut mined = BTreeSet::new();

                for transaction in &transactions {
                    let mut tx = transaction.clone();
                    tx.mine()?;
                    mined.insert(tx);
                }

                for transaction in &mined {
                    self.handle_transaction(&transaction)?;
                }

                let cons_msg =
                    ConsensusMessage::new_push_transactions(&self.address, id + 1, &node, &mined)?;
                self.send_message(&cons_msg)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `serve_mining` serves the mining operations.
    pub fn serve_mining(&mut self) -> Result<()> {
        let timeout = self.state.lock().unwrap().config.timeout;
        let mut transport = self.transport.clone();

        transport
            .serve(timeout, |msg| {
                let cons_msg = msg.to_consensus_message()?;

                self.handle_mine(&cons_msg)
                    .map_err(|e| NetworkError::Consensus {
                        msg: format!("{}", e),
                    })
            })
            .map_err(|e| e.into())
    }
}
