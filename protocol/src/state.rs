//! # Protocol State
//!
//! `state` is the module containing the protocol state type and functions.

use crate::error::Error;
use crate::result::Result;
use config::consensus::ConsensusConfig;
use crypto::hash::Digest;
use models::account::Account;
use models::address::Address;
use models::conflict_set::ConflictSet;
use models::consensus_state::ConsensusState;
use models::error::Error as ModelsError;
use models::node::Node;
use models::stage::Stage;
use models::traits::Storable;
use models::transaction::Transaction;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `ProtocolState` is the protocol state type.
#[allow(dead_code)]
#[derive(Debug)]
pub struct ProtocolState<S: Store, P: Store> {
    pub stage: Stage,
    pub address: Vec<u8>,
    pub config: ConsensusConfig,
    pub state: ConsensusState,
    pub store: Arc<Mutex<S>>,
    pub pool: Arc<Mutex<P>>,
}

impl<S: Store, P: Store> ProtocolState<S, P> {
    /// `create` creates a new `ProtocolState` instance, erasing
    /// the previous content of the stores.
    /// The method is equivalent to the "Init" procedure in
    /// the Avalanche paper.
    pub fn create(
        stage: Stage,
        address: &[u8],
        config: &mut ConsensusConfig,
        eve_account: &Account,
        seed: &BTreeSet<Vec<u8>>,
        store: Arc<Mutex<S>>,
        pool: Arc<Mutex<P>>,
    ) -> Result<ProtocolState<S, P>> {
        config.validate()?;
        eve_account.validate()?;

        if !eve_account.is_eve()? {
            let err = Error::InvalidAccount;
            return Err(err);
        }

        if eve_account.stage != stage {
            let err = Error::InvalidStage;
            return Err(err);
        }

        config.populate();

        let mut eve_transaction = Transaction::new_eve(stage, &eve_account.address())?;
        eve_transaction.mine()?;

        store.lock().unwrap().clear()?;
        pool.lock().unwrap().clear()?;

        Account::create(
            &mut *store.lock().unwrap(),
            stage,
            &eve_account.address(),
            &eve_account,
        )?;

        Transaction::create(
            &mut *store.lock().unwrap(),
            stage,
            &eve_transaction.id,
            &eve_transaction,
        )?;

        let mut seed_nodes = BTreeSet::new();

        for address in seed {
            let node = Node::new(stage, address);
            Node::create(&mut *store.lock().unwrap(), stage, &node.id, &node)?;
            seed_nodes.insert(node);
        }

        let state =
            ConsensusState::new(0, stage, &eve_account.address(), &eve_transaction.id, seed);
        ConsensusState::create(&mut *store.lock().unwrap(), stage, &state.id, &state)?;

        let state = ProtocolState {
            stage,
            address: address.to_owned(),
            config: config.to_owned(),
            state,
            store,
            pool,
        };

        Ok(state)
    }

    /// `open` creates a new `ProtocolState` instance from a
    /// populated store.
    pub fn open(
        stage: Stage,
        address: &[u8],
        config: &mut ConsensusConfig,
        store: Arc<Mutex<S>>,
        pool: Arc<Mutex<P>>,
    ) -> Result<ProtocolState<S, P>> {
        config.validate()?;
        config.populate();

        ConsensusState::cleanup(&mut *store.lock().unwrap(), stage, None)?;

        let states = ConsensusState::query(&*store.lock().unwrap(), stage, None, None, None, None)?;

        let max_id: u64 = states.iter().map(|state| state.id).max().unwrap_or(0);

        let mut last_state = ConsensusState::default();

        if states.len() > 1 {
            for state in states {
                if state.id != max_id {
                    ConsensusState::remove(&mut *store.lock().unwrap(), stage, &state.id)?;
                } else {
                    last_state = state;
                }
            }
        } else if states.len() == 1 {
            last_state = states.iter().next().cloned().unwrap();
        } else {
            let err = Error::NotFound;
            return Err(err);
        }

        let state = ProtocolState {
            stage,
            address: address.to_owned(),
            config: config.to_owned(),
            state: last_state,
            store,
            pool,
        };

        Ok(state)
    }

    /// `save` saves the `ProtocolState` state in the store.
    pub fn save(&mut self) -> Result<()> {
        ConsensusState::cleanup(&mut *self.store.lock().unwrap(), self.stage, None)?;

        ConsensusState::create(
            &mut *self.store.lock().unwrap(),
            self.stage,
            &self.state.id,
            &self.state,
        )
        .map_err(|e| e.into())
    }

    /// `set_config` sets a new `ConsensusConfig` in the `ProtocolState`.
    pub fn set_config(&mut self, config: &ConsensusConfig) -> Result<()> {
        config.validate()?;

        self.config = config.to_owned();

        Ok(())
    }

    /// `set_state` sets a new `ConsensusState` in the `ProtocolState`.
    pub fn set_state(&mut self, state: &ConsensusState) -> Result<()> {
        state.validate()?;

        self.state = state.to_owned();

        Ok(())
    }

    /// `get_known_ancestors` returns a `Transaction` known ancestors.
    pub fn get_known_ancestors(&self, tx_id: &Digest) -> Result<BTreeSet<Digest>> {
        let tx = match Transaction::get(&*self.pool.lock().unwrap(), self.stage, tx_id) {
            Ok(tx) => Ok(tx),
            Err(ModelsError::NotFound) => {
                Transaction::get(&*self.store.lock().unwrap(), self.stage, tx_id)
            }
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
        let tx = match Transaction::get(&*self.pool.lock().unwrap(), self.stage, tx_id) {
            Ok(tx) => Ok(tx),
            Err(ModelsError::NotFound) => {
                Transaction::get(&*self.store.lock().unwrap(), self.stage, tx_id)
            }
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

    /// `get_transaction_conflict_set` returns a `Transaction` `ConflictSet`.
    pub fn get_transaction_conflict_set(&self, tx_id: &Digest) -> Result<ConflictSet> {
        if let Some(cs_id) = self.state.get_transaction_conflict_set(tx_id) {
            let cs = ConflictSet::get(&*self.pool.lock().unwrap(), self.stage, &cs_id)?;
            cs.validate()?;

            Ok(cs)
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `calc_confidence` calculates the confidence of a `Transaction`.
    pub fn calc_confidence(&self, tx_id: &Digest) -> Result<u64> {
        let tx_in_pool = Transaction::lookup(&*self.pool.lock().unwrap(), self.stage, tx_id)?;
        let tx_in_store = Transaction::lookup(&*self.store.lock().unwrap(), self.stage, tx_id)?;

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
            if ConflictSet::lookup(&*self.pool.lock().unwrap(), self.stage, &address)? {
                let mut cs = ConflictSet::get(&*self.pool.lock().unwrap(), self.stage, &address)?;
                cs.validate()?;
                cs.transactions.insert(tx_id);
                ConflictSet::update(&mut *self.pool.lock().unwrap(), self.stage, &address, &cs)?;
            } else {
                let mut cs = ConflictSet::new(address, self.stage);
                cs.add_transaction(tx_id);
                cs.count = 0;
                ConflictSet::create(&mut *self.pool.lock().unwrap(), self.stage, &address, &cs)?;
            }
        }

        Ok(())
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
        match Transaction::get(&*self.pool.lock().unwrap(), self.stage, tx_id) {
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
                let found = Transaction::lookup(&*self.store.lock().unwrap(), self.stage, tx_id)?;
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

        match Transaction::get(&*self.pool.lock().unwrap(), self.stage, tx_id) {
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
                let found = Transaction::lookup(&*self.store.lock().unwrap(), self.stage, tx_id)?;
                if found {
                    return Ok(true);
                }
            }
            Err(err) => {
                return Err(err.into());
            }
        }

        if self.config.beta1.is_some() || self.config.beta2.is_some() {
            let cs = self.get_transaction_conflict_set(tx_id)?;

            cs.validate()?;

            if let Some(beta1) = self.config.beta1 {
                if cs.transactions.len() == 1 && cs.count > beta1 {
                    return Ok(true);
                }
            }

            if let Some(beta2) = self.config.beta2 {
                if cs.count > beta2 {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// `sample_nodes` samples a maximum of k nodes from the store.
    pub fn sample_nodes(&mut self) -> Result<BTreeSet<Node>> {
        self.config.populate();
        let count = self.config.k.unwrap();
        Node::sample(&*self.store.lock().unwrap(), self.stage, None, None, count)
            .map_err(|e| e.into())
    }

    /// `random_node` returns a random node.
    pub fn random_node(&self) -> Result<Node> {
        let nodes = Node::sample(&*self.store.lock().unwrap(), self.stage, None, None, 1)?;

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

    /// `clear_state` clears the state of the `ProtocolState`.
    pub fn clear_state(&mut self) {
        self.state.clear()
    }

    /// `clear` clears the state and stores of the `ProtocolState`.
    pub fn clear(&mut self) -> Result<()> {
        self.clear_state();
        self.pool.lock().unwrap().clear()?;
        self.store.lock().unwrap().clear()?;

        Ok(())
    }

    /// `validate` validates the `ProtocolState`.
    pub fn validate(&self) -> Result<()> {
        self.config.validate()?;
        self.state.validate()?;

        Ok(())
    }
}
