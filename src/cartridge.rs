use crate::util::{combine_nibbles, nth_bit};

trait Mapper {
    fn map(prg_rom: &Vec<u8>, chr_rom: &Vec<u8>, address: u16) -> u8;
}

#[derive(Debug)]
struct Mapper000 {}

impl Mapper for Mapper000 {

    fn map(prg_rom: &Vec<u8>, chr_rom: &Vec<u8>, address: u16) -> u8 {
        let boundary = 0x7FFF;
        if 0x6000 >= address && 0x7FFF <= address {
            return prg_rom[(address & boundary) as usize]
        }
        panic!("Unknown address to map");
    }
}

#[derive(Debug)]
pub struct Cartridge {
    prg_rom_banks: u8,
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper_code: u8,
}

impl Cartridge {

    // TODO: Add mocking, use ONLY for testing
    pub fn new() -> Cartridge {
        return Cartridge {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper_code: 0,
            prg_rom_banks: 0
        }
    }

    pub fn read(&mut self, address: u16) -> u8 {
        return self.map_address(address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let mapped = self.map_address(address);
        // TODO
    }

    fn map_address(&mut self, address: u16) -> u8 {
        match self.mapper_code {
            000 => Mapper000::map(&self.prg_rom, &self.chr_rom, address),
            _ => panic!("Unknown mapper code")
        }
    }
}

#[derive(Debug)]
pub struct CartridgeLoader {
    payload: Vec<u8>
}

impl CartridgeLoader {
    pub fn load_cartridge(payload: Vec<u8>) -> Cartridge {
        let mut loader = CartridgeLoader { payload };
        loader.assert_constant();
        let mapper_code = loader.load_mapper();
        let prg_rom = loader.load_prg();
        let prg_rom_banks = loader.prg_size();
        let chr_rom = loader.load_chr();
        return Cartridge {
            prg_rom_banks,
            prg_rom,
            chr_rom,
            mapper_code,
        }
    }

    fn assert_constant(&mut self) {
        let header_constant_start = 0;
        let header_constant_end = 3;
        let header_constant_combination: Vec<u8> = vec![0x4E, 0x45, 0x53, 0x1A];
        let valid_header = self.payload[header_constant_start..header_constant_end] == *header_constant_combination;
        if !valid_header {
            panic!("ROM does not contain the usual header");
        }
    }

    fn load_mapper(&mut self) -> u8 {
        let lower_mapper_flag = 6;
        let upper_mapper_flag = 7;
        let lower_nibble = (self.payload[lower_mapper_flag] & 0x10) >> 4;
        let upper_nibble = self.payload[upper_mapper_flag] & 0x10;
        let mapper_code = lower_nibble | upper_nibble;
        return mapper_code
    }

    fn prg_size(&mut self) -> u8 {
        let prg_rom_size_flag = 4;
        return self.payload[prg_rom_size_flag] * 2;
    }

    fn chr_size(&mut self) -> u8 {
        let chr_size_flag = 5;
        return self.payload[chr_size_flag]
    }

    fn trainer_offset(&mut self) -> u16 {
        let trainer_flag = 6;
        let has_trainer = nth_bit(self.payload[trainer_flag], 3);
        return if has_trainer {
            512
        } else {
            0
        }
    }

    fn load_prg(&mut self) -> Vec<u8> {
        let header_offset = 16;
        let prg_start = (header_offset + self.trainer_offset()) as usize; // HEADER - 16 bytes + Trainer 512 BYTES
        let size = self.prg_size() as usize;
        return self.payload[prg_start..(prg_start + size)].to_vec()
    }

    fn load_chr(&mut self) -> Vec<u8> {
        let header_offset = 16;
        let trainer_offset = self.trainer_offset();
        let prg_offset = self.prg_size() as u16;
        let chr_size = self.chr_size() as usize;
        let chr_start = (header_offset + trainer_offset + prg_offset) as usize;
        return self.payload[chr_start..(chr_start + chr_size)].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cartridge() {
        let cartridge = CartridgeLoader::load_cartridge(vec![0, 0, 0, 0, 0x91, 0x82]);
        assert_eq!(cartridge.mapper_code, 0x12)
    }
}
