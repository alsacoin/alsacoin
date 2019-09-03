//! # Channel Node
//!
//! `channel_node` contains the mpsc Channel node types and functions. In this case
//! the network of nodes is local and every node occupies a different thread in the same process.

use crate::error::Error;
use crate::message::Message;
use crate::result::Result;
use crate::traits::Transport;
use crypto::hash::{Blake512Hasher, Digest};
use crypto::random::Random;
use std::collections::BTreeMap;
use std::ops::FnMut;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// `ChannelNode` is a node using a mpsc Channel as transport. It only communicates to local nodes
/// inhabiting other threads.
#[derive(Clone)]
pub struct ChannelNode {
    id: Digest,
    address: Vec<u8>,
    receiver: Arc<Mutex<Receiver<Message>>>,
    channels: BTreeMap<Digest, Sender<Message>>,
}

impl ChannelNode {
    /// `ADDRESS_LEN` is the length of a `ChannelNode` address.
    pub const ADDRESS_LEN: u32 = 16;

    /// `new` creates a new `ChannelNode` backend.
    pub fn new() -> Result<ChannelNode> {
        let address = Self::gen_address()?;
        let id = Blake512Hasher::hash(&address);

        let (sender, receiver) = channel();

        let mut channels = BTreeMap::new();
        channels.insert(id, sender.clone());

        let receiver = Arc::new(Mutex::new(receiver));

        let node = ChannelNode {
            id,
            address,
            receiver,
            channels,
        };

        Ok(node)
    }

    /// `gen_address` generates a new `ChannelNode` address.
    pub fn gen_address() -> Result<Vec<u8>> {
        Random::bytes(Self::ADDRESS_LEN as usize).map_err(|e| e.into())
    }

    /// `calc_id` calculates the `ChannelNode` id.
    pub fn calc_id(&self) -> Digest {
        Blake512Hasher::hash(&self.address)
    }

    /// `lookup_channel` look ups a recorded `ChannelNode`.
    pub fn lookup_channel(&self, id: &Digest) -> bool {
        self.channels.contains_key(id)
    }

    /// `add_channel` records a new `ChannelNode`.
    pub fn add_channel(&mut self, id: Digest, channel: &Sender<Message>) -> Result<()> {
        if self.id == id {
            let err = Error::NotAllowed;
            return Err(err);
        }

        if self.lookup_channel(&id) {
            let err = Error::AlreadyFound;
            return Err(err);
        }

        self.channels.insert(id, channel.to_owned());

        Ok(())
    }

    /// `remove_channel` removes a recorded `ChannelNode`.
    pub fn remove_channel(&mut self, id: &Digest) -> Result<()> {
        if id == &self.id {
            let err = Error::NotAllowed;
            return Err(err);
        }

        if self.channels.remove(id).is_none() {
            let err = Error::NotFound;
            Err(err)
        } else {
            Ok(())
        }
    }

    /// `validate` validates the `ChannelNode`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id() {
            let err = Error::InvalidId;
            return Err(err);
        }

        if !self.lookup_channel(&self.id) {
            let err = Error::NotFound;
            return Err(err);
        }

        Ok(())
    }

    /// `_send` sends binary data to a known `ChannelNode`.
    fn _send(&self, address: &[u8], data: &[u8], _timeout: Option<u64>) -> Result<()> {
        if address.len() != Self::ADDRESS_LEN as usize {
            let err = Error::InvalidLength;
            return Err(err);
        }

        let id = Blake512Hasher::hash(address);

        if let Some(ref sender) = self.channels.get(&id) {
            let msg = Message {
                address: self.address.clone(),
                data: data.to_owned(),
            };

            sender.send(msg).map_err(|e| e.into())
        } else {
            let err = Error::NotFound;
            Err(err)
        }
    }

    /// `_recv` receives a `Message` from a known `ChannelNode`.
    /// No timeouts: https://github.com/rust-lang/rust/issues/39364.
    fn _recv(&mut self, _timeout: Option<u64>) -> Result<Message> {
        self.receiver.lock().unwrap().recv().map_err(|e| e.into())
    }

    /// `_serve` handles incoming `Message`s.
    fn _serve<F>(&mut self, _timeout: Option<u64>, mut handler: F) -> Result<()>
    where
        F: FnMut(Message) -> Result<()>,
    {
        for message in self.receiver.lock().unwrap().try_iter() {
            handler(message)?;
        }

        Ok(())
    }
}

impl Transport for ChannelNode {
    fn local_address(&self) -> Result<Vec<u8>> {
        Ok(self.address.clone())
    }

    fn send(&mut self, address: &[u8], data: &[u8], timeout: Option<u64>) -> Result<()> {
        self._send(address, data, timeout)
    }

    fn recv(&mut self, timeout: Option<u64>) -> Result<Message> {
        self._recv(timeout)
    }

    fn serve<F: FnMut(Message) -> Result<()>>(
        &mut self,
        timeout: Option<u64>,
        handler: F,
    ) -> Result<()> {
        self._serve(timeout, handler)
    }
}

#[test]
fn test_channel_node_ops() {
    use crypto::random::Random;

    let res = ChannelNode::new();
    assert!(res.is_ok());

    let mut trsp_a = res.unwrap();

    let res = trsp_a.validate();
    assert!(res.is_ok());

    let trsp_a_id = trsp_a.id;
    let trsp_a_addr = trsp_a.address.clone();
    let trsp_a_channel = trsp_a.channels.get(&trsp_a_id).unwrap().clone();

    let found = trsp_a.lookup_channel(&trsp_a_id);
    assert!(found);

    let res = trsp_a.add_channel(trsp_a_id, &trsp_a_channel);
    assert!(res.is_err());

    let res = trsp_a.remove_channel(&trsp_a_id);
    assert!(res.is_err());

    let data_len = 1000;
    let data = Random::bytes(data_len).unwrap();

    let res = trsp_a.send(&trsp_a_addr, &data, None);
    assert!(res.is_ok());

    let res = trsp_a.recv(None);
    assert!(res.is_ok());

    let msg = res.unwrap();
    assert_eq!(msg.address, trsp_a_addr);
    assert_eq!(msg.data, data);

    let mut trsp_b = ChannelNode::new().unwrap();
    let trsp_b_id = trsp_b.id;
    let trsp_b_addr = trsp_b.address.clone();
    let trsp_b_channel = trsp_b.channels.get(&trsp_b_id).unwrap().clone();

    let found = trsp_b.lookup_channel(&trsp_a_id);
    assert!(!found);

    let found = trsp_a.lookup_channel(&trsp_b_id);
    assert!(!found);

    let res = trsp_b.add_channel(trsp_a_id, &trsp_a_channel);
    assert!(res.is_ok());

    let res = trsp_a.add_channel(trsp_b_id, &trsp_b_channel);
    assert!(res.is_ok());

    let found = trsp_b.lookup_channel(&trsp_a_id);
    assert!(found);

    let found = trsp_a.lookup_channel(&trsp_b_id);
    assert!(found);

    let res = trsp_b.add_channel(trsp_a_id, &trsp_a_channel);
    assert!(res.is_err());

    let res = trsp_a.add_channel(trsp_b_id, &trsp_b_channel);
    assert!(res.is_err());

    let res = trsp_a.send(&trsp_b_addr, &data, None);
    assert!(res.is_ok());

    let res = trsp_b.recv(None);
    assert!(res.is_ok());

    let msg = res.unwrap();
    assert_eq!(msg.address, trsp_a_addr);
    assert_eq!(msg.data, data);

    let res = trsp_b.send(&trsp_a_addr, &data, None);
    assert!(res.is_ok());

    let res = trsp_a.recv(None);
    assert!(res.is_ok());

    let msg = res.unwrap();
    assert_eq!(msg.address, trsp_b_addr);
    assert_eq!(msg.data, data);

    let res = trsp_b.remove_channel(&trsp_a_id);
    assert!(res.is_ok());

    let res = trsp_a.remove_channel(&trsp_b_id);
    assert!(res.is_ok());

    let found = trsp_b.lookup_channel(&trsp_a_id);
    assert!(!found);

    let found = trsp_a.lookup_channel(&trsp_b_id);
    assert!(!found);

    let res = trsp_b.send(&trsp_a_addr, &data, None);
    assert!(res.is_err());

    let res = trsp_b.send(&trsp_a_addr, &data, None);
    assert!(res.is_err());
}
