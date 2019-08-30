//! # Protocol
//!
//! `protocol` is the module containing the type implementing the Avalanche Consensus Protocol.

use crate::error::Error;
use crate::result::Result;
use crypto::hash::Digest;
use models::conflict_set::ConflictSet;
use models::consensus_message::ConsensusMessage;
use models::consensus_params::ConsensusParams;
use models::consensus_state::ConsensusState;
use models::error::Error as ModelsError;
use models::node::Node;
use models::stage::Stage;
use models::traits::Storable;
use models::transaction::Transaction;
use network::message::Message;
use network::traits::Transport;
use std::collections::BTreeSet;
use store::traits::Store;

/// `Protocol` is the type encapsulating the Avalanche Consensus Protocol.
pub struct Protocol<S: Store, P: Store, T: Transport> {
    stage: Stage,
    address: Vec<u8>,
    params: ConsensusParams,
    state: ConsensusState,
    store: S,
    pool: P,
    transport: T,
    timeout: Option<u64>,
}

impl<S: Store, P: Store, T: Transport> Protocol<S, P, T> {
    /// `new` creates a new `Protocol` instance.
    /// The method is equivalent to the "Init" procedure in
    /// the Avalanche paper.
    pub fn new(
        address: &[u8],
        params: &ConsensusParams,
        state: &ConsensusState,
        store: S,
        pool: P,
        transport: T,
        timeout: Option<u64>,
    ) -> Result<Protocol<S, P, T>> {
        params.validate()?;
        state.validate()?;

        if params.stage != state.stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        let protocol = Protocol {
            stage: params.stage,
            address: address.to_owned(),
            params: params.to_owned(),
            state: state.to_owned(),
            store,
            pool,
            transport,
            timeout,
        };

        Ok(protocol)
    }

    /// `set_params` sets a new `ConsensusParams` in the `Protocol`.
    pub fn set_params(&mut self, params: &ConsensusParams) -> Result<()> {
        params.validate()?;

        self.params = params.to_owned();

        Ok(())
    }

    /// `set_state` sets a new `ConsensusState` in the `Protocol`.
    pub fn set_state(&mut self, state: &ConsensusState) -> Result<()> {
        state.validate()?;

        self.state = state.to_owned();

        Ok(())
    }

    /// `clear` clears the state of the `Protocol`.
    pub fn clear(&mut self) {
        self.state.clear()
    }

    /// `validate` validates the `Protocol`.
    pub fn validate(&self) -> Result<()> {
        self.params.validate()?;
        self.state.validate()?;

        Ok(())
    }

    /// `send_message` sends a `ConsensusMessage` to a `Node`.
    pub fn send_message(&mut self, cons_msg: &ConsensusMessage) -> Result<()> {
        cons_msg.validate()?;

        let address = cons_msg.node().address;
        let msg = Message::from_consensus_message(cons_msg)?;
        let data = msg.to_bytes()?;

        self.transport
            .send(&address, &data, self.timeout)
            .map_err(|e| e.into())
    }

    /// `recv_message` receives a `ConsensusMessage` from a `Node`.
    pub fn recv_message(&mut self) -> Result<ConsensusMessage> {
        let msg = self.transport.recv(self.timeout)?;

        msg.to_consensus_message().map_err(|e| e.into())
    }

    /// `is_preferred` returns if a `Transaction` is preferred.
    /// The name of the function in the Avalanche paper is "IsPreferred".
    pub fn is_preferred(&self, tx_id: &Digest) -> Result<bool> {
        if let Some(cs_id) = self.state.get_transaction_conflict_set(tx_id) {
            let cs = ConflictSet::get(&self.pool, self.stage, &cs_id)?;
            if let Some(pref_id) = cs.preferred {
                let is_pref = tx_id == &pref_id;
                Ok(is_pref)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    /// `is_strongly_preferred` returns if a `Transaction` is strongly preferred.
    /// The name of the function in the Avalanche paper is "IsStronglyPreferred".
    pub fn is_strongly_preferred(&self, tx_id: &Digest) -> Result<bool> {
        match Transaction::get(&self.pool, self.stage, tx_id) {
            Ok(tx) => {
                let ancestors: BTreeSet<Digest> = tx
                    .ancestors()
                    .iter()
                    .filter(|id| self.state.lookup_known_transaction(&id))
                    .copied()
                    .collect();

                for tx_id in ancestors {
                    if !self.is_preferred(&tx_id)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            Err(ModelsError::NotFound) => {
                // if it was stored in the store it was
                // accepted (chit = 1)
                let found = Transaction::lookup(&self.store, self.stage, tx_id)?;
                Ok(found)
            }
            Err(err) => Err(err.into()),
        }
    }

    /// `is_accepted` returns if a `Transaction` is accepted.
    /// The name of the function in the Avalanche paper is "IsAccepted".
    pub fn is_accepted(&self, tx_id: &Digest) -> Result<bool> {
        let chit = self.state.get_transaction_chit(tx_id).unwrap_or(false);

        if chit {
            return Ok(true);
        }

        match Transaction::get(&self.pool, self.stage, tx_id) {
            Ok(tx) => {
                let ancestors: BTreeSet<Digest> = tx
                    .ancestors()
                    .iter()
                    .filter(|id| self.state.lookup_known_transaction(&id))
                    .copied()
                    .collect();

                let mut accepted = true;

                for tx_id in ancestors {
                    if !self.is_accepted(&tx_id)? {
                        accepted = false;
                    }
                }

                if accepted {
                    return Ok(accepted);
                }
            }
            Err(ModelsError::NotFound) => {
                // if it was stored in the store it was
                // accepted (chit = 1)
                let found = Transaction::lookup(&self.store, self.stage, tx_id)?;
                if found {
                    return Ok(true);
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        if self.params.beta1.is_some() || self.params.beta2.is_some() {
            let cs_id = self.state.get_transaction_conflict_set(tx_id);
            if cs_id.is_none() {
                let err = Error::NotFound;
                return Err(err);
            }

            let cs_id = cs_id.unwrap();

            let cs = ConflictSet::get(&self.pool, self.stage, &cs_id)?;

            if let Some(beta1) = self.params.beta1 {
                if cs.transactions.len() == 1 && cs.count > beta1 {
                    return Ok(true);
                }
            }

            if let Some(beta2) = self.params.beta2 {
                if cs.count > beta2 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// `calc_confidence` calculates the confidence of a `Transaction`.
    pub fn calc_confidence(&self, tx_id: &Digest) -> Result<u64> {
        let tx_in_pool = Transaction::lookup(&self.pool, self.stage, tx_id)?;
        let tx_in_store = Transaction::lookup(&self.store, self.stage, tx_id)?;

        if tx_in_pool || tx_in_store {
            let confidence = if let Some(successors) = self.state.get_successors(tx_id) {
                let mut confidence = 0;

                for succ_id in successors {
                    if !self.state.lookup_known_transaction(&succ_id) {
                        let err = ModelsError::NotFound;
                        return Err(err.into());
                    }

                    let chit = self.state.get_transaction_chit(&succ_id).unwrap_or(false) as u64;

                    confidence += chit;
                }

                confidence += self.state.get_transaction_chit(tx_id).unwrap_or(false) as u64;

                confidence
            } else {
                0
            };

            Ok(confidence)
        } else {
            let err = ModelsError::NotFound;
            Err(err.into())
        }
    }

    /// `update_confidence` updates the confidence of a `Transaction`.
    pub fn update_confidence(&mut self, tx_id: &Digest) -> Result<()> {
        let confidence = self.calc_confidence(tx_id)?;

        self.state
            .set_transaction_confidence(*tx_id, confidence)
            .map_err(|e| e.into())
    }

    /// `sample_nodes` samples a maximum of k nodes from the store.
    pub fn sample_nodes(&self) -> Result<Vec<Node>> {
        let count = self.params.k;
        Node::sample(&self.store, self.stage, None, None, count).map_err(|e| e.into())
    }

    /// `random_node` returns a random node.
    pub fn random_node(&self) -> Result<Node> {
        let nodes = Node::sample(&self.store, self.stage, None, None, 1)?;

        if nodes.len() != 1 {
            let err = ModelsError::InvalidLength;
            return Err(err.into());
        }

        let node = nodes[0].clone();

        Ok(node)
    }

    /// `push_node` sends a `Node` to a remote node.
    pub fn push_node(&mut self, _address: &[u8], _node: &Node) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `on_push_node` handles a `PushNode` request.
    pub fn on_push_node(&mut self, _msg: ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `push_transaction` sends a `Transaction` to a remote transaction.
    pub fn push_transaction(&mut self, _address: &[u8], _transaction: &Transaction) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `on_push_transaction` handles a `PushTransaction`.
    /// It is equivalent to the `OnReceiveTx` function in the Avalanche paper.
    pub fn on_push_transaction(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_nodes` fetches nodes from remote.
    pub fn fetch_nodes(
        &mut self,
        _address: &[u8],
        _nodes: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Node>> {
        // TODO
        // NB: use k as *maxnodes*
        unreachable!()
    }

    /// `on_fetch_nodes` handles a `FetchNodes` request.
    pub fn on_fetch_nodes(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_random_nodes` fetches random nodes from remote.
    pub fn fetch_random_nodes(&mut self, _address: &[u8], _count: u32) -> Result<BTreeSet<Node>> {
        // TODO
        // NB: use k as *maxnodes*
        unreachable!()
    }

    /// `on_fetch_random_nodes` handles a `FetchRandomNodes` request.
    pub fn on_fetch_random_nodes(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_transactions` fetches transactions from remote.
    pub fn fetch_transactions(
        &mut self,
        _address: &[u8],
        _transactions: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Transaction>> {
        // TODO
        // NB: use k as *maxtransactions*
        unreachable!()
    }

    /// `on_fetch_transactions` handles a `FetchTransactions` request.
    pub fn on_fetch_transactions(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_random_transactions` fetches random transactions from remote.
    pub fn fetch_random_transactions(&mut self, _count: u32) -> Result<BTreeSet<Transaction>> {
        // TODO
        // NB: use k as *maxtransactions*
        unreachable!()
    }

    /// `on_fetch_random_transactions` handles a `FetchRandomTransactions` request.
    pub fn on_fetch_random_transactions(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_ancestors` fetches a `Transaction` ancestors from remote if missing.
    pub fn fetch_ancestors(&mut self, transaction: &Transaction) -> Result<BTreeSet<Transaction>> {
        transaction.validate()?;

        let to_fetch: BTreeSet<Digest> = transaction
            .ancestors()
            .iter()
            .filter(|id| !self.state.lookup_known_transaction(&id))
            .copied()
            .collect();

        if to_fetch.is_empty() {
            return Ok(BTreeSet::new());
        }

        let nodes = self.sample_nodes()?;
        let mut res = BTreeSet::new();

        for node in &nodes {
            let result = self.fetch_transactions(&node.address, &to_fetch);

            let txs = if let Ok(txs) = result {
                txs
            } else {
                let mut node = self.random_node()?;
                while node.address == self.address || nodes.contains(&node) {
                    node = self.random_node()?;
                }

                self.fetch_transactions(&node.address, &to_fetch)?
            };

            for tx in txs {
                res.insert(tx);
            }
        }

        Ok(res)
    }

    /// `query` queries remote nodes.
    pub fn query(&mut self, _transaction: &Transaction) -> Result<Vec<bool>> {
        // TODO
        // NB: use k as *maxnodes*
        unreachable!()
    }

    /// `reply` replies to a `Query` request.
    /// In the Avalanche paper the function is called "OnQuery".
    pub fn reply(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        // NB: use k as *maxnodes*
        unreachable!()
    }

    /// `handle` handles incoming `ConsensusMessage`s.
    pub fn handle(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `serve` serves incoming `ConsensusMessage`s.
    pub fn serve(&mut self) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `avalanche_loop` executes the main loop of the `Protocol`.
    /// The name of the function in the Avalanche paper is "AvalancheLoop".
    pub fn avalanche_loop(&mut self) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `run` runs the `Protocol`.
    pub fn run(&mut self) -> Result<()> {
        // TODO
        unreachable!()
    }
}
