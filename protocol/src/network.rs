//! # Network
//!
//! `network` contains the network functionalities used in the module.

use crate::error::Error;
use crate::result::Result;
use crate::state::ProtocolState;
use crypto::hash::Digest;
use models::conflict_set::ConflictSet;
use models::consensus_message::ConsensusMessage;
use models::error::Error as ModelsError;
use models::node::Node;
use models::traits::Storable;
use models::transaction::Transaction;
use network::error::Error as NetworkError;
use network::message::Message;
use network::traits::Transport;
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};
use store::traits::Store;

/// `send_message` sends a `ConsensusMessage` to a `Node`.
pub fn send_message<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    cons_msg: &ConsensusMessage,
) -> Result<()> {
    cons_msg.validate()?;

    let address = cons_msg.node().address;
    let msg = Message::from_consensus_message(cons_msg)?;
    let data = msg.to_bytes()?;

    transport
        .lock()
        .unwrap()
        .send(&address, &data, state.lock().unwrap().config.timeout)
        .map_err(|e| e.into())
}

/// `recv_message` receives a `ConsensusMessage` from a `Node`.
pub fn recv_message<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
) -> Result<ConsensusMessage> {
    let msg = transport
        .lock()
        .unwrap()
        .recv(state.lock().unwrap().config.timeout)?;

    msg.to_consensus_message().map_err(|e| e.into())
}

/// `handle_node` elaborates an incoming `Node`.
pub fn handle_node<S: Store, P: Store>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    node: &Node,
) -> Result<()> {
    node.validate()?;

    if node.address == state.lock().unwrap().address {
        let err = Error::InvalidNode;
        return Err(err);
    }

    if !Node::lookup(
        &*state.lock().unwrap().store.lock().unwrap(),
        state.lock().unwrap().stage,
        &node.id,
    )? {
        Node::create(
            &mut *state.lock().unwrap().store.lock().unwrap(),
            state.lock().unwrap().stage,
            &node.id,
            &node,
        )?;
        state.lock().unwrap().state.add_known_node(node.id);
    } else {
        let known_node = Node::get(
            &*state.lock().unwrap().store.lock().unwrap(),
            state.lock().unwrap().stage,
            &node.id,
        )?;
        if known_node.last_seen < node.last_seen {
            Node::update(
                &mut *state.lock().unwrap().store.lock().unwrap(),
                state.lock().unwrap().stage,
                &node.id,
                &node,
            )?;
        }

        if !state.lock().unwrap().state.lookup_known_node(&node.id) {
            state.lock().unwrap().state.add_known_node(node.id);
        }
    }

    Ok(())
}

/// `push_transactions` sends `Transaction`s to a remote node.
pub fn push_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    fetch_id: u64,
    transactions: &BTreeSet<Transaction>,
) -> Result<()> {
    let stage = state.lock().unwrap().stage;
    let node = Node::new(stage, address);

    let cons_msg = ConsensusMessage::new_push_transactions(
        &*state.lock().unwrap().address,
        fetch_id + 1,
        &node,
        transactions,
    )?;

    send_message(state, transport, &cons_msg)
}

/// `handle_fetch_transactions` handles a `FetchTransactions` request.
pub fn handle_fetch_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::FetchTransactions {
            address,
            id,
            node,
            ids,
            ..
        } => {
            if node.address != state.lock().unwrap().address {
                let err = Error::InvalidAddress;
                return Err(err);
            }

            let node = Node::new(state.lock().unwrap().stage, &address);
            handle_node(state.clone(), &node)?;

            let mut transactions = BTreeSet::new();

            for id in ids {
                if Transaction::lookup(
                    &*state.lock().unwrap().store.lock().unwrap(),
                    state.lock().unwrap().stage,
                    &id,
                )? {
                    let transaction = Transaction::get(
                        &*state.lock().unwrap().store.lock().unwrap(),
                        state.lock().unwrap().stage,
                        &id,
                    )?;
                    transactions.insert(transaction);
                }
            }

            let cons_msg = ConsensusMessage::new_push_transactions(
                &*state.lock().unwrap().address,
                id + 1,
                &node,
                &transactions,
            )?;
            send_message(state, transport, &cons_msg)
        }
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `handle_fetch_random_transactions` handles a `FetchRandomTransactions` request.
pub fn handle_fetch_random_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::FetchRandomTransactions {
            address,
            id,
            node,
            count,
            ..
        } => {
            if node.address != state.lock().unwrap().address {
                let err = Error::InvalidAddress;
                return Err(err);
            }

            let node = Node::new(state.lock().unwrap().stage, &address);
            handle_node(state.clone(), &node)?;

            let transactions = Transaction::sample(
                &*state.lock().unwrap().store.lock().unwrap(),
                state.lock().unwrap().stage,
                None,
                None,
                count,
            )?;

            let cons_msg = ConsensusMessage::new_push_transactions(
                &*state.lock().unwrap().address,
                id + 1,
                &node,
                &transactions,
            )?;
            send_message(state, transport, &cons_msg)
        }
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `handle_push_transactions` handles a `PushTransactions`.
pub fn handle_push_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
    prev_id: u64,
    ids: &BTreeSet<Digest>,
) -> Result<BTreeSet<Transaction>> {
    msg.validate()?;
    let expected_ids = ids;

    if msg.is_push_transactions()?
        && msg.node().address == state.lock().unwrap().address
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
                    handle_transaction(state.clone(), transport.clone(), &transaction)?;
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
pub fn handle_push_random_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
    fetch_id: u64,
    count: u32,
) -> Result<BTreeSet<Transaction>> {
    msg.validate()?;
    let expected_count = count;

    if msg.is_push_transactions()?
        && msg.node().address == state.lock().unwrap().address
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
                    handle_transaction(state.clone(), transport.clone(), &transaction)?;
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
pub fn fetch_node_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    ids: &BTreeSet<Digest>,
) -> Result<BTreeSet<Transaction>> {
    let node = Node::new(state.lock().unwrap().stage, address);
    let mut res = BTreeSet::new();

    let cons_msg =
        ConsensusMessage::new_fetch_transactions(&*state.lock().unwrap().address, &node, ids)?;
    send_message(state.clone(), transport.clone(), &cons_msg)?;
    let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

    while max_retries > 0 {
        let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
        if recv_cons_msg.is_push_transactions()?
            && recv_cons_msg.node().address == state.lock().unwrap().address
            && recv_cons_msg.id() == cons_msg.id() + 1
        {
            let transactions = handle_push_transactions(
                state.clone(),
                transport.clone(),
                &recv_cons_msg,
                cons_msg.id(),
                ids,
            )?;

            for transaction in transactions {
                handle_transaction(state.clone(), transport.clone(), &transaction)?;
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
pub fn fetch_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    ids: &BTreeSet<Digest>,
) -> Result<BTreeSet<Transaction>> {
    let nodes = state.lock().unwrap().sample_nodes()?;
    let mut res = BTreeSet::new();

    for node in nodes {
        let cons_msg =
            ConsensusMessage::new_fetch_transactions(&*state.lock().unwrap().address, &node, ids)?;
        send_message(state.clone(), transport.clone(), &cons_msg)?;
        let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
            if recv_cons_msg.is_push_transactions()?
                && recv_cons_msg.node().address == state.lock().unwrap().address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let transactions = handle_push_transactions(
                    state.clone(),
                    transport.clone(),
                    &recv_cons_msg,
                    cons_msg.id(),
                    ids,
                )?;

                for transaction in transactions {
                    handle_transaction(state.clone(), transport.clone(), &transaction)?;
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
pub fn fetch_node_random_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    count: u32,
) -> Result<BTreeSet<Transaction>> {
    let node = Node::new(state.lock().unwrap().stage, address);
    let mut res = BTreeSet::new();

    let cons_msg = ConsensusMessage::new_fetch_random_transactions(
        &*state.lock().unwrap().address,
        &node,
        count,
    )?;
    send_message(state.clone(), transport.clone(), &cons_msg)?;
    let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

    while max_retries > 0 {
        let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
        if recv_cons_msg.is_push_transactions()?
            && recv_cons_msg.node().address == state.lock().unwrap().address
            && recv_cons_msg.id() == cons_msg.id() + 1
        {
            let transactions = handle_push_random_transactions(
                state.clone(),
                transport.clone(),
                &recv_cons_msg,
                cons_msg.id(),
                count,
            )?;

            for transaction in transactions {
                handle_transaction(state.clone(), transport.clone(), &transaction)?;
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
pub fn fetch_random_transactions<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    count: u32,
) -> Result<BTreeSet<Transaction>> {
    let nodes = state.lock().unwrap().sample_nodes()?;
    let mut res = BTreeSet::new();

    for node in nodes {
        let cons_msg = ConsensusMessage::new_fetch_random_transactions(
            &*state.lock().unwrap().address,
            &node,
            count,
        )?;
        send_message(state.clone(), transport.clone(), &cons_msg)?;
        let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
            if recv_cons_msg.is_push_transactions()?
                && recv_cons_msg.node().address == state.lock().unwrap().address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let transactions = handle_push_random_transactions(
                    state.clone(),
                    transport.clone(),
                    &recv_cons_msg,
                    cons_msg.id(),
                    count,
                )?;

                for transaction in transactions {
                    handle_transaction(state.clone(), transport.clone(), &transaction)?;
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
pub fn push_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    fetch_id: u64,
    nodes: &BTreeSet<Node>,
) -> Result<()> {
    let node = Node::new(state.lock().unwrap().stage, address);
    let cons_msg = ConsensusMessage::new_push_nodes(
        &*state.lock().unwrap().address,
        fetch_id + 1,
        &node,
        nodes,
    )?;
    send_message(state, transport, &cons_msg)
}

/// `handle_fetch_nodes` handles a `FetchNodes` request.
pub fn handle_fetch_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::FetchNodes {
            address,
            id,
            node,
            ids,
            ..
        } => {
            if node.address != state.lock().unwrap().address {
                let err = Error::InvalidAddress;
                return Err(err);
            }

            let node = Node::new(state.lock().unwrap().stage, &address);
            handle_node(state.clone(), &node)?;

            let mut nodes = BTreeSet::new();

            for id in ids {
                if Node::lookup(
                    &*state.lock().unwrap().store.lock().unwrap(),
                    state.lock().unwrap().stage,
                    &id,
                )? {
                    let node = Node::get(
                        &*state.lock().unwrap().store.lock().unwrap(),
                        state.lock().unwrap().stage,
                        &id,
                    )?;
                    nodes.insert(node);
                }
            }

            let cons_msg = ConsensusMessage::new_push_nodes(
                &*state.lock().unwrap().address,
                id + 1,
                &node,
                &nodes,
            )?;
            send_message(state, transport, &cons_msg)
        }
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `handle_fetch_random_nodes` handles a `FetchRandomNodes` request.
pub fn handle_fetch_random_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::FetchRandomNodes {
            address,
            id,
            node,
            count,
            ..
        } => {
            if node.address != state.lock().unwrap().address {
                let err = Error::InvalidAddress;
                return Err(err);
            }

            let node = Node::new(state.lock().unwrap().stage, &address);
            handle_node(state.clone(), &node)?;

            let nodes = Node::sample(
                &*state.lock().unwrap().store.lock().unwrap(),
                state.lock().unwrap().stage,
                None,
                None,
                count,
            )?;

            let cons_msg = ConsensusMessage::new_push_nodes(
                &*state.lock().unwrap().address,
                id + 1,
                &node,
                &nodes,
            )?;
            send_message(state, transport, &cons_msg)
        }
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `handle_push_nodes` handles a `PushNodes`.
pub fn handle_push_nodes<S: Store, P: Store>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    msg: &ConsensusMessage,
    fetch_id: u64,
    ids: &BTreeSet<Digest>,
) -> Result<BTreeSet<Node>> {
    msg.validate()?;
    let expected_ids = ids;

    if msg.is_push_nodes()?
        && msg.node().address == state.lock().unwrap().address
        && msg.id() == fetch_id + 1
    {
        match msg.to_owned() {
            ConsensusMessage::PushNodes { ids, nodes, .. } => {
                if !ids.is_subset(&expected_ids) {
                    let err = Error::InvalidMessage;
                    return Err(err);
                }

                for node in &nodes {
                    handle_node(state.clone(), &node)?;
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
pub fn handle_push_random_nodes<S: Store, P: Store>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    msg: &ConsensusMessage,
    fetch_id: u64,
    count: u32,
) -> Result<BTreeSet<Node>> {
    msg.validate()?;
    let expected_count = count;

    if msg.is_push_nodes()?
        && msg.node().address == state.lock().unwrap().address
        && msg.id() == fetch_id + 1
    {
        match msg.to_owned() {
            ConsensusMessage::PushNodes { count, nodes, .. } => {
                if count > expected_count {
                    let err = Error::InvalidMessage;
                    return Err(err);
                }

                for node in &nodes {
                    handle_node(state.clone(), &node)?;
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
pub fn fetch_node_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    ids: &BTreeSet<Digest>,
) -> Result<BTreeSet<Node>> {
    let node = Node::new(state.lock().unwrap().stage, address);
    let cons_msg = ConsensusMessage::new_fetch_nodes(&*state.lock().unwrap().address, &node, ids)?;
    send_message(state.clone(), transport.clone(), &cons_msg)?;

    let mut res = BTreeSet::new();
    let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

    while max_retries > 0 {
        let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
        if recv_cons_msg.is_push_nodes()?
            && recv_cons_msg.node().address == state.lock().unwrap().address
            && recv_cons_msg.id() == cons_msg.id() + 1
        {
            let nodes = handle_push_nodes(state.clone(), &recv_cons_msg, cons_msg.id(), ids)?;

            for node in nodes {
                handle_node(state.clone(), &node)?;
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
pub fn fetch_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    ids: &BTreeSet<Digest>,
) -> Result<BTreeSet<Node>> {
    let nodes = state.lock().unwrap().sample_nodes()?;
    let mut res = BTreeSet::new();

    for node in nodes {
        let cons_msg =
            ConsensusMessage::new_fetch_nodes(&*state.lock().unwrap().address, &node, ids)?;
        send_message(state.clone(), transport.clone(), &cons_msg)?;

        let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
            if recv_cons_msg.is_push_nodes()?
                && recv_cons_msg.node().address == state.lock().unwrap().address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let nodes = handle_push_nodes(state.clone(), &recv_cons_msg, cons_msg.id(), ids)?;

                for node in nodes {
                    handle_node(state.clone(), &node)?;
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
pub fn fetch_node_random_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    count: u32,
) -> Result<BTreeSet<Node>> {
    let node = Node::new(state.lock().unwrap().stage, &address);
    let cons_msg =
        ConsensusMessage::new_fetch_random_nodes(&*state.lock().unwrap().address, &node, count)?;
    send_message(state.clone(), transport.clone(), &cons_msg)?;

    let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);
    let mut res = BTreeSet::new();

    while max_retries > 0 {
        let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
        if recv_cons_msg.is_push_nodes()?
            && recv_cons_msg.node().address == state.lock().unwrap().address
            && recv_cons_msg.id() == cons_msg.id() + 1
        {
            let nodes =
                handle_push_random_nodes(state.clone(), &recv_cons_msg, cons_msg.id(), count)?;

            for node in nodes {
                handle_node(state.clone(), &node)?;
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
pub fn fetch_random_nodes<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    count: u32,
) -> Result<BTreeSet<Node>> {
    let nodes = state.lock().unwrap().sample_nodes()?;
    let mut res = BTreeSet::new();

    for node in nodes {
        let cons_msg = ConsensusMessage::new_fetch_random_nodes(
            &*state.lock().unwrap().address,
            &node,
            count,
        )?;

        send_message(state.clone(), transport.clone(), &cons_msg)?;

        let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

        while max_retries > 0 {
            let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
            if recv_cons_msg.is_push_nodes()?
                && recv_cons_msg.node().address == state.lock().unwrap().address
                && recv_cons_msg.id() == cons_msg.id() + 1
            {
                let nodes =
                    handle_push_random_nodes(state.clone(), &recv_cons_msg, cons_msg.id(), count)?;

                for node in nodes {
                    handle_node(state.clone(), &node)?;
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

/// `fetch_missing_ancestors` fetches a `Transaction` ancestors from remote if missing.
pub fn fetch_missing_ancestors<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    transaction: &Transaction,
) -> Result<BTreeSet<Transaction>> {
    transaction.validate()?;

    let to_fetch: BTreeSet<Digest> = transaction
        .ancestors()?
        .iter()
        .filter(|id| !state.lock().unwrap().state.lookup_known_transaction(&id))
        .copied()
        .collect();

    if to_fetch.is_empty() {
        return Ok(BTreeSet::new());
    }

    let nodes = state.lock().unwrap().sample_nodes()?;
    let mut res = BTreeSet::new();

    for node in &nodes {
        let result =
            fetch_node_transactions(state.clone(), transport.clone(), &node.address, &to_fetch);

        let txs = if let Ok(txs) = result {
            txs
        } else {
            let mut node = state.lock().unwrap().random_node()?;
            while node.address == state.lock().unwrap().address || nodes.contains(&node) {
                node = state.lock().unwrap().random_node()?;
            }

            fetch_node_transactions(state.clone(), transport.clone(), &node.address, &to_fetch)?
        };

        for tx in txs {
            res.insert(tx);
        }
    }

    Ok(res)
}

/// `mine` mines a set of `Transaction`s.
pub fn mine<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    transactions: &BTreeSet<Transaction>,
) -> Result<()> {
    for transaction in transactions {
        transaction.validate()?;

        if transaction.is_mined() {
            let err = Error::AlreadyMined;
            return Err(err);
        }
    }

    let node = Node::new(state.lock().unwrap().stage, address);
    let cons_msg =
        ConsensusMessage::new_mine(&*state.lock().unwrap().address, &node, transactions)?;
    send_message(state, transport, &cons_msg)
}

/// `handle_mine` handles a `Mine` `ConsensusMessage` request.
pub fn handle_mine<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::Mine {
            id,
            address,
            node,
            transactions,
            ..
        } => {
            if node.address != state.lock().unwrap().address {
                let err = Error::InvalidAddress;
                return Err(err);
            }

            let node = Node::new(state.lock().unwrap().stage, &address);
            handle_node(state.clone(), &node)?;

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
                handle_transaction(state.clone(), transport.clone(), &transaction)?;
            }

            let cons_msg = ConsensusMessage::new_push_transactions(
                &*state.lock().unwrap().address,
                id + 1,
                &node,
                &mined,
            )?;
            send_message(state, transport, &cons_msg)
        }
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `serve_mining` serves the mining operations.
pub fn serve_mining<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
) -> Result<()> {
    let timeout = state.lock().unwrap().config.timeout;

    transport
        .lock()
        .unwrap()
        .serve(timeout, |msg| {
            let cons_msg = msg.to_consensus_message()?;

            handle_mine(state.clone(), transport.clone(), &cons_msg).map_err(|e| {
                NetworkError::Consensus {
                    msg: format!("{}", e),
                }
            })
        })
        .map_err(|e| e.into())
}

/// `update_ancestors` updates the ancestors set of a `Transaction`.
pub fn update_ancestors<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    transaction: &Transaction,
) -> Result<()> {
    for ancestor in fetch_missing_ancestors(state.clone(), transport.clone(), transaction)? {
        handle_transaction(state.clone(), transport.clone(), &ancestor)?;
    }

    Ok(())
}

/// `handle_transaction` elaborates an incoming `Node`.
/// It is equivalent to the `OnReceiveTx` function in the Avalanche paper.
pub fn handle_transaction<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    transaction: &Transaction,
) -> Result<()> {
    transaction.validate()?;
    transaction.validate_mined()?;

    let tx_id = transaction.id;

    // NB: state may have been cleared, so the first places to check are the stores

    if !Transaction::lookup(
        &*state.lock().unwrap().pool.lock().unwrap(),
        state.lock().unwrap().stage,
        &tx_id,
    )? && !Transaction::lookup(
        &*state.lock().unwrap().store.lock().unwrap(),
        state.lock().unwrap().stage,
        &tx_id,
    )? {
        Transaction::create(
            &mut *state.lock().unwrap().pool.lock().unwrap(),
            state.lock().unwrap().stage,
            &tx_id,
            &transaction,
        )?;
        state.lock().unwrap().state.add_known_transaction(tx_id);

        state.lock().unwrap().upsert_conflict_sets(&transaction)?;

        state
            .lock()
            .unwrap()
            .state
            .set_transaction_chit(tx_id, false)?;
        state
            .lock()
            .unwrap()
            .state
            .set_transaction_confidence(tx_id, 0)?;

        update_ancestors(state.clone(), transport.clone(), transaction)?;
        state.lock().unwrap().update_successors(transaction)?;
    }

    Ok(())
}

/// `handle_reply` handles a `Reply` request.
pub fn handle_reply<S: Store, P: Store>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
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

            if node.address != state.lock().unwrap().address {
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
pub fn query_node<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    address: &[u8],
    transaction: &Transaction,
) -> Result<bool> {
    let node = Node::new(state.lock().unwrap().stage, address);
    let cons_msg =
        ConsensusMessage::new_query(&*state.lock().unwrap().address, &node, transaction)?;
    send_message(state.clone(), transport.clone(), &cons_msg)?;

    let mut res = false;
    let mut max_retries = state.lock().unwrap().config.max_retries.unwrap_or(1);

    while max_retries > 0 {
        let recv_cons_msg = recv_message(state.clone(), transport.clone())?;
        if recv_cons_msg.is_reply()?
            && recv_cons_msg.node().address == state.lock().unwrap().address
            && recv_cons_msg.id() == cons_msg.id() + 1
        {
            res = handle_reply(
                state.clone(),
                &recv_cons_msg,
                cons_msg.id(),
                &transaction.id,
            )?;
            break;
        } else {
            max_retries -= 1;
        }
    }

    Ok(res)
}

/// `query` queries remote nodes.
pub fn query<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    transaction: &Transaction,
) -> Result<u32> {
    let nodes = state.lock().unwrap().sample_nodes()?;
    let mut res = 0u32;

    for node in nodes {
        let chit = query_node(state.clone(), transport.clone(), &node.address, transaction)? as u32;
        res += chit;
    }

    Ok(res)
}

/// `reply` replies to a `Query` request.
/// In the Avalanche paper the function is called "OnQuery".
pub fn reply<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::Query {
            address,
            id,
            node,
            transaction,
            ..
        } => {
            if node.address != state.lock().unwrap().address {
                let err = Error::InvalidAddress;
                return Err(err);
            }

            let chit = state
                .lock()
                .unwrap()
                .is_strongly_preferred(&transaction.id)?;
            let node = Node::new(state.lock().unwrap().stage, &address);
            handle_node(state.clone(), &node)?;

            let cons_msg = ConsensusMessage::new_reply(
                &*state.lock().unwrap().address,
                id,
                &node,
                transaction.id,
                chit,
            )?;
            send_message(state, transport, &cons_msg)
        }
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `handle` handles incoming `ConsensusMessage`s.
pub fn handle<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
    msg: &ConsensusMessage,
) -> Result<()> {
    msg.validate()?;

    match msg.to_owned() {
        ConsensusMessage::FetchNodes { .. } => {
            handle_fetch_nodes(state.clone(), transport.clone(), msg)
        }
        ConsensusMessage::FetchRandomNodes { .. } => {
            handle_fetch_random_nodes(state.clone(), transport.clone(), msg)
        }
        ConsensusMessage::FetchTransactions { .. } => {
            handle_fetch_transactions(state.clone(), transport.clone(), msg)
        }
        ConsensusMessage::FetchRandomTransactions { .. } => {
            handle_fetch_random_transactions(state.clone(), transport.clone(), msg)
        }
        ConsensusMessage::Query { .. } => reply(state.clone(), transport.clone(), msg),
        _ => {
            let err = Error::InvalidMessage;
            Err(err)
        }
    }
}

/// `serve_incoming` serves incoming `ConsensusMessage`s.
pub fn serve_incoming<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
) -> Result<()> {
    let timeout = state.lock().unwrap().config.timeout;

    transport
        .lock()
        .unwrap()
        .serve(timeout, |msg| {
            let cons_msg = msg.to_consensus_message()?;

            handle(state.clone(), transport.clone(), &cons_msg).map_err(|e| {
                NetworkError::Consensus {
                    msg: format!("{}", e),
                }
            })
        })
        .map_err(|e| e.into())
}

/// `avalanche_step` is a single execution of the main Avalanche Consensus procedure.
pub fn avalanche_step<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
) -> Result<()> {
    let tx_ids: BTreeSet<Digest> = state
        .lock()
        .unwrap()
        .state
        .known_transactions
        .iter()
        .filter(|id| !state.lock().unwrap().state.lookup_queried_transaction(&id))
        .copied()
        .collect();

    for tx_id in tx_ids {
        let tx = match Transaction::get(
            &*state.lock().unwrap().pool.lock().unwrap(),
            state.lock().unwrap().stage,
            &tx_id,
        ) {
            Ok(tx) => {
                tx.validate()?;
                Ok(tx)
            }
            Err(ModelsError::NotFound) => {
                let tx = Transaction::get(
                    &*state.lock().unwrap().store.lock().unwrap(),
                    state.lock().unwrap().stage,
                    &tx_id,
                )?;
                tx.validate()?;
                Ok(tx)
            }
            Err(err) => Err(err),
        }?;

        let missing_txs = fetch_missing_ancestors(state.clone(), transport.clone(), &tx)?;

        for missing_tx in missing_txs.iter() {
            handle_transaction(state.clone(), transport.clone(), &missing_tx)?;
        }

        let chit_sum = query(state.clone(), transport.clone(), &tx)?;

        let mut config = state.lock().unwrap().config.clone();
        config.populate();

        if chit_sum >= config.alpha.unwrap() {
            state
                .lock()
                .unwrap()
                .state
                .set_transaction_chit(tx_id, true)?;

            let mut cs = if let Some(cs_id) = state
                .lock()
                .unwrap()
                .state
                .get_transaction_conflict_set(&tx_id)
            {
                ConflictSet::get(
                    &*state.lock().unwrap().pool.lock().unwrap(),
                    state.lock().unwrap().stage,
                    &cs_id,
                )
            } else {
                let err = ModelsError::NotFound;
                Err(err)
            }?;

            cs.validate()?;

            state.lock().unwrap().update_confidence(&tx_id)?;

            if cs.preferred.is_none() || cs.last.is_none() {
                let err = Error::NotFound;
                return Err(err);
            }

            let pref_id = cs.preferred.unwrap();
            let last_id = cs.last.unwrap();

            let pref_confidence = state
                .lock()
                .unwrap()
                .state
                .get_transaction_confidence(&pref_id)
                .unwrap_or(0);

            let confidence = state
                .lock()
                .unwrap()
                .state
                .get_transaction_confidence(&tx_id)
                .unwrap_or(0);

            if confidence > pref_confidence {
                cs.preferred = Some(tx_id);
            }

            if tx_id != last_id {
                cs.last = Some(tx_id);
                cs.count = 1;
            } else {
                cs.count += 1;
            }

            ConflictSet::update(
                &mut *state.lock().unwrap().pool.lock().unwrap(),
                state.lock().unwrap().stage,
                &cs.address,
                &cs,
            )?;

            Transaction::insert(
                &mut *state.lock().unwrap().store.lock().unwrap(),
                state.lock().unwrap().stage,
                &tx_id,
                &tx,
            )?;
        } else {
            let ancestors: BTreeSet<Digest> = tx
                .ancestors()?
                .iter()
                .filter(|id| state.lock().unwrap().state.lookup_known_transaction(&id))
                .copied()
                .collect();

            for tx_id in ancestors {
                if let Some(cs_id) = state
                    .lock()
                    .unwrap()
                    .state
                    .get_transaction_conflict_set(&tx_id)
                {
                    let mut cs = ConflictSet::get(
                        &*state.lock().unwrap().pool.lock().unwrap(),
                        state.lock().unwrap().stage,
                        &cs_id,
                    )?;
                    cs.validate()?;
                    cs.count = 0;

                    ConflictSet::update(
                        &mut *state.lock().unwrap().pool.lock().unwrap(),
                        state.lock().unwrap().stage,
                        &cs_id,
                        &cs,
                    )?;
                } else {
                    let err = Error::NotFound;
                    return Err(err);
                }
            }
        }

        state.lock().unwrap().state.add_queried_transaction(tx.id)?;
    }

    Ok(())
}

/// `serve_avalanche` serves the main loop of the `Protocol`.
/// The name of the function in the Avalanche paper is "AvalancheLoop".
pub fn serve_avalanche<S: Store, P: Store, T: Transport>(
    state: Arc<Mutex<ProtocolState<S, P>>>,
    transport: Arc<Mutex<T>>,
) -> Result<()> {
    let mut res = Ok(());

    while res.is_ok() {
        res = avalanche_step(state.clone(), transport.clone());
    }

    res
}
