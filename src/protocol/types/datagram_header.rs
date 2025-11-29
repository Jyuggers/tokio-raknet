use bytes::{Buf, BufMut};

use crate::protocol::{
    constants::DatagramFlags,
    packet::{DecodeError, RaknetEncodable},
    types::Sequence24,
};

pub struct DatagramHeader {
    pub flags: DatagramFlags,
    pub sequence: Sequence24,
}

impl DatagramHeader {
    pub fn encode(&self, dst: &mut impl BufMut) {
        dst.put_u8(self.flags.bits());
        self.sequence.encode_raknet(dst);
    }

    pub fn decode(src: &mut impl Buf) -> Result<Self, DecodeError> {
        if src.remaining() < 4 {
            return Err(DecodeError::UnexpectedEof);
        }
        let raw_flags = src.get_u8();
        let flags = DatagramFlags::from_bits_truncate(raw_flags);
        let sequence = Sequence24::decode_raknet(src)?;
        Ok(DatagramHeader { flags, sequence })
    }
}
