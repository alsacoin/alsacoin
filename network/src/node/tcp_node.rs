//! # Tcp Node
//!
//! `tcp_transport` contains the Tcp transport backend types and functions.

use crate::error::Error;
use crate::message::Message;
use crate::result::Result;
use crate::traits::Transport;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crypto::hash::{Blake512Hasher, Digest};
use std::net::{TcpListener, TcpStream};
//use std::net::{Incoming, Shutdown};
use std::io::{Cursor, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::ops::FnMut;

/// `TcpNode` is a network node using a Tcp transport.
pub struct TcpNode {
    id: Digest,
    address: SocketAddrV4,
}

impl TcpNode {
    /// `DEFAULT_PORT` is the default port of the `TcpNode`.
    pub const DEFAULT_PORT: u16 = 2019;

    /// `new` creates a new `TcpNode` from an IPv4 address.
    pub fn new(addr: &str) -> Result<TcpNode> {
        let ip_addr: Ipv4Addr = addr.parse()?;
        let address = SocketAddrV4::new(ip_addr, Self::DEFAULT_PORT);

        let addr_buf = Self::address_to_bytes(&address)?;

        let id = Blake512Hasher::hash(&addr_buf);

        let address = SocketAddrV4::new(ip_addr, Self::DEFAULT_PORT);

        let node = TcpNode { id, address };

        Ok(node)
    }

    /// `from_parts` creates a new `TcpNode` with an ip octet and a port.
    pub fn from_parts(ip: [u8; 4], port: u16) -> Result<TcpNode> {
        let ip_addr = Ipv4Addr::from(ip);
        let address = SocketAddrV4::new(ip_addr, port);

        let addr_buf = Self::address_to_bytes(&address)?;

        let id = Blake512Hasher::hash(&addr_buf);

        let node = TcpNode { id, address };

        Ok(node)
    }

    /// `address_to_bytes` converts a SocketAddrV4 to a vector of bytes.
    pub fn address_to_bytes(address: &SocketAddrV4) -> Result<Vec<u8>> {
        let mut buf = Vec::new();

        for n in &address.ip().octets() {
            buf.write_u8(*n)?;
        }

        buf.write_u16::<BigEndian>(address.port())?;

        Ok(buf)
    }

    /// `address_from_bytes` returns an address from a slice of bytes.
    pub fn address_from_bytes(buf: &[u8]) -> Result<SocketAddrV4> {
        let mut reader = Cursor::new(buf);

        let mut ip = [0u8; 4];

        #[allow(clippy::needless_range_loop)]
        for i in 0..4 {
            ip[i] = reader.read_u8()?;
        }

        let port = reader.read_u16::<BigEndian>()?;

        let ip_addr = Ipv4Addr::from(ip);
        let address = SocketAddrV4::new(ip_addr, port);
        Ok(address)
    }

    /// `address_bytes` converts the `TcpNode` address to a vector of bytes.
    pub fn address_bytes(&self) -> Result<Vec<u8>> {
        Self::address_to_bytes(&self.address)
    }

    /// `calc_id` calculates the `TcpNode` id.
    pub fn calc_id(&self) -> Result<Digest> {
        let addr_buf = self.address_bytes()?;
        let id = Blake512Hasher::hash(&addr_buf);
        Ok(id)
    }

    /// `validate` validates the `TcpNode`.
    pub fn validate(&self) -> Result<()> {
        if self.id != self.calc_id()? {
            let err = Error::InvalidId;
            return Err(err);
        }

        Ok(())
    }

    /// `_send` sends binary data to a `TcpNode`.
    fn _send(&self, address: &[u8], data: &[u8]) -> Result<()> {
        let socketaddr = TcpNode::address_from_bytes(address)?;
        let mut stream = TcpStream::connect(&socketaddr)?;

        // TODO: set a write timeout
        stream.write_all(data)?;

        Ok(())
    }

    /// `_recv` receives a `Message` from a known `ChannelNode`.
    fn _recv(&mut self) -> Result<Message> {
        let listener = TcpListener::bind(&self.address)?;
        let (mut stream, _) = listener.accept()?;

        let mut buf = Vec::new();

        // TODO: set a read timeout
        stream.read_to_end(&mut buf)?;

        Message::from_bytes(&buf)
    }

    /// `_serve` handles incoming `Message`s.
    fn _serve<F>(&mut self, _handler: F) -> Result<()>
    where
        F: FnMut(Message) -> Result<()>,
    {
        // TODO
        unreachable!()
    }
}

impl Transport for TcpNode {
    fn local_address(&self) -> Result<Vec<u8>> {
        self.address_bytes()
    }

    fn send(&mut self, address: &[u8], data: &[u8]) -> Result<()> {
        self._send(address, data)
    }

    fn recv(&mut self) -> Result<Message> {
        self._recv()
    }

    fn serve<F: FnMut(Message) -> Result<()>>(&mut self, handler: F) -> Result<()> {
        self._serve(handler)
    }
}
