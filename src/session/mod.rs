use bytes::BytesMut;

use crate::{
    protocol::{
        constants,
        packet::RaknetPacket,
        reliability::Reliability,
        types::{EncapsulatedPacketHeader, Sequence24},
    },
    transport::encapsulated_packet::EncapsulatedPacket,
};

pub struct Session {
    mtu: usize,
}

impl Session {
    fn queue_packet(&self, pkt: RaknetPacket, reliability: Reliability, channel: u8) {
        let mut payload_buf = BytesMut::new();
        pkt.encode(&mut payload_buf);
        let mut payload = payload_buf.freeze();

        let max_len = self.mtu
            - constants::MAXIMUM_ENCAPSULATED_HEADER_SIZE
            - constants::RAKNET_DATAGRAM_HEADER_SIZE;

        let header = EncapsulatedPacketHeader {
            reliability,
            is_split: false,
            needs_bas: true, // Cloudburst sets this
        };
        let split = None;
        let ordering_index = if reliability.is_ordered() {
            let idx = self.order_write[channel as usize];
            self.order_write[channel as usize] = idx + Sequence24::new(1);
            Some(idx)
        } else {
            None
        };

        let packet = EncapsulatedPacket {
            header,
            bit_length: ((payload.len() as u16) << 3),
            reliable_index: if reliability.is_reliable() {
                let idx = self.next_reliable;
                self.next_reliable = self.next_reliable.next();
                Some(idx)
            } else {
                None
            },
            sequence_index: None, // TODO if you implement sequenced
            ordering_index,
            ordering_channel: ordering_index.map(|_| channel),
            split: None,
            payload,
        };
        // push into outgoing queue
    }
}
