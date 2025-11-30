//! Unconnected (offline) RakNet discovery and ping packets.

use bytes::{Buf, BufMut, Bytes};

use crate::protocol::{
    packet::{Packet, RaknetEncodable},
    types::{Advertisement, Magic, RaknetTime},
};

/// Unconnected ping used by clients to discover RakNet servers.
#[derive(Debug, Clone)]
pub struct UnconnectedPing {
    pub ping_time: RaknetTime,
    pub magic: Magic,
}

impl Packet for UnconnectedPing {
    const ID: u8 = 0x01;

    fn encode_body(
        &self,
        dst: &mut impl BufMut,
    ) -> Result<(), crate::protocol::packet::EncodeError> {
        self.ping_time.encode_raknet(dst)?;

        self.magic.encode_raknet(dst)?;
        Ok(())
    }

    fn decode_body(src: &mut impl Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            ping_time: RaknetTime::decode_raknet(src)?,
            magic: Magic::decode_raknet(src)?,
        })
    }
}

/// Unconnected pong sent by servers in response to `UnconnectedPing`.
#[derive(Debug, Clone)]
pub struct UnconnectedPong {
    pub ping_time: RaknetTime,
    pub server_guid: u64,
    pub magic: Magic,
    pub advertisement: Advertisement,
}

impl Packet for UnconnectedPong {
    const ID: u8 = 0x1c;

    fn encode_body(
        &self,
        dst: &mut impl BufMut,
    ) -> Result<(), crate::protocol::packet::EncodeError> {
        self.ping_time.encode_raknet(dst)?;
        self.server_guid.encode_raknet(dst)?;
        self.magic.encode_raknet(dst)?;
        self.advertisement.encode_raknet(dst)?;
        Ok(())
    }

    fn decode_body(src: &mut impl Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            ping_time: RaknetTime::decode_raknet(src)?,
            server_guid: u64::decode_raknet(src)?,
            magic: Magic::decode_raknet(src)?,
            advertisement: Advertisement::decode_raknet(src)?,
        })
    }
}

/// Legacy packet used for querying open connections; currently unimplemented.
#[derive(Debug, Clone)]
pub struct UnconnectedPingOpenConnections {
    pub payload: Bytes,
}

impl Packet for UnconnectedPingOpenConnections {
    const ID: u8 = 0x02;

    fn encode_body(
        &self,
        dst: &mut impl BufMut,
    ) -> Result<(), crate::protocol::packet::EncodeError> {
        dst.put_slice(&self.payload);
        Ok(())
    }

    fn decode_body(src: &mut impl Buf) -> Result<Self, super::DecodeError> {
        let remaining = src.remaining();
        let payload = src.copy_to_bytes(remaining);

        Err(super::DecodeError::UnimplementedPacket {
            id: Self::ID,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn unconnected_ping_roundtrip() {
        let pkt = UnconnectedPing {
            ping_time: RaknetTime(123),
            magic: [0x23; 16],
        };
        let mut buf = BytesMut::new();
        pkt.encode_body(&mut buf).unwrap();
        let mut slice = buf.freeze();
        let decoded = UnconnectedPing::decode_body(&mut slice).unwrap();
        assert_eq!(decoded.ping_time.0, pkt.ping_time.0);
        assert_eq!(decoded.magic, pkt.magic);
    }

    #[test]
    fn unconnected_pong_roundtrip() {
        let pkt = UnconnectedPong {
            ping_time: RaknetTime(1),
            server_guid: 2,
            magic: [0x45; 16],
            advertisement: Advertisement(None),
        };
        let mut buf = BytesMut::new();
        pkt.encode_body(&mut buf).unwrap();
        let mut slice = buf.freeze();
        let decoded = UnconnectedPong::decode_body(&mut slice).unwrap();
        assert_eq!(decoded.ping_time.0, pkt.ping_time.0);
        assert_eq!(decoded.server_guid, pkt.server_guid);
        assert_eq!(decoded.magic, pkt.magic);
    }
}
