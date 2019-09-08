//! # Protocol Client
//!
//! `client` is the module containing the protocol client type and functions.

use crate::network as protocol_network;
use crate::result::Result;
use crate::state::ProtocolState;
use crypto::hash::Digest;
use log::logger::Logger;
use models::node::Node;
use models::transaction::Transaction;
use network::traits::Transport;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolClient` is the protocol client type.
pub struct ProtocolClient<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    pub state: Arc<Mutex<ProtocolState<S, P>>>,
    pub transport: Arc<Mutex<T>>,
    pub logger: Arc<Logger>,
}

impl<S, P, T> ProtocolClient<S, P, T>
where
    S: Store + Send + 'static,
    P: Store + Send + 'static,
    T: Transport + Send + 'static,
{
    /// `new` creates a new `ProtocolClient`.
    pub fn new(
        state: Arc<Mutex<ProtocolState<S, P>>>,
        transport: Arc<Mutex<T>>,
        logger: Arc<Logger>,
    ) -> Result<ProtocolClient<S, P, T>> {
        state.lock().unwrap().validate()?;
        logger.validate()?;

        let client = ProtocolClient {
            state,
            transport,
            logger,
        };

        Ok(client)
    }

    /// `validate` validates the `ProtocolClient`.
    pub fn validate(&self) -> Result<()> {
        self.state.lock().unwrap().validate()?;
        self.logger.validate().map_err(|e| e.into())
    }

    /// `fetch_node_transactions` fetches transactions from a remote node.
    pub fn fetch_node_transactions(
        &mut self,
        address: &[u8],
        ids: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Transaction>> {
        protocol_network::fetch_node_transactions(
            self.state.clone(),
            self.transport.clone(),
            address,
            ids,
        )
    }

    /// `fetch_transactions` fetches transactions from remote.
    pub fn fetch_transactions(&mut self, ids: &BTreeSet<Digest>) -> Result<BTreeSet<Transaction>> {
        protocol_network::fetch_transactions(self.state.clone(), self.transport.clone(), ids)
    }

    /// `fetch_node_random_transactions` fetches random transactions from a remote node.
    pub fn fetch_node_random_transactions(
        &mut self,
        address: &[u8],
        count: u32,
    ) -> Result<BTreeSet<Transaction>> {
        protocol_network::fetch_node_random_transactions(
            self.state.clone(),
            self.transport.clone(),
            address,
            count,
        )
    }

    /// `fetch_random_transactions` fetches random transactions from remote.
    pub fn fetch_random_transactions(&mut self, count: u32) -> Result<BTreeSet<Transaction>> {
        protocol_network::fetch_random_transactions(
            self.state.clone(),
            self.transport.clone(),
            count,
        )
    }

    /// `fetch_node_nodes` fetches nodes from a remote node.
    pub fn fetch_node_nodes(
        &mut self,
        address: &[u8],
        ids: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Node>> {
        protocol_network::fetch_node_nodes(self.state.clone(), self.transport.clone(), address, ids)
    }

    /// `fetch_nodes` fetches nodes from remote.
    pub fn fetch_nodes(&mut self, ids: &BTreeSet<Digest>) -> Result<BTreeSet<Node>> {
        protocol_network::fetch_nodes(self.state.clone(), self.transport.clone(), ids)
    }

    /// `fetch_node_random_nodes` fetches random nodes from a remote node.
    pub fn fetch_node_random_nodes(
        &mut self,
        address: &[u8],
        count: u32,
    ) -> Result<BTreeSet<Node>> {
        protocol_network::fetch_node_random_nodes(
            self.state.clone(),
            self.transport.clone(),
            address,
            count,
        )
    }

    /// `fetch_random_nodes` fetches random nodes from remote.
    pub fn fetch_random_nodes(&mut self, count: u32) -> Result<BTreeSet<Node>> {
        protocol_network::fetch_random_nodes(self.state.clone(), self.transport.clone(), count)
    }

    /// `fetch_missing_ancestors` fetches a `Transaction` ancestors from remote if missing.
    pub fn fetch_missing_ancestors(
        &mut self,
        transaction: &Transaction,
    ) -> Result<BTreeSet<Transaction>> {
        protocol_network::fetch_missing_ancestors(
            self.state.clone(),
            self.transport.clone(),
            transaction,
        )
    }

    /// `query_node` queries a single remote node.
    pub fn query_node(&mut self, address: &[u8], transaction: &Transaction) -> Result<bool> {
        protocol_network::query_node(
            self.state.clone(),
            self.transport.clone(),
            address,
            transaction,
        )
    }

    /// `query` queries remote nodes.
    pub fn query(&mut self, transaction: &Transaction) -> Result<u32> {
        protocol_network::query(self.state.clone(), self.transport.clone(), transaction)
    }

    /// `mine` mines a set of `Transaction`s.
    pub fn mine(&mut self, address: &[u8], transactions: &BTreeSet<Transaction>) -> Result<()> {
        protocol_network::mine(
            self.state.clone(),
            self.transport.clone(),
            address,
            transactions,
        )
    }
}
