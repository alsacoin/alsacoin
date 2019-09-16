//! # Tcp Node
//!
//! `tcp_transport` contains the Tcp transport backend types and functions.

use crate::error::Error;
use crate::message::Message;
use crate::result::Result;
use crate::traits::Network;
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crypto::hash::{Blake512Hasher, Digest};
use std::io::{Cursor, Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4};
use std::net::{TcpListener, TcpStream};
use std::ops::FnMut;
use std::time::Duration;

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

/// `TcpNode` is a network node using a Tcp transport.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

        let addr_buf = address_to_bytes(&address)?;

        let id = Blake512Hasher::hash(&addr_buf);

        let address = SocketAddrV4::new(ip_addr, Self::DEFAULT_PORT);

        let node = TcpNode { id, address };

        Ok(node)
    }

    /// `local` buids a local `TcpNode`.
    pub fn local() -> Result<TcpNode> {
        TcpNode::new("127.0.0.1")
    }

    /// `from_parts` creates a new `TcpNode` with an ip octet and a port.
    pub fn from_parts(ip: [u8; 4], port: u16) -> Result<TcpNode> {
        let ip_addr = Ipv4Addr::from(ip);
        let address = SocketAddrV4::new(ip_addr, port);

        let addr_buf = address_to_bytes(&address)?;

        let id = Blake512Hasher::hash(&addr_buf);

        let node = TcpNode { id, address };

        Ok(node)
    }

    /// `address_bytes` converts the `TcpNode` address to a vector of bytes.
    pub fn address_bytes(&self) -> Result<Vec<u8>> {
        address_to_bytes(&self.address)
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
    fn _send(&self, address: &[u8], data: &[u8], timeout: Option<u64>) -> Result<()> {
        let socketaddr = address_from_bytes(address)?;
        let mut stream = TcpStream::connect(&socketaddr)?;

        let timeout = timeout.map(Duration::from_secs);

        stream.set_write_timeout(timeout)?;

        stream.write_all(data)?;

        Ok(())
    }

    /// `_recv` receives a `Message` from a known `TcpNode`.
    fn _recv(&mut self, timeout: Option<u64>) -> Result<Message> {
        let listener = TcpListener::bind(&self.address)?;
        let (mut stream, _) = listener.accept()?;

        let mut buf = Vec::new();

        let timeout = timeout.map(Duration::from_secs);

        stream.set_read_timeout(timeout)?;

        stream.read_to_end(&mut buf)?;

        Message::from_bytes(&buf)
    }

    /// `_serve` handles incoming `Message`s.
    fn _serve<F>(&mut self, timeout: Option<u64>, mut handler: F) -> Result<()>
    where
        F: FnMut(Message) -> Result<()>,
    {
        let listener = TcpListener::bind(&self.address)?;

        for stream in listener.incoming() {
            let mut stream = stream?;

            let mut buf = Vec::new();

            let timeout = timeout.map(Duration::from_secs);

            stream.set_read_timeout(timeout)?;

            stream.read_to_end(&mut buf)?;

            let msg = Message::from_bytes(&buf)?;

            handler(msg)?;
        }

        Ok(())
    }
}

impl Network for TcpNode {
    fn local_address(&self) -> Result<Vec<u8>> {
        self.address_bytes()
    }

    fn send(&mut self, address: &[u8], data: &[u8], timeout: Option<u64>) -> Result<()> {
        self._send(address, data, timeout)
    }

    fn recv(&mut self, timeout: Option<u64>) -> Result<Message> {
        self._recv(timeout)
    }

    fn serve(
        &mut self,
        timeout: Option<u64>,
        handler: Box<dyn FnMut(Message) -> Result<()>>,
    ) -> Result<()> {
        self._serve(timeout, handler)
    }
}

#[test]
fn test_tcp_node_ops() {
    use crypto::random::Random;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    let res = TcpNode::local();
    assert!(res.is_ok());

    let mut trsp_a = res.unwrap();

    let res = trsp_a.validate();
    assert!(res.is_ok());

    let data_len = 1000;
    let data = Random::bytes(data_len).unwrap();
    let data_arc = Arc::new(data.clone());
    let trsp_a_addr = trsp_a.address.clone();

    let handler = move |msg: Message| {
        let trsp_a_addr_buf = address_to_bytes(&trsp_a_addr).unwrap();
        assert_eq!(msg.address, trsp_a_addr_buf);
        assert_eq!(msg.data, *data_arc);

        Ok(())
    };

    thread::spawn(move || {
        let _ = trsp_a.serve(None, Box::new(handler));
    });

    thread::sleep(Duration::from_secs(3));
    let trsp_a_addr_buf = address_to_bytes(&trsp_a_addr).unwrap();
    let res = trsp_a.send(&trsp_a_addr_buf, &data, None);
    assert!(res.is_ok());
}
