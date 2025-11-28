use std::net::SocketAddr;

use bytes::{BufMut, Bytes};

use crate::protocol::{
    constants,
    packet::{Packet, RaknetEncodable},
    types::{EoBPadding, Magic, RaknetTime},
};

pub struct OpenConnectionRequest1 {
    pub magic: Magic,
    pub protocol_version: u8,
    pub padding: EoBPadding,
}

impl Packet for OpenConnectionRequest1 {
    const ID: u8 = 0x05;

    fn encode_body(&self, dst: &mut impl bytes::BufMut) {
        self.magic.encode_raknet(dst);
        self.protocol_version.encode_raknet(dst);
        self.padding.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            magic: Magic::decode_raknet(src)?,
            protocol_version: u8::decode_raknet(src)?,
            padding: EoBPadding::decode_raknet(src)?,
        })
    }
}

pub struct OpenConnectionReply1 {
    pub magic: Magic,
    pub server_guide: u64,
    pub cookie: Option<u32>,
    pub mtu: u16,
}

impl Packet for OpenConnectionReply1 {
    const ID: u8 = 0x06;

    fn encode_body(&self, dst: &mut impl bytes::BufMut) {
        self.magic.encode_raknet(dst);
        self.server_guide.encode_raknet(dst);
        (self.cookie.is_some() as u8 != 0).encode_raknet(dst); // security bool
        if self.cookie.is_some() {
            self.cookie.unwrap().encode_raknet(dst);
        }
        self.mtu.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            magic: Magic::decode_raknet(src)?,
            server_guide: u64::decode_raknet(src)?,
            cookie: if u8::decode_raknet(src)? != 0 {
                Some(u32::decode_raknet(src)?)
            } else {
                None
            },
            mtu: u16::decode_raknet(src)?,
        })
    }
}

pub struct OpenConnectionRequest2 {
    pub magic: Magic,
    pub cookie: Option<u32>,
    pub client_proof: bool,
    pub server_addr: SocketAddr,
    pub mtu: u16,
    pub client_guid: u64,
}

impl Packet for OpenConnectionRequest2 {
    const ID: u8 = 0x07;

    fn encode_body(&self, dst: &mut impl bytes::BufMut) {
        self.magic.encode_raknet(dst);
        (self.cookie.is_some() as u8 != 0).encode_raknet(dst); // security bool
        if self.cookie.is_some() {
            self.cookie.unwrap().encode_raknet(dst);
            self.client_proof.encode_raknet(dst);
        }
        self.server_addr.encode_raknet(dst);
        self.mtu.encode_raknet(dst);
        self.client_guid.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        let magic = Magic::decode_raknet(src)?;

        // Grab the remaining bytes into a temp buffer.
        // Safely grab the remaining bytes without moving `src`.
        let remaining = src.remaining();
        let rest: Bytes = src.copy_to_bytes(remaining); // advances `src` by `remaining`

        // First attempt: cookie + proof + addr + mtu + guid.
        if remaining >= 5 {
            let mut tmp = rest.clone();
            let attempt = (|| -> Result<OpenConnectionRequest2, super::DecodeError> {
                let cookie = u32::decode_raknet(&mut tmp)?;
                let client_proof = bool::decode_raknet(&mut tmp)?;
                let server_addr = SocketAddr::decode_raknet(&mut tmp)?;
                let mtu = u16::decode_raknet(&mut tmp)?;
                let client_guid = u64::decode_raknet(&mut tmp)?;
                Ok(OpenConnectionRequest2 {
                    magic,
                    cookie: Some(cookie),
                    client_proof,
                    server_addr,
                    mtu,
                    client_guid,
                })
            })();
            if attempt.is_ok() {
                return attempt;
            }
        }

        // Fallback: addr + mtu + guid, no cookie/proof.
        let mut tmp = rest.clone();
        let server_addr = SocketAddr::decode_raknet(&mut tmp)?;
        let mtu = u16::decode_raknet(&mut tmp)?;
        let client_guid = u64::decode_raknet(&mut tmp)?;

        Ok(OpenConnectionRequest2 {
            magic,
            cookie: None,
            client_proof: false,
            server_addr,
            mtu,
            client_guid,
        })
    }
}

pub struct OpenConnectionReply2 {
    pub magic: Magic,
    pub server_guid: u64,
    pub server_addr: SocketAddr,
    pub mtu: u16,
    pub security: bool,
}

impl Packet for OpenConnectionReply2 {
    const ID: u8 = 0x08;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.magic.encode_raknet(dst);
        self.server_guid.encode_raknet(dst);
        self.server_addr.encode_raknet(dst);
        self.mtu.encode_raknet(dst);
        self.security.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            magic: Magic::decode_raknet(src)?,
            server_guid: u64::decode_raknet(src)?,
            server_addr: SocketAddr::decode_raknet(src)?,
            mtu: u16::decode_raknet(src)?,
            security: bool::decode_raknet(src)?,
        })
    }
}

pub struct IncompatibleProtocolVersion {
    pub protocol: u8,
    pub magic: Magic,
    pub server_guid: u64,
}

impl Packet for IncompatibleProtocolVersion {
    const ID: u8 = 0x19;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.protocol.encode_raknet(dst);
        self.magic.encode_raknet(dst);
        self.server_guid.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            protocol: u8::decode_raknet(src)?,
            magic: Magic::decode_raknet(src)?,
            server_guid: u64::decode_raknet(src)?,
        })
    }
}

pub struct AlreadyConnected {
    pub magic: Magic,
    pub server_guid: u64,
}

impl Packet for AlreadyConnected {
    const ID: u8 = 0x12;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.magic.encode_raknet(dst);
        self.server_guid.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            magic: Magic::decode_raknet(src)?,
            server_guid: u64::decode_raknet(src)?,
        })
    }
}

pub struct ConnectionRequest {
    pub server_guid: u64,
    pub timestamp: RaknetTime,
    pub secure: bool,
}

impl Packet for ConnectionRequest {
    const ID: u8 = 0x09;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.server_guid.encode_raknet(dst);
        self.timestamp.encode_raknet(dst);
        self.secure.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            server_guid: u64::decode_raknet(src)?,
            timestamp: RaknetTime::decode_raknet(src)?,
            secure: bool::decode_raknet(src)?,
        })
    }
}

pub struct ConnectionRequestAccepted {
    pub address: SocketAddr,
    pub system_index: u16,
    pub system_addresses: [SocketAddr; 10],
    pub request_timestamp: RaknetTime,
    pub accepted_timestamp: RaknetTime,
}

impl Packet for ConnectionRequestAccepted {
    const ID: u8 = 0x10;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.address.encode_raknet(dst);
        self.system_index.encode_raknet(dst);

        for address in &self.system_addresses {
            address.encode_raknet(dst);
        }

        self.request_timestamp.encode_raknet(dst);
        self.accepted_timestamp.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        let address = SocketAddr::decode_raknet(src)?;
        let system_index = u16::decode_raknet(src)?;

        let mut system_addresses: [SocketAddr; 10] = [SocketAddr::V4(constants::ANY_V4); 10];

        for addr in &mut system_addresses {
            *addr = SocketAddr::decode_raknet(src)?
        }

        let request_timestamp = RaknetTime::decode_raknet(src)?;
        let accepted_timestamp = RaknetTime::decode_raknet(src)?;

        Ok(Self {
            address,
            system_index,
            system_addresses,
            request_timestamp,
            accepted_timestamp,
        })
    }
}

pub struct ConnectionRequestFailed {
    pub magic: Magic,
    pub server_guid: u64,
}

impl Packet for ConnectionRequestFailed {
    const ID: u8 = 0x11;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.magic.encode_raknet(dst);
        self.server_guid.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            magic: Magic::decode_raknet(src)?,
            server_guid: u64::decode_raknet(src)?,
        })
    }
}

pub struct NewIncomingConnection {
    pub server_address: SocketAddr,
    pub system_addresses: [SocketAddr; 10],
    pub request_timestamp: RaknetTime,
    pub accepted_timestamp: RaknetTime,
}

impl Packet for NewIncomingConnection {
    const ID: u8 = 0x13;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.server_address.encode_raknet(dst);
        for address in &self.system_addresses {
            address.encode_raknet(dst);
        }
        self.request_timestamp.encode_raknet(dst);
        self.accepted_timestamp.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl bytes::Buf) -> Result<Self, super::DecodeError> {
        let server_address = SocketAddr::decode_raknet(src)?;

        let mut system_addresses: [SocketAddr; 10] = [SocketAddr::V4(constants::ANY_V4); 10];

        for addr in &mut system_addresses {
            *addr = SocketAddr::decode_raknet(src)?
        }

        let request_timestamp = RaknetTime::decode_raknet(src)?;
        let accepted_timestamp = RaknetTime::decode_raknet(src)?;
        Ok(Self {
            server_address,
            system_addresses,
            request_timestamp,
            accepted_timestamp,
        })
    }
}
