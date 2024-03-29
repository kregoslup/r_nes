use std::u8;
use std::ops::BitOr;
use log::{info, warn};
use std::path::Path;
use std::fs::File;
use std::io::Read;

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

pub fn read_file(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data);
    return data;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_first_bit() {
        let value: u8 = 0b00_01_00_00;
        info!("{:?} 7", nth_bit(value, 7));
        info!("{:?} 6", nth_bit(value, 6));
        info!("{:?} 5", nth_bit(value, 5));
        info!("{:?} 4", nth_bit(value, 4));
        info!("{:?} 3", nth_bit(value, 3));
        info!("{:?} 2", nth_bit(value, 2));
        info!("{:?} 1", nth_bit(value, 1));
        info!("{:?} 0", nth_bit(value, 0));
        assert_eq!(nth_bit(value, 7) as u8, 0);
        assert_eq!(nth_bit(value, 6) as u8, 0);
        assert_eq!(nth_bit(value, 5) as u8, 0);
        assert_eq!(nth_bit(value, 4) as u8, 1);
        assert_eq!(nth_bit(value, 3) as u8, 0);
        assert_eq!(nth_bit(value, 2) as u8, 0);
        assert_eq!(nth_bit(value, 1) as u8, 0);
        assert_eq!(nth_bit(value, 0) as u8, 0)
    }

    #[test]
    fn test_get_first_bit_v2() {
        let value: u8 = 0b00_00_00_01;
        info!("{:?} 7", nth_bit(value, 7));
        info!("{:?} 6", nth_bit(value, 6));
        info!("{:?} 5", nth_bit(value, 5));
        info!("{:?} 4", nth_bit(value, 4));
        info!("{:?} 3", nth_bit(value, 3));
        info!("{:?} 2", nth_bit(value, 2));
        info!("{:?} 1", nth_bit(value, 1));
        info!("{:?} 0", nth_bit(value, 0));
        assert_eq!(nth_bit(value, 7) as u8, 0);
        assert_eq!(nth_bit(value, 6) as u8, 0);
        assert_eq!(nth_bit(value, 5) as u8, 0);
        assert_eq!(nth_bit(value, 4) as u8, 0);
        assert_eq!(nth_bit(value, 3) as u8, 0);
        assert_eq!(nth_bit(value, 2) as u8, 0);
        assert_eq!(nth_bit(value, 1) as u8, 0);
        assert_eq!(nth_bit(value, 0) as u8, 1)
    }

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