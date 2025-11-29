use bytes::{Buf, BufMut};

use crate::{
    protocol::{
        packet::{DecodeError, RaknetEncodable},
        types::DatagramHeader,
    },
    transport::encapsulated_packet::EncapsulatedPacket,
};

pub struct Datagram {
    pub header: DatagramHeader,
    pub packets: Vec<EncapsulatedPacket>,
}

impl Datagram {
    pub fn encode(&self, dst: &mut impl BufMut) {
        self.header.encode(dst);
        for pkt in &self.packets {
            pkt.encode_raknet(dst);
        }
    }

    pub fn decode(src: &mut impl Buf) -> Result<Self, DecodeError> {
        let header = DatagramHeader::decode(src)?;
        let mut packets = Vec::new();
        while src.has_remaining() {
            packets.push(EncapsulatedPacket::decode_raknet(src)?);
        }
        Ok(Self { header, packets })
    }
}
