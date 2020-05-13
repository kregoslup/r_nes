use std::u8;
use std::ops::BitOr;

pub fn combine_u8(lsb: u8, msb: u8) -> u16 {
    ((msb as u16) << 8).bitor(lsb as u16)
}

pub fn combine_nibbles(lsb: u8, msb: u8) -> u8 {
    (msb << 4).bitor(lsb)
}

pub fn msb(value: u8) -> u8 {
    value >> 7
}

pub fn nth_bit(input: u8, n: u8) -> bool {
    if n < 8 {
        input & (1 << n) != 0
    } else {
        false
    }
}

pub fn lsb(value: u8) -> u8 {
    value & 0b0000_0001
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combine_u8() {
        let lsb = 0x12;
        let msb = 0x34;
        assert_eq!(combine_u8(lsb, msb), 0x3412)
    }

    #[test]
    fn test_combine_nibbles() {
        let lsb = 0x01;
        let msb = 0x02;
        assert_eq!(combine_nibbles(lsb, msb), 0x12)
    }
}