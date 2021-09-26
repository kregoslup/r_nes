use crate::util::{combine_nibbles, nth_bit};
use log::{info, warn};
use std::fs::File;
use std::io::Read;
use crate::ppu::NameTableMirroring;
use crate::ppu::NameTableMirroring::{HORIZONTAL, VERTICAL};

static PRG_ROM_SIZE_FLAG: u8 = 4;

trait Mapper {
    fn map_cpu(prg_rom: &Vec<u8>, banks: u8, address: u16) -> u8;

    // TODO: Remove, not used
    fn map_ppu(chr_rom: &Vec<u8>, address: u16) -> u8;
}

// TODO: trait
#[derive(Debug)]
struct Mapper000 {}

impl Mapper for Mapper000 {

    fn map_cpu(prg_rom: &Vec<u8>, banks: u8, address: u16) -> u8 {
        if address >= 0x8000 && address <= 0xFFFF {
            let mapped_address = address & (if banks > 1 {0x7FFF} else {0x3FFF});
            return prg_rom[mapped_address as usize]
        }
        panic!("Unknown cpu address to map: {:X}", address);
    }

    fn map_ppu(chr_rom: &Vec<u8>, address: u16) -> u8 {
        if 0x0000 >= address && 0x1FFF <= address {
            return chr_rom[address as usize]
        }
        panic!("Unknown ppu address to map: {:X}", address);
    }
}

#[derive(Debug)]
pub struct Cartridge {
    prg_rom_banks: u8,
    prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    mapper_code: u8,
    pub nametable_mirroring: NameTableMirroring
}

impl Cartridge {

    // TODO: Add mocking, use only for testing
    pub fn new() -> Cartridge {
        return Cartridge {
            prg_rom: vec![],
            chr_rom: vec![],
            mapper_code: 0,
            prg_rom_banks: 0,
            nametable_mirroring: HORIZONTAL
        }
    }

    pub fn cpu_read(&mut self, address: u16) -> u8 {
        return self.map_cpu_address(address);
    }

    pub fn ppu_read(&mut self, address: u16) -> u8 {
        return self.map_ppu_address(address)
    }

    pub fn cpu_write(&mut self, address: u16, value: u8) {
        let mapped = self.map_cpu_address(address);
        // TODO
    }

    fn map_cpu_address(&mut self, address: u16) -> u8 {
        match self.mapper_code {
            000 => Mapper000::map_cpu(&self.prg_rom, self.prg_rom_banks, address),
            _ => panic!("Unknown mapper code")
        }
    }

    fn map_ppu_address(&mut self, address: u16) -> u8 {
        match self.mapper_code {
            000 => Mapper000::map_ppu(&self.chr_rom, address),
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
        let prg_rom_banks = loader.prg_banks();
        let chr_rom = loader.load_chr();
        let nametable_mirroring = loader.load_nametable_mirroring();
        return Cartridge {
            prg_rom_banks,
            prg_rom,
            chr_rom,
            mapper_code,
            nametable_mirroring
        }
    }

    fn assert_constant(&mut self) {
        let header_constant_start = 0;
        let header_constant_end = 4;
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

    fn prg_banks(&mut self) -> u8 {
        return self.payload[PRG_ROM_SIZE_FLAG as usize]
    }

    fn prg_size(&mut self) -> u16 {
        return self.payload[PRG_ROM_SIZE_FLAG as usize] as u16 * 16 * 1024; // 16KB * size
    }

    fn chr_size(&mut self) -> u16 {
        let chr_size_flag = 5;
        return self.payload[chr_size_flag] as u16 * 8 * 1024 // 8KB * size!!
    }

    fn trainer_offset(&mut self) -> u16 {
        let trainer_flag = 6;
        let has_trainer = nth_bit(self.payload[trainer_flag], 2); // TODO: Check bit
        return if has_trainer {
            512
        } else {
            0
        }
    }

    fn load_nametable_mirroring(&mut self) -> NameTableMirroring {
        let nametable_flag = 6;
        let mirroring = nth_bit(self.payload[nametable_flag], 0);
        return if mirroring {
            VERTICAL
        } else {
            HORIZONTAL
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
