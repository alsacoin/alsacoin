//! # Protocol
//!
//! `protocol` is the module containing the type implementing the Avalanche Consensus Protocol.

use crate::error::Error;
use crate::result::Result;
use crypto::hash::Digest;
use models::account::Account;
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
use network::error::Error as NetworkError;
use network::message::Message;
use network::traits::Transport;
use std::collections::BTreeSet;
use store::traits::Store;

/// `Protocol` is the type encapsulating the Avalanche Consensus Protocol.
#[allow(dead_code)]
pub struct Protocol<S: Store, P: Store, T: Transport> {
    stage: Stage,
    address: Vec<u8>,
    params: ConsensusParams,
    state: ConsensusState,
    eve_account: Account,
    eve_transaction: Transaction,
    store: S,
    pool: P,
    transport: T,
}

impl<S: Store, P: Store, T: Transport> Protocol<S, P, T> {
    /// `new` creates a new `Protocol` instance.
    /// The method is equivalent to the "Init" procedure in
    /// the Avalanche paper.
    pub fn new(
        address: &[u8],
        params: &ConsensusParams,
        state: &ConsensusState,
        eve_account: &Account,
        mut store: S,
        pool: P,
        transport: T,
    ) -> Result<Protocol<S, P, T>> {
        params.validate()?;
        state.validate()?;
        eve_account.validate()?;

        if !eve_account.is_eve()? {
            let err = Error::InvalidAccount;
            return Err(err);
        }

        let stage = state.stage;

        if params.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        if eve_account.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        let mut eve_transaction = Transaction::new_eve(stage, &eve_account.address)?;
        eve_transaction.mine()?;

        Account::create(&mut store, stage, &eve_account.address, &eve_account)?;

        Transaction::create(&mut store, stage, &eve_transaction.id, &eve_transaction)?;

        let protocol = Protocol {
            stage: params.stage,
            address: address.to_owned(),
            params: params.to_owned(),
            state: state.to_owned(),
            eve_account: eve_account.to_owned(),
            eve_transaction: eve_transaction.to_owned(),
            store,
            pool,
            transport,
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
            .ancestors()?
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
            .ancestors()?
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
            .send(&address, &data, self.params.timeout)
            .map_err(|e| e.into())
    }

    /// `recv_message` receives a `ConsensusMessage` from a `Node`.
    pub fn recv_message(&mut self) -> Result<ConsensusMessage> {
        let msg = self.transport.recv(self.params.timeout)?;

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
                    .ancestors()?
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
                    .ancestors()?
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
    pub fn sample_nodes(&self) -> Result<BTreeSet<Node>> {
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

        if let Some(node) = nodes.iter().next().cloned() {
            Ok(node)
        } else {
            let err = Error::InvalidLength;
            Err(err)
        }
    }

    /// `handle_node` elaborates an incoming `Node`.
    pub fn handle_node(&mut self, node: &Node) -> Result<()> {
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

    /// `handle_transaction` elaborates an incoming `Node`.
    /// It is equivalent to the `OnReceiveTx` function in the Avalanche paper.
    pub fn handle_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;
        transaction.validate_mined()?;

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

    /// `push_transactions` sends `Transaction`s to a remote node.
    pub fn push_transactions(
        &mut self,
        address: &[u8],
        fetch_id: u64,
        transactions: &BTreeSet<Transaction>,
    ) -> Result<()> {
        let node = Node::new(self.stage, address);
        let cons_msg = ConsensusMessage::new_push_transactions(
            &self.address,
            fetch_id + 1,
            &node,
            transactions,
        )?;
        self.send_message(&cons_msg)
    }

    /// `handle_fetch_transactions` handles a `FetchTransactions` request.
    pub fn handle_fetch_transactions(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::FetchTransactions {
                address,
                id,
                node,
                ids,
                ..
            } => {
                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                let node = Node::new(self.stage, &address);
                self.handle_node(&node)?;

                let mut transactions = BTreeSet::new();

                for id in ids {
                    if Transaction::lookup(&self.store, self.stage, &id)? {
                        let transaction = Transaction::get(&self.store, self.stage, &id)?;
                        transactions.insert(transaction);
                    }
                }

                let cons_msg = ConsensusMessage::new_push_transactions(
                    &self.address,
                    id + 1,
                    &node,
                    &transactions,
                )?;
                self.send_message(&cons_msg)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `handle_fetch_random_transactions` handles a `FetchRandomTransactions` request.
    pub fn handle_fetch_random_transactions(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::FetchRandomTransactions {
                address,
                id,
                node,
                count,
            } => {
                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                let node = Node::new(self.stage, &address);
                self.handle_node(&node)?;

                let transactions = Transaction::sample(&self.store, self.stage, None, None, count)?;

                let cons_msg = ConsensusMessage::new_push_transactions(
                    &self.address,
                    id + 1,
                    &node,
                    &transactions,
                )?;
                self.send_message(&cons_msg)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `handle_push_transactions` handles a `PushTransactions`.
    pub fn handle_push_transactions(
        &mut self,
        msg: &ConsensusMessage,
        prev_id: u64,
        ids: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Transaction>> {
        msg.validate()?;
        let expected_ids = ids;

        if msg.is_push_transactions()?
            && msg.node().address == self.address
            && msg.id() == prev_id + 1
        {
            match msg.to_owned() {
                ConsensusMessage::PushTransactions {
                    ids, transactions, ..
                } => {
                    if !ids.is_subset(&expected_ids) {
                        let err = Error::InvalidMessage;
                        return Err(err);
                    }

                    for transaction in &transactions {
                        self.handle_transaction(&transaction)?;
                    }

                    Ok(transactions)
                }
                _ => {
                    let err = Error::InvalidMessage;
                    Err(err)
                }
            }
        } else {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }

    /// `handle_push_random_transactions` handles a `PushTransactions` following a
    /// `FetchRandomTransactions`.
    pub fn handle_push_random_transactions(
        &mut self,
        msg: &ConsensusMessage,
        fetch_id: u64,
        count: u32,
    ) -> Result<BTreeSet<Transaction>> {
        msg.validate()?;
        let expected_count = count;

        if msg.is_push_transactions()?
            && msg.node().address == self.address
            && msg.id() == fetch_id + 1
        {
            match msg.to_owned() {
                ConsensusMessage::PushTransactions {
                    count,
                    transactions,
                    ..
                } => {
                    if count > expected_count {
                        let err = Error::InvalidMessage;
                        return Err(err);
                    }

                    for transaction in &transactions {
                        self.handle_transaction(&transaction)?;
                    }

                    Ok(transactions)
                }
                _ => {
                    let err = Error::InvalidMessage;
                    Err(err)
                }
            }
        } else {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }

    /// `fetch_node_transactions` fetches transactions from a remote node.
    pub fn fetch_node_transactions(
        &mut self,
        address: &[u8],
        ids: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Transaction>> {
        let node = Node::new(self.stage, address);
        let mut res = BTreeSet::new();

        let cons_msg = ConsensusMessage::new_fetch_transactions(&self.address, &node, ids)?;
        self.send_message(&cons_msg)?;
        let mut max_retries = self.params.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = self.recv_message()?;
            if recv_cons_msg.is_push_transactions()?
                && recv_cons_msg.node().address == self.address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let transactions =
                    self.handle_push_transactions(&recv_cons_msg, cons_msg.id(), ids)?;

                for transaction in transactions {
                    self.handle_transaction(&transaction)?;
                    res.insert(transaction);
                }

                break;
            } else {
                max_retries -= 1;
            }
        }

        Ok(res)
    }

    /// `fetch_transactions` fetches transactions from remote.
    pub fn fetch_transactions(&mut self, ids: &BTreeSet<Digest>) -> Result<BTreeSet<Transaction>> {
        let nodes = self.sample_nodes()?;
        let mut res = BTreeSet::new();

        for node in nodes {
            let cons_msg = ConsensusMessage::new_fetch_transactions(&self.address, &node, ids)?;
            self.send_message(&cons_msg)?;
            let mut max_retries = self.params.max_retries.unwrap_or(1);

            while max_retries > 0 {
                let recv_cons_msg = self.recv_message()?;
                if recv_cons_msg.is_push_transactions()?
                    && recv_cons_msg.node().address == self.address
                    && recv_cons_msg.id() == cons_msg.id() + 1
                {
                    let transactions =
                        self.handle_push_transactions(&recv_cons_msg, cons_msg.id(), ids)?;

                    for transaction in transactions {
                        self.handle_transaction(&transaction)?;
                        res.insert(transaction);
                    }

                    break;
                } else {
                    max_retries -= 1;
                }
            }
        }

        Ok(res)
    }

    /// `fetch_node_random_transactions` fetches random transactions from a remote node.
    pub fn fetch_node_random_transactions(
        &mut self,
        address: &[u8],
        count: u32,
    ) -> Result<BTreeSet<Transaction>> {
        let node = Node::new(self.stage, address);
        let mut res = BTreeSet::new();

        let cons_msg =
            ConsensusMessage::new_fetch_random_transactions(&self.address, &node, count)?;
        self.send_message(&cons_msg)?;
        let mut max_retries = self.params.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = self.recv_message()?;
            if recv_cons_msg.is_push_transactions()?
                && recv_cons_msg.node().address == self.address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let transactions =
                    self.handle_push_random_transactions(&recv_cons_msg, cons_msg.id(), count)?;

                for transaction in transactions {
                    self.handle_transaction(&transaction)?;
                    res.insert(transaction);
                }

                break;
            } else {
                max_retries -= 1;
            }
        }

        Ok(res)
    }

    /// `fetch_random_transactions` fetches random transactions from remote.
    pub fn fetch_random_transactions(&mut self, count: u32) -> Result<BTreeSet<Transaction>> {
        let nodes = self.sample_nodes()?;
        let mut res = BTreeSet::new();

        for node in nodes {
            let cons_msg =
                ConsensusMessage::new_fetch_random_transactions(&self.address, &node, count)?;
            self.send_message(&cons_msg)?;
            let mut max_retries = self.params.max_retries.unwrap_or(1);

            while max_retries > 0 {
                let recv_cons_msg = self.recv_message()?;
                if recv_cons_msg.is_push_transactions()?
                    && recv_cons_msg.node().address == self.address
                    && recv_cons_msg.id() == cons_msg.id() + 1
                {
                    let transactions =
                        self.handle_push_random_transactions(&recv_cons_msg, cons_msg.id(), count)?;

                    for transaction in transactions {
                        self.handle_transaction(&transaction)?;
                        res.insert(transaction);
                    }

                    break;
                } else {
                    max_retries -= 1;
                }
            }
        }

        Ok(res)
    }

    /// `push_nodes` sends `Node`s to a remote node.
    pub fn push_nodes(
        &mut self,
        address: &[u8],
        fetch_id: u64,
        nodes: &BTreeSet<Node>,
    ) -> Result<()> {
        let node = Node::new(self.stage, address);
        let cons_msg = ConsensusMessage::new_push_nodes(&self.address, fetch_id + 1, &node, nodes)?;
        self.send_message(&cons_msg)
    }

    /// `handle_fetch_nodes` handles a `FetchNodes` request.
    pub fn handle_fetch_nodes(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::FetchNodes {
                address,
                id,
                node,
                ids,
                ..
            } => {
                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                let node = Node::new(self.stage, &address);
                self.handle_node(&node)?;

                let mut nodes = BTreeSet::new();

                for id in ids {
                    if Node::lookup(&self.store, self.stage, &id)? {
                        let node = Node::get(&self.store, self.stage, &id)?;
                        nodes.insert(node);
                    }
                }

                let cons_msg =
                    ConsensusMessage::new_push_nodes(&self.address, id + 1, &node, &nodes)?;
                self.send_message(&cons_msg)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `handle_fetch_random_nodes` handles a `FetchRandomNodes` request.
    pub fn handle_fetch_random_nodes(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::FetchRandomNodes {
                address,
                id,
                node,
                count,
            } => {
                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                let node = Node::new(self.stage, &address);
                self.handle_node(&node)?;

                let nodes = Node::sample(&self.store, self.stage, None, None, count)?;

                let cons_msg =
                    ConsensusMessage::new_push_nodes(&self.address, id + 1, &node, &nodes)?;
                self.send_message(&cons_msg)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `handle_push_nodes` handles a `PushNodes`.
    pub fn handle_push_nodes(
        &mut self,
        msg: &ConsensusMessage,
        fetch_id: u64,
        ids: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Node>> {
        msg.validate()?;
        let expected_ids = ids;

        if msg.is_push_nodes()? && msg.node().address == self.address && msg.id() == fetch_id + 1 {
            match msg.to_owned() {
                ConsensusMessage::PushNodes { ids, nodes, .. } => {
                    if !ids.is_subset(&expected_ids) {
                        let err = Error::InvalidMessage;
                        return Err(err);
                    }

                    for node in &nodes {
                        self.handle_node(&node)?;
                    }

                    Ok(nodes)
                }
                _ => {
                    let err = Error::InvalidMessage;
                    Err(err)
                }
            }
        } else {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }

    /// `handle_push_random_nodes` handles a `PushNodes` following a
    /// `FetchRandomNodes`.
    pub fn handle_push_random_nodes(
        &mut self,
        msg: &ConsensusMessage,
        fetch_id: u64,
        count: u32,
    ) -> Result<BTreeSet<Node>> {
        msg.validate()?;
        let expected_count = count;

        if msg.is_push_nodes()? && msg.node().address == self.address && msg.id() == fetch_id + 1 {
            match msg.to_owned() {
                ConsensusMessage::PushNodes { count, nodes, .. } => {
                    if count > expected_count {
                        let err = Error::InvalidMessage;
                        return Err(err);
                    }

                    for node in &nodes {
                        self.handle_node(&node)?;
                    }

                    Ok(nodes)
                }
                _ => {
                    let err = Error::InvalidMessage;
                    Err(err)
                }
            }
        } else {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }

    /// `fetch_node_nodes` fetches nodes from a remote node.
    pub fn fetch_node_nodes(
        &mut self,
        address: &[u8],
        ids: &BTreeSet<Digest>,
    ) -> Result<BTreeSet<Node>> {
        let node = Node::new(self.stage, address);
        let cons_msg = ConsensusMessage::new_fetch_nodes(&self.address, &node, ids)?;
        self.send_message(&cons_msg)?;

        let mut res = BTreeSet::new();
        let mut max_retries = self.params.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = self.recv_message()?;
            if recv_cons_msg.is_push_nodes()?
                && recv_cons_msg.node().address == self.address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let nodes = self.handle_push_nodes(&recv_cons_msg, cons_msg.id(), ids)?;

                for node in nodes {
                    self.handle_node(&node)?;
                    res.insert(node);
                }

                break;
            } else {
                max_retries -= 1;
            }
        }

        Ok(res)
    }

    /// `fetch_nodes` fetches nodes from remote.
    pub fn fetch_nodes(&mut self, ids: &BTreeSet<Digest>) -> Result<BTreeSet<Node>> {
        let nodes = self.sample_nodes()?;
        let mut res = BTreeSet::new();

        for node in nodes {
            let cons_msg = ConsensusMessage::new_fetch_nodes(&self.address, &node, ids)?;
            self.send_message(&cons_msg)?;

            let mut max_retries = self.params.max_retries.unwrap_or(1);

            while max_retries > 0 {
                let recv_cons_msg = self.recv_message()?;
                if recv_cons_msg.is_push_nodes()?
                    && recv_cons_msg.node().address == self.address
                    && recv_cons_msg.id() == cons_msg.id() + 1
                {
                    let nodes = self.handle_push_nodes(&recv_cons_msg, cons_msg.id(), ids)?;

                    for node in nodes {
                        self.handle_node(&node)?;
                        res.insert(node);
                    }

                    break;
                } else {
                    max_retries -= 1;
                }
            }
        }

        Ok(res)
    }

    /// `fetch_node_random_nodes` fetches random nodes from a remote node.
    pub fn fetch_node_random_nodes(
        &mut self,
        address: &[u8],
        count: u32,
    ) -> Result<BTreeSet<Node>> {
        let node = Node::new(self.stage, &address);
        let cons_msg = ConsensusMessage::new_fetch_random_nodes(&self.address, &node, count)?;
        self.send_message(&cons_msg)?;

        let mut max_retries = self.params.max_retries.unwrap_or(1);
        let mut res = BTreeSet::new();

        while max_retries > 0 {
            let recv_cons_msg = self.recv_message()?;
            if recv_cons_msg.is_push_nodes()?
                && recv_cons_msg.node().address == self.address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let nodes = self.handle_push_random_nodes(&recv_cons_msg, cons_msg.id(), count)?;

                for node in nodes {
                    self.handle_node(&node)?;
                    res.insert(node);
                }

                break;
            } else {
                max_retries -= 1;
            }
        }

        Ok(res)
    }

    /// `fetch_random_nodes` fetches random nodes from remote.
    pub fn fetch_random_nodes(&mut self, count: u32) -> Result<BTreeSet<Node>> {
        let nodes = self.sample_nodes()?;
        let mut res = BTreeSet::new();

        for node in nodes {
            let cons_msg = ConsensusMessage::new_fetch_random_nodes(&self.address, &node, count)?;
            self.send_message(&cons_msg)?;

            let mut max_retries = self.params.max_retries.unwrap_or(1);

            while max_retries > 0 {
                let recv_cons_msg = self.recv_message()?;
                if recv_cons_msg.is_push_nodes()?
                    && recv_cons_msg.node().address == self.address
                    && recv_cons_msg.id() == cons_msg.id() + 1
                {
                    let nodes =
                        self.handle_push_random_nodes(&recv_cons_msg, cons_msg.id(), count)?;

                    for node in nodes {
                        self.handle_node(&node)?;
                        res.insert(node);
                    }

                    break;
                } else {
                    max_retries -= 1;
                }
            }
        }

        Ok(res)
    }

    /// `update_successors` updates the set of successors of the ancestor `Transaction`s of the
    /// `Transaction`.
    pub fn update_successors(&mut self, transaction: &Transaction) -> Result<()> {
        transaction.validate()?;

        let id = transaction.id;
        let ancestors = transaction.ancestors()?;
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
            .ancestors()?
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
            let result = self.fetch_node_transactions(&node.address, &to_fetch);

            let txs = if let Ok(txs) = result {
                txs
            } else {
                let mut node = self.random_node()?;
                while node.address == self.address || nodes.contains(&node) {
                    node = self.random_node()?;
                }

                self.fetch_node_transactions(&node.address, &to_fetch)?
            };

            for tx in txs {
                res.insert(tx);
            }
        }

        Ok(res)
    }

    /// `mine` mines a set of `Transaction`s.
    pub fn mine(&mut self, address: &[u8], transactions: &BTreeSet<Transaction>) -> Result<()> {
        for transaction in transactions {
            transaction.validate()?;

            if transaction.is_mined() {
                let err = Error::AlreadyMined;
                return Err(err);
            }
        }

        let node = Node::new(self.stage, address);
        let cons_msg = ConsensusMessage::new_mine(&self.address, &node, transactions)?;
        self.send_message(&cons_msg)
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
        let timeout = self.params.timeout;
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

    /// `update_ancestors` updates the ancestors set of a `Transaction`.
    pub fn update_ancestors(&mut self, transaction: &Transaction) -> Result<()> {
        for ancestor in self.fetch_missing_ancestors(transaction)? {
            self.handle_transaction(&ancestor)?;
        }

        Ok(())
    }

    /// `handle_reply` handles a `Reply` request.
    pub fn handle_reply(
        &mut self,
        msg: &ConsensusMessage,
        query_id: u64,
        transaction_id: &Digest,
    ) -> Result<bool> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::Reply {
                id,
                node,
                tx_id,
                chit,
                ..
            } => {
                if id != query_id + 1 {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                if transaction_id != &tx_id {
                    let err = Error::InvalidId;
                    return Err(err);
                }

                Ok(chit)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `query_node` queries a single remote node.
    pub fn query_node(&mut self, address: &[u8], transaction: &Transaction) -> Result<bool> {
        let node = Node::new(self.stage, address);
        let cons_msg = ConsensusMessage::new_query(&self.address, &node, transaction)?;
        self.send_message(&cons_msg)?;

        let mut res = false;
        let mut max_retries = self.params.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = self.recv_message()?;
            if recv_cons_msg.is_reply()?
                && recv_cons_msg.node().address == self.address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                res = self.handle_reply(&recv_cons_msg, cons_msg.id(), &transaction.id)?;
                break;
            } else {
                max_retries -= 1;
            }
        }

        Ok(res)
    }

    /// `query` queries remote nodes.
    pub fn query(&mut self, transaction: &Transaction) -> Result<u32> {
        let nodes = self.sample_nodes()?;
        let mut res = 0u32;

        for node in nodes {
            let chit = self.query_node(&node.address, transaction)? as u32;
            res += chit;
        }

        Ok(res)
    }

    /// `reply` replies to a `Query` request.
    /// In the Avalanche paper the function is called "OnQuery".
    pub fn reply(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::Query {
                address,
                id,
                node,
                transaction,
            } => {
                if node.address != self.address {
                    let err = Error::InvalidAddress;
                    return Err(err);
                }

                let chit = self.is_strongly_preferred(&transaction.id)?;
                let node = Node::new(self.stage, &address);
                self.handle_node(&node)?;

                let cons_msg =
                    ConsensusMessage::new_reply(&self.address, id, &node, transaction.id, chit)?;
                self.send_message(&cons_msg)
            }
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `handle` handles incoming `ConsensusMessage`s.
    pub fn handle(&mut self, msg: &ConsensusMessage) -> Result<()> {
        msg.validate()?;

        match msg.to_owned() {
            ConsensusMessage::FetchNodes { .. } => self.handle_fetch_nodes(msg),
            ConsensusMessage::FetchRandomNodes { .. } => self.handle_fetch_random_nodes(msg),
            ConsensusMessage::FetchTransactions { .. } => self.handle_fetch_transactions(msg),
            ConsensusMessage::FetchRandomTransactions { .. } => {
                self.handle_fetch_random_transactions(msg)
            }
            ConsensusMessage::Query { .. } => self.reply(msg),
            _ => {
                let err = Error::InvalidMessage;
                Err(err)
            }
        }
    }

    /// `serve_incoming` serves incoming `ConsensusMessage`s.
    pub fn serve_incoming(&mut self) -> Result<()> {
        let timeout = self.params.timeout;
        let mut transport = self.transport.clone();

        transport
            .serve(timeout, |msg| {
                let cons_msg = msg.to_consensus_message()?;

                self.handle(&cons_msg).map_err(|e| NetworkError::Consensus {
                    msg: format!("{}", e),
                })
            })
            .map_err(|e| e.into())
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
                self.handle_transaction(&missing_tx)?;
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

                Transaction::insert(&mut self.store, self.stage, &tx_id, &tx)?;
            } else {
                let ancestors: BTreeSet<Digest> = tx
                    .ancestors()?
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

    /// `serve_avalanche` serves the main loop of the `Protocol`.
    /// The name of the function in the Avalanche paper is "AvalancheLoop".
    pub fn serve_avalanche(&mut self) -> Result<()> {
        let mut res = Ok(());

        while res.is_ok() {
            res = self.avalanche_step();
        }

        res
    }
}
