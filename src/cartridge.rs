use crate::util::{combine_nibbles};

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8
}

impl Cartridge {
    pub fn load_cartridge(payload: Vec<u8>) -> Cartridge {
        Cartridge::assert_constant(&payload);
        let mapper = Cartridge::load_mapper(&payload);
        let prg_rom = Cartridge::load_prg(&payload);
        let chr_rom = Cartridge::load_chr(&payload);
        return Cartridge {
            prg_rom,
            chr_rom,
            mapper: 0
        }
    }

    fn assert_constant(payload: &Vec<u8>) {
        let header_constant_start = 0;
        let header_constant_end = 3;
        let header_constant_combination: Vec<u8> = vec![0x4E, 0x45, 0x53, 0x1A];
        let valid_header = payload[header_constant_start..header_constant_end] == *header_constant_combination;
        if !valid_header {
            panic!("ROM does not contain the usual header");
        }
    }

    fn load_mapper(payload: &Vec<u8>) -> u8 {
        let lower_mapper_flag = 6;
        let upper_mapper_flag = 7;
        let lower_nibble = (payload[lower_mapper_flag] & 0x10) >> 4;
        let upper_nibble = payload[lower_mapper_flag] & 0x10;
        return lower_nibble | upper_nibble;
    }

    fn load_prg(payload: &Vec<u8>) -> Vec<u8> {
        let prg_start = 2 + 64; // HEADER - 16 bytes + Trainer 512 BYTES
        let prg_rom_size_flag = 4;
        let size = payload[prg_rom_size_flag] * 2; // Size provided in 16 kB units;
        return payload[prg_start..(prg_start + size)]
    }

    fn load_chr(payload: &Vec<u8>) -> Vec<u8> {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cartridge() {
        let cartridge = Cartridge::load_cartridge(vec![0, 0, 0, 0, 0x91, 0x82]);
        assert_eq!(cartridge.mapper, 0x12)
    }
}
