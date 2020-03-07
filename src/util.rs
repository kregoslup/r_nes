use std::u8;
use std::ops::BitOr;

pub fn combine_u8(lsb: u8, msb: u8) -> u16 {
    ((msb << 7) as u16).bitor(lsb as u16)
}

pub fn msb(value: u8) -> u8 {
    value >> 7
}

pub fn lsb(value: u8) -> u8 {
    value & 0b0000_0001
}