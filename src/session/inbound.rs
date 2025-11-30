use std::time::Instant;

use crate::protocol::{
    encapsulated_packet::EncapsulatedPacket,
    packet::{DecodeError, RaknetPacket},
    types::Sequence24,
};
use bytes::Bytes;

use crate::protocol::ack::{AckNackPayload, SequenceRange};

use super::Session;

impl Session {
    /// Handle an incoming data payload (a list of encapsulated packets).
    pub fn handle_data_payload(
        &mut self,
        packets: Vec<EncapsulatedPacket>,
        now: Instant,
    ) -> Result<Vec<RaknetPacket>, DecodeError> {
        let mut out = Vec::new();

        self.sliding.on_packet_received(now);

        for enc in packets.into_iter() {
            self.handle_encapsulated(enc, now, &mut out)?;
        }

        Ok(out)
    }

    /// Handle an incoming dedicated ACK payload.
    pub fn handle_ack_payload(&mut self, payload: AckNackPayload) {
        self.incoming_acks.extend(payload.ranges);
    }

    /// Handle an incoming dedicated NACK payload.
    pub fn handle_nack_payload(&mut self, payload: AckNackPayload) {
        self.incoming_naks.extend(payload.ranges);
    }

    fn handle_encapsulated(
        &mut self,
        enc: EncapsulatedPacket,
        now: Instant,
        out: &mut Vec<RaknetPacket>,
    ) -> Result<(), DecodeError> {
        let assembled_opt = self.split_assembler.add(enc, now)?;
        let enc = match assembled_opt {
            Some(pkt) => pkt,
            None => return Ok(()),
        };

        let rel = enc.header.reliability;

        if rel.is_reliable() {
            let ridx = match enc.reliable_index {
                Some(idx) => idx,
                None => {
                    return Ok(());
                }
            };

            if !self.process_reliable_index(ridx) {
                return Ok(());
            }
        }

        if rel.is_ordered() {
            self.handle_ordered(enc, out)?;
        } else {
            self.decode_and_push(enc, out)?;
        }

        Ok(())
    }

    pub(crate) fn decode_and_push(
        &mut self,
        enc: EncapsulatedPacket,
        out: &mut Vec<RaknetPacket>,
    ) -> Result<(), DecodeError> {
        let mut buf = enc.payload.clone();

        let pkt = match RaknetPacket::decode(&mut buf) {
            Ok(pkt) => pkt,
            Err(DecodeError::UnknownId(id)) => {
                let body = if !enc.payload.is_empty() {
                    enc.payload.slice(1..)
                } else {
                    Bytes::new()
                };
                RaknetPacket::UserData { id, payload: body }
            }
            Err(e) => return Err(e),
        };

        if let RaknetPacket::EncapsulatedAck(payload) = pkt {
            self.incoming_acks.extend(payload.0.ranges);
            return Ok(());
        }
        if let RaknetPacket::EncapsulatedNak(payload) = pkt {
            self.incoming_naks.extend(payload.0.ranges);
            return Ok(());
        }

        out.push(pkt);
        Ok(())
    }

    pub(crate) fn process_incoming_acks_naks(&mut self, now: Instant) {
        self.process_incoming_acks(now);
        self.process_incoming_naks(now);
    }

    fn process_incoming_acks(&mut self, now: Instant) {
        while let Some(range) = self.incoming_acks.pop_front() {
            Self::for_each_sequence_in_range(range, |seq| {
                if let Some(tracked) = self.sent_datagrams.remove(&seq)
                    && let crate::protocol::datagram::DatagramPayload::EncapsulatedPackets(_) =
                        &tracked.datagram.payload
                {
                    self.sliding
                        .on_ack(now, &tracked.datagram, seq, tracked.send_time);
                }
            });
        }
    }

    fn process_incoming_naks(&mut self, now: Instant) {
        while let Some(range) = self.incoming_naks.pop_front() {
            Self::for_each_sequence_in_range(range, |seq| {
                if let Some(mut tracked) = self.sent_datagrams.remove(&seq)
                    && let crate::protocol::datagram::DatagramPayload::EncapsulatedPackets(_) =
                        &tracked.datagram.payload
                {
                    self.sliding.on_nak();
                    tracked.next_send = now;
                    self.sent_datagrams.insert(seq, tracked);
                }
            });
        }
    }

    fn for_each_sequence_in_range<F>(range: SequenceRange, mut f: F)
    where
        F: FnMut(Sequence24),
    {
        let mut seq = range.start;
        loop {
            f(seq);
            if seq == range.end {
                break;
            }
            seq = seq.next();
        }
    }
}
