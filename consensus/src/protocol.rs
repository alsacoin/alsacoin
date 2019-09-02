//! # Protocol
//!
//! `protocol` is the module containing the type implementing the Avalanche Consensus Protocol.

use crate::error::Error;
use crate::result::Result;
use crypto::hash::Digest;
use models::address::Address;
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

    /// `get_known_ancestors` returns a `Transaction` known ancestors.
    pub fn get_known_ancestors(&self, tx_id: &Digest) -> Result<BTreeSet<Digest>> {
        let tx = match Transaction::get(&self.pool, self.stage, tx_id) {
            Ok(tx) => Ok(tx),
            Err(ModelsError::NotFound) => Transaction::get(&self.store, self.stage, tx_id),
            Err(e) => Err(e),
        }?;

        tx.validate()?;

        let ancestors: BTreeSet<Digest> = tx
            .ancestors()
            .iter()
            .filter(|id| self.state.lookup_known_transaction(&id))
            .copied()
            .collect();

        Ok(ancestors)
    }

    /// `get_unknown_ancestors` returns the unknown ancestors of a `Transactions`.
    pub fn get_unknown_ancestors(&self, tx_id: &Digest) -> Result<BTreeSet<Digest>> {
        let tx = match Transaction::get(&self.pool, self.stage, tx_id) {
            Ok(tx) => Ok(tx),
            Err(ModelsError::NotFound) => Transaction::get(&self.store, self.stage, tx_id),
            Err(e) => Err(e),
        }?;

        tx.validate()?;

        let ancestors: BTreeSet<Digest> = tx
            .ancestors()
            .iter()
            .filter(|id| !self.state.lookup_known_transaction(&id))
            .copied()
            .collect();

        Ok(ancestors)
    }

    /// `get_transaction_conflict_set` returns a `Transaction` `ConflictSet`.
    pub fn get_transaction_conflict_set(&self, tx_id: &Digest) -> Result<ConflictSet> {
        if let Some(cs_id) = self.state.get_transaction_conflict_set(tx_id) {
            let cs = ConflictSet::get(&self.pool, self.stage, &cs_id)?;
            cs.validate()?;

            Ok(cs)
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `clear_state` clears the state of the `Protocol`.
    pub fn clear_state(&mut self) {
        self.state.clear()
    }

    /// `clear` clears the state and stores of the `Protocol`.
    pub fn clear(&mut self) -> Result<()> {
        self.clear_state();
        self.pool.clear()?;
        self.store.clear()?;

        Ok(())
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
        let cs = self.get_transaction_conflict_set(tx_id)?;
        cs.validate()?;

        if let Some(pref_id) = cs.preferred {
            let is_pref = tx_id == &pref_id;
            Ok(is_pref)
        } else {
            Ok(false)
        }
    }

    /// `is_strongly_preferred` returns if a `Transaction` is strongly preferred.
    /// The name of the function in the Avalanche paper is "IsStronglyPreferred".
    pub fn is_strongly_preferred(&self, tx_id: &Digest) -> Result<bool> {
        match Transaction::get(&self.pool, self.stage, tx_id) {
            Ok(tx) => {
                tx.validate()?;

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
                tx.validate()?;

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
            let cs = self.get_transaction_conflict_set(tx_id)?;

            cs.validate()?;

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
                        let err = Error::NotFound;
                        return Err(err);
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
            let err = Error::NotFound;
            Err(err)
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
            let err = Error::InvalidLength;
            return Err(err);
        }

        let node = nodes[0].clone();

        Ok(node)
    }

    /// `on_node` elaborates an incoming `Node`.
    pub fn on_node(&mut self, node: &Node) -> Result<()> {
        node.validate()?;

        if node.address == self.address {
            let err = Error::InvalidNode;
            return Err(err);
        }

        if !Node::lookup(&self.store, self.stage, &node.id)? {
            Node::create(&mut self.store, self.stage, &node.id, &node)?;
            self.state.add_known_node(node.id);
        } else {
            let known_node = Node::get(&self.store, self.stage, &node.id)?;
            if known_node.last_seen < node.last_seen {
                Node::update(&mut self.store, self.stage, &node.id, &node)?;
            }

            if !self.state.lookup_known_node(&node.id) {
                self.state.add_known_node(node.id);
            }
        }

        Ok(())
    }

    /// `upsert_conflict_sets` upserts the `ConsensusState` conflict sets.
    pub fn upsert_conflict_sets(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;

        let tx_id = transaction.id;
        let addresses: BTreeSet<Address> = transaction
            .outputs
            .values()
            .map(|out| out.address)
            .collect();

        for address in addresses {
            if ConflictSet::lookup(&self.pool, self.stage, &address)? {
                let mut cs = ConflictSet::get(&self.pool, self.stage, &address)?;
                cs.validate()?;
                cs.transactions.insert(tx_id);
                ConflictSet::update(&mut self.pool, self.stage, &address, &cs)?;
            } else {
                let mut cs = ConflictSet::new(address, self.stage);
                cs.add_transaction(tx_id);
                cs.count = 0;
                ConflictSet::create(&mut self.pool, self.stage, &address, &cs)?;
            }
        }

        Ok(())
    }

    /// `on_transaction` elaborates an incoming `Node`.
    /// It is equivalent to the `OnReceiveTx` function in the Avalanche paper.
    pub fn on_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;
        let tx_id = transaction.id;

        // NB: state may have been cleared, so the first places to check are the stores

        if !Transaction::lookup(&self.pool, self.stage, &tx_id)?
            && !Transaction::lookup(&self.store, self.stage, &tx_id)?
        {
            Transaction::create(&mut self.pool, self.stage, &tx_id, &transaction)?;
            self.state.add_known_transaction(tx_id);

            self.upsert_conflict_sets(&transaction)?;

            self.state.set_transaction_chit(tx_id, false)?;
            self.state.set_transaction_confidence(tx_id, 0)?;

            self.update_ancestors(transaction)?;
            self.update_successors(transaction)?;
        }

        Ok(())
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
        unreachable!()
    }

    /// `on_fetch_random_transactions` handles a `FetchRandomTransactions` request.
    pub fn on_fetch_random_transactions(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `update_successors` updates the set of successors of the ancestor `Transaction`s of the
    /// `Transaction`.
    pub fn update_successors(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;

        let id = transaction.id;
        let ancestors = transaction.ancestors();
        for anc_id in ancestors {
            self.state.add_successor(&anc_id, id)?;
        }

        Ok(())
    }

    /// `fetch_missing_ancestors` fetches a `Transaction` ancestors from remote if missing.
    pub fn fetch_missing_ancestors(
        &mut self,
        transaction: &Transaction,
    ) -> Result<BTreeSet<Transaction>> {
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

        // TODO: on other threads
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

    /// `update_ancestors` updates the ancestors set of a `Transaction`.
    pub fn update_ancestors(&mut self, transaction: &Transaction) -> Result<()> {
        // TODO: threads
        for ancestor in self.fetch_missing_ancestors(transaction)? {
            self.on_transaction(&ancestor)?;
        }

        Ok(())
    }

    /// `reply` replies to a `Query` request.
    /// In the Avalanche paper the function is called "OnQuery".
    pub fn reply(&mut self, _msg: &ConsensusMessage) -> Result<()> {
        // TODO
        unreachable!()
    }

    /// `query_node` queries a single remote node.
    pub fn query_node(&mut self, _address: &[u8], _transaction: &Transaction) -> Result<bool> {
        // TODO
        unreachable!()
    }

    /// `query` queries remote nodes.
    pub fn query(&mut self, _transaction: &Transaction) -> Result<u32> {
        // TODO
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

    /// `avalanche_step` is a single execution of the main Avalanche Consensus procedure.
    pub fn avalanche_step(&mut self) -> Result<()> {
        let tx_ids: BTreeSet<Digest> = self
            .state
            .known_transactions
            .iter()
            .filter(|id| !self.state.lookup_queried_transaction(&id))
            .copied()
            .collect();

        for tx_id in tx_ids {
            let tx = match Transaction::get(&self.pool, self.stage, &tx_id) {
                Ok(tx) => {
                    tx.validate()?;
                    Ok(tx)
                }
                Err(ModelsError::NotFound) => {
                    let tx = Transaction::get(&self.store, self.stage, &tx_id)?;
                    tx.validate()?;
                    Ok(tx)
                }
                Err(err) => Err(err),
            }?;

            let missing_txs = self.fetch_missing_ancestors(&tx)?;

            for missing_tx in missing_txs.iter() {
                self.on_transaction(&missing_tx)?;
            }

            let chit_sum = self.query(&tx)?;

            if chit_sum >= self.params.alpha {
                self.state.set_transaction_chit(tx_id, true)?;

                let mut cs = if let Some(cs_id) = self.state.get_transaction_conflict_set(&tx_id) {
                    ConflictSet::get(&self.pool, self.stage, &cs_id)
                } else {
                    let err = ModelsError::NotFound;
                    Err(err)
                }?;

                cs.validate()?;

                self.update_confidence(&tx_id)?;

                if cs.preferred.is_none() || cs.last.is_none() {
                    let err = Error::NotFound;
                    return Err(err);
                }

                let pref_id = cs.preferred.unwrap();
                let last_id = cs.last.unwrap();

                let pref_confidence = self.state.get_transaction_confidence(&pref_id).unwrap_or(0);

                let confidence = self.state.get_transaction_confidence(&tx_id).unwrap_or(0);

                if confidence > pref_confidence {
                    cs.preferred = Some(tx_id);
                }

                if tx_id != last_id {
                    cs.last = Some(tx_id);
                    cs.count = 1;
                } else {
                    cs.count += 1;
                }

                ConflictSet::update(&mut self.pool, self.stage, &cs.address, &cs)?;
            } else {
                let ancestors: BTreeSet<Digest> = tx
                    .ancestors()
                    .iter()
                    .filter(|id| self.state.lookup_known_transaction(&id))
                    .copied()
                    .collect();

                for tx_id in ancestors {
                    if let Some(cs_id) = self.state.get_transaction_conflict_set(&tx_id) {
                        let mut cs = ConflictSet::get(&self.pool, self.stage, &cs_id)?;
                        cs.validate()?;
                        cs.count = 0;

                        ConflictSet::update(&mut self.pool, self.stage, &cs_id, &cs)?;
                    } else {
                        let err = Error::NotFound;
                        return Err(err);
                    }
                }
            }

            self.state.add_queried_transaction(tx.id)?;
        }

        Ok(())
    }

    /// `avalanche_loop` executes the main loop of the `Protocol`.
    /// The name of the function in the Avalanche paper is "AvalancheLoop".
    pub fn avalanche_loop(&mut self) -> Result<()> {
        let mut res = Ok(());

        while res.is_ok() {
            res = self.avalanche_step();
        }

        res
    }
}
