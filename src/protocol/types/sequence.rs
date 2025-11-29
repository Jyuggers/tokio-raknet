use std::ops::Add;

use crate::protocol::{packet::RaknetEncodable, types::U24LE};

const MODULO: u32 = 1 << 24;
const MASK: u32 = MODULO - 1;
const HALF: u32 = MODULO / 2;

/// Sequence type for a U24.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Sequence24(u32);

impl Sequence24 {
    pub fn new(v: u32) -> Sequence24 {
        Sequence24(v & MASK)
    }

    pub fn value(&self) -> u32 {
        self.0 & MASK
    }

    // clone mutations.

    pub fn next(&self) -> Sequence24 {
        Sequence24::new(self.0 + 1)
    }

    pub fn prev(&self) -> Sequence24 {
        Sequence24(if self.0 == 0 { MASK } else { self.0 - 1 })
    }
}

impl Ord for Sequence24 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let mut d = (other.value() - self.value()) as i32;
        if d > HALF as i32 {
            d -= MODULO as i32;
        } else if d < -(HALF as i32) {
            d += MODULO as i32;
        }
        d.cmp(&0)
    }
}

impl PartialOrd for Sequence24 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Add for Sequence24 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut value = self.value() + rhs.value();
        value %= MODULO;

        Sequence24::new(value)
    }
}

impl Add<i32> for Sequence24 {
    type Output = Self;

    fn add(self, rhs: i32) -> Self::Output {
        let mut value = self.value() as i32 + rhs;
        value %= MODULO as i32;

        if value < 0 {
            value += MODULO as i32;
        }

        Sequence24::new(value as u32)
    }
}

impl From<&Sequence24> for U24LE {
    fn from(value: &Sequence24) -> Self {
        U24LE(value.value())
    }
}

impl From<Sequence24> for U24LE {
    fn from(seq: Sequence24) -> Self {
        U24LE(seq.value())
    }
}

impl From<U24LE> for Sequence24 {
    fn from(raw: U24LE) -> Self {
        Sequence24::new(raw.0)
    }
}

impl RaknetEncodable for Sequence24 {
    fn encode_raknet(&self, dst: &mut impl bytes::BufMut) {
        U24LE::from(self).encode_raknet(dst);
    }

    fn decode_raknet(
        src: &mut impl bytes::Buf,
    ) -> Result<Self, crate::protocol::packet::DecodeError> {
        Ok(Sequence24::new(U24LE::decode_raknet(src)?.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_on_next() {
        let max = Sequence24::new(MASK);
        assert_eq!(max.next().value(), 0);
    }

    #[test]
    fn ordering_handles_wrap() {
        let a = Sequence24::new(MASK);
        let b = a.next();
        assert!(b > a);
    }
}
