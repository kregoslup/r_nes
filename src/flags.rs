use std::fmt;
use bitflags::_core::fmt::{Formatter, Error};

bitflags! {
    pub struct Flags: u8 {
        const NEGATIVE = 0b1000_0000;
        const OVERFLOW = 0b0100_0000;
        const PLACEHOLDER = 0b0010_0000;
        const BRK = 0b0001_0000;
        const DECIMAL = 0b0000_1000;
        const IRQ_DIS = 0b0000_0100;
        const ZERO = 0b0000_0010;
        const CARRY = 0b0000_0001;
    }
}

impl Default for Flags {
    fn default() -> Flags {
        Flags::PLACEHOLDER
    }
}

impl Flags {
    pub fn set_flag(&mut self, value: bool, flag: Flags) {
        if value {
            self.insert(flag)
        } else {
            self.remove(flag)
        }
    }
}

impl From<Flags> for u8 {
    fn from(flag: Flags) -> Self {
        flag.bits()
    }
}

impl From<u8> for Flags {
    fn from(item: u8) -> Self {
        Flags::from_bits_truncate(item)
    }
}

impl fmt::Display for Flags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#08b}", self.bits)
    }
}
