use std::u8;
use std::ops::BitOr;

pub fn combine_u8(lsb: u8, msb: u8) -> u16 {
    ((msb << 7) as u16).bitor(lsb as u16)
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