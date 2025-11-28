use bytes::{Buf, BufMut, Bytes};

use crate::protocol::{
    packet::{Packet, RaknetEncodable},
    types::{Advertisement, Magic, RaknetTime},
};

pub struct UnconnectedPing {
    pub ping_time: RaknetTime,
    pub magic: Magic,
}

impl Packet for UnconnectedPing {
    const ID: u8 = 0x01;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.ping_time.encode_raknet(dst);

        self.magic.encode_raknet(dst);
    }

    fn decode_body(src: &mut impl Buf) -> Result<Self, super::DecodeError> {
        Ok(Self {
            ping_time: RaknetTime::decode_raknet(src)?,
            magic: Magic::decode_raknet(src)?,
        })
    }
}

pub struct UnconnectedPong {
    pub ping_time: RaknetTime,
    pub server_guid: u64,
    pub magic: Magic,
    pub advertisement: Advertisement,
}

impl Packet for UnconnectedPong {
    const ID: u8 = 0x1c;

    fn encode_body(&self, dst: &mut impl BufMut) {
        self.ping_time.encode_raknet(dst);
        self.server_guid.encode_raknet(dst);
        self.magic.encode_raknet(dst);
        self.advertisement.encode_raknet(dst);
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

pub struct UnconnectedPingOpenConnections {
    pub payload: Bytes,
}

impl Packet for UnconnectedPingOpenConnections {
    const ID: u8 = 0x02;

    fn encode_body(&self, dst: &mut impl BufMut) {
        dst.put_slice(&self.payload);
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
