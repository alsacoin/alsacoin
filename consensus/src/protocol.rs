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
use models::stage::Stage;
use models::traits::Storable;
use models::transaction::Transaction;
use network::message::Message;
use network::traits::Transport;
use store::traits::Store;

/// `Protocol` is the type encapsulating the Avalanche Consensus Protocol.
pub struct Protocol<S: Store, P: Store, T: Transport> {
    stage: Stage,
    params: ConsensusParams,
    state: ConsensusState,
    _store: S,
    pool: P,
    transport: T,
    timeout: Option<u64>,
}

impl<S: Store, P: Store, T: Transport> Protocol<S, P, T> {
    /// `new` creates a new `Protocol` instance.
    /// The method is equivalent to the "Init" procedure in
    /// the Avalanche paper.
    pub fn new(
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
            params: params.to_owned(),
            state: state.to_owned(),
            _store: store,
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

    /// `on_new_transaction` handles a new `Transaction`.
    /// It is equivalent to the `OnGenerateTx` of the Avalanche paper.
    pub fn on_new_transaction(&mut self, _transaction: &Transaction) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_nodes` fetches nodes from remote.
    pub fn fetch_nodes(&mut self) -> Result<()> {
        // TODO
        // NB: use k as *maxnodes*
        unreachable!()
    }

    /// `on_fetch_nodes` handles a `FetchNodes` request.
    /// It is equivalent to the `OnReceiveTx` function in the Avalanche paper.
    pub fn on_fetch_nodes(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_random_nodes` fetches random nodes from remote.
    pub fn fetch_random_nodes(&mut self) -> Result<()> {
        // TODO
        // NB: use k as *maxnodes*
        unreachable!()
    }

    /// `on_fetch_random_nodes` handles a `FetchRandomNodes` request.
    /// It is equivalent to the `OnReceiveTx` function in the Avalanche paper.
    pub fn on_fetch_random_nodes(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `fetch_transactions` fetches transactions from remote.
    pub fn fetch_transactions(&mut self) -> Result<()> {
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
    pub fn fetch_random_transactions(&mut self) -> Result<()> {
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
    pub fn fetch_ancestors(&mut self, _transaction: &Transaction) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `query` queries remote nodes.
    pub fn query(&mut self) -> Result<()> {
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
    pub fn is_strongly_preferred(&self, _tx_id: &Digest) -> Result<bool> {
        // TODO
        unreachable!()
    }

    /// `is_accepted` returns if a `Transaction` is accepted.
    /// The name of the function in the Avalanche paper is "IsAccepted".
    pub fn is_accepted(&self, _tx_id: &Digest) -> Result<bool> {
        // TODO
        unreachable!()
    }

    /// `update_chit` updates the chit of a `Transaction`.
    pub fn update_chit(&mut self, _tx_id: &Digest) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `update_confidence` updates the confidence of a `Transaction`.
    pub fn update_confidence(&mut self, _tx_id: &Digest) -> Result<()> {
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
