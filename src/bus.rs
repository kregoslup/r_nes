use std::path::Path;
use std::fs::File;
use std::io::Read;
use dirs::home_dir;
use std::fmt::Debug;
use crate::cartridge::Cartridge;
use crate::ppu::Ppu;

static RAM_MIRROR_BOUNDARY: u16 = 0x07FF;
static RAM_BOUNDARY: u16 = 0x1FFF;

static PPU_MIRROR_BOUNDARY: u16 = 0x2007;
static PPU_BOUNDARY: u16 = 0x3FFF;

static CARTRIDGE_LOWER_BOUNDARY: u16 = 0x4020;
static MEMORY_MAP_BOUNDARY: u16 = 0xFFFF;

#[derive(Debug)]
pub struct Bus {
    memory: Vec<u8>,
    ppu: Ppu
//    cartridge: Cartridge
}

impl Bus {

    pub fn new(memory: Vec<u8>, ppu: Ppu) -> Bus {
        Bus {
            memory,
            ppu
        }
    }

    pub fn fetch(&self, address: u16) -> u8 {
        if address <= RAM_BOUNDARY {
            self.memory[(address & RAM_MIRROR_BOUNDARY) as usize]
        } else if (address > RAM_BOUNDARY) & (address <= PPU_BOUNDARY) {
            // ADD PPU
            ppu.fetch(address)
        } else if (address >= CARTRIDGE_LOWER_BOUNDARY) & (address <= MEMORY_MAP_BOUNDARY) {
            // TODO: call cartridge
            unimplemented!();
        } else {
            panic!("Memory address not supported, {:#01X}", address)
        }
    }

    pub fn store(&mut self, value: u8, address: u16) {
        if address <= RAM_BOUNDARY {
            self.memory[(address & RAM_MIRROR_BOUNDARY) as usize] = value;
        } else if (address > RAM_BOUNDARY) & (address <= PPU_BOUNDARY) {
            // ADD PPU
            ppu.save(address, value)
        } else if (address >= CARTRIDGE_LOWER_BOUNDARY) & (address <= MEMORY_MAP_BOUNDARY) {
            // TODO: call cartridge
            unimplemented!();
        } else {
            panic!("Memory address not supported, {:#01X}", address)
        }
    }
}

fn read_file(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data);
    return data;
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Needs refactor for CI
    #[test]
    #[ignore]
    fn test_read_file() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        assert_ne!(read_file(tmp_dir.as_path()).len(), 0)
    }
}