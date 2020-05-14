use crate::util::{combine_nibbles};

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8
}

impl Cartridge {
    pub fn load_cartridge(payload: Vec<u8>) -> Cartridge {
        let mapper = Cartridge::load_mapper(&payload);
        let prg_rom = Cartridge::load_prg(&payload);
        let chr_rom = Cartridge::load_chr(&payload);
        return Cartridge {
            prg_rom,
            chr_rom,
            mapper: 0
        }
    }

    fn load_mapper(payload: &Vec<u8>) -> u8 {
        let lower_mapper_flag = 5;
        let upper_mapper_flag = 6;
        let lower_nibble = payload[lower_mapper_flag];
        let upper_nibble = payload[lower_mapper_flag];
        return combine_nibbles(lower_nibble, upper_nibble);
    }

    fn load_prg(payload: &Vec<u8>) -> Vec<u8> {

    }

    fn load_chr(payload: &Vec<u8>) -> Vec<u8> {

    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_mapper() {
        let cartridge = Cartridge::load_cartridge(vec![0, 0, 0, 0, 0x01, 0x02]);
        assert_eq!(cartridge.mapper, 0x12)
    }
}
