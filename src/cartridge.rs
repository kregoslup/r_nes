use crate::util::{combine_nibbles, nth_bit};

trait Mapper {
    fn map(address: u16) -> u16;
}

pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8
}

impl Cartridge {
    pub fn read(&mut self, address: u16) -> u8 {
        let mapped = mapper.map(address);
    }

    pub fn write(&mut self, address: u16, value: u8) {
        let mapped = mapper.map(address);
    }
}

pub struct CartridgeLoader {
    payload: Vec<u8>
}

impl CartridgeLoader {
    pub fn load_cartridge(payload: Vec<u8>) -> Box<Cartridge> {
        let mut loader = CartridgeLoader { payload };
        loader::assert_constant(&payload);
        let mapper = loader.load_mapper();
        let prg_rom = loader.load_prg();
        let chr_rom = loader.load_chr();
        return Cartridge {
            prg_rom,
            chr_rom,
            mapper
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
        let upper_nibble = self.payload[lower_mapper_flag] & 0x10;
        let mapper_code = lower_nibble | upper_nibble;
        // return mapper fetch
    }

    fn prg_size(&mut self) -> u8 {
        let prg_rom_size_flag = 4;
        return self.payload[prg_rom_size_flag] * 2;
    }

    fn chr_size(&mut self) -> u8 {
        let chr_size_flag = 5;
        return self.payload[chr_size_flag]
    }

    fn trainer_offset(&mut self) -> u8 {
        let trainer_flag = 6;
        let has_trainer = nth_bit(self.payload[trainer_flag], 3);
        return if has_trainer() {
            512
        } else {
            0
        }
    }

    fn load_prg(&mut self) -> Vec<u8> {
        let header_offset = 16;
        let prg_start = header_offset + self.trainer_offset(); // HEADER - 16 bytes + Trainer 512 BYTES
        let size = self.prg_size();
        return self.payload[prg_start..(prg_start + size)]
    }

    fn load_chr(&mut self) -> Vec<u8> {
        let header_offset = 16;
        let trainer_offset = self.trainer_offset();
        let prg_offset = self.prg_size();
        let chr_size = self.chr_size();
        let chr_start = header_offset + trainer_offset + prg_offset;
        return self.payload[chr_start..(chr_start + chr_size)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_cartridge() {
        let cartridge = CartridgeLoader::load_cartridge(vec![0, 0, 0, 0, 0x91, 0x82]);
        assert_eq!(cartridge.mapper, 0x12)
    }
}
