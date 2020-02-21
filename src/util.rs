use std::u8;
use std::ops::BitOr;

pub fn wrapping_add(lhs: u8, rhs: u8) -> (u8, bool) {
    lhs.overflowing_add(rhs)
}

pub fn wrapping_sub(lhs: u8, rhs: u8) -> (u8, bool) {
    lhs.overflowing_sub(rhs)
}

pub fn combine_u8(lsb: u8, msb: u8) -> u16 {
    ((msb << 7) as u16).bitor(lsb as u16)
}

pub fn msb(value: u8) -> u8 {
    value >> 7
}
