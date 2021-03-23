use std::path::Path;
use std::fs::File;
use log::{info, warn};
use std::io::Read;
use dirs::home_dir;
use std::fmt::Debug;
use crate::cartridge::{Cartridge, CartridgeLoader};
use crate::ppu::Ppu;
use crate::cpu::Cpu;
use crate::screen::Screen;

static RAM_MIRROR_BOUNDARY: u16 = 0x07FF;
static RAM_BOUNDARY: u16 = 0x1FFF;

static PPU_MIRROR_BOUNDARY: u16 = 0x2007;
static PPU_BOUNDARY: u16 = 0x3FFF;

static CARTRIDGE_LOWER_BOUNDARY: u16 = 0x4020;
static MEMORY_MAP_BOUNDARY: u16 = 0xFFFF;

static APU_LOWER_BOUNDARY: u16 = 0x4000;
static APU_UPPER_BOUNDARY: u16 = 0x401F;

#[derive(Debug)]
pub struct Bus {
    memory: Vec<u8>,
    ppu: Ppu,
    cartridge: Cartridge,
    pub nmi: bool
}

impl Bus {
    pub(crate) fn new(memory: Vec<u8>, ppu: Ppu, cartridge: Cartridge) -> Bus {
        Bus {
            memory,
            ppu,
            cartridge,
            nmi: false
        }
    }

    pub fn emulate(&mut self, screen: &Screen) {
        let previous_state = self.ppu.nmi_occurred;
        self.ppu.emulate(screen);
        self.nmi = !previous_state & self.ppu.nmi_occurred;
    }

    pub fn fetch(&mut self, address: u16) -> u8 {
        if self.is_ram(address) {
            self.memory[self.as_ram_address(address) as usize]
        } else if self.is_ppu(address) {
            self.ppu.fetch(self.as_ppu_address(address))
        } else if self.is_cartridge(address) {
            self.cartridge.cpu_read(address)
        } else if self.is_apu(address) {
            info!("Accessing APU");
            return 0;
        } else {
            panic!("Memory address not supported, {:#01X}", address)
        }
    }

    pub fn store(&mut self, value: u8, address: u16) {
        if self.is_ram(address) {
            let as_ram_address = self.as_ram_address(address) as usize;
            info!("Storing value {:#01X} at address {:#01X}", value, as_ram_address);
            self.memory[as_ram_address] = value;
        } else if self.is_ppu(address) {
            self.ppu.save(self.as_ppu_address(address), value)
        } else if self.is_cartridge(address) {
            unimplemented!();
        } else if self.is_apu(address) {
            info!("Writing APU");
        } else {
            panic!("Memory address not supported, {:#01X}", address)
        }
    }

    fn is_apu(&self, address: u16) -> bool {
        return (address >= APU_LOWER_BOUNDARY) & (address <= APU_UPPER_BOUNDARY)
    }

    fn is_ram(&self, address: u16) -> bool {
        return address <= RAM_BOUNDARY
    }

    fn is_ppu(&self, address: u16) -> bool {
        return (address > RAM_BOUNDARY) & (address <= PPU_BOUNDARY)
    }

    fn is_cartridge(&self, address: u16) -> bool {
        return (address >= CARTRIDGE_LOWER_BOUNDARY) & (address <= MEMORY_MAP_BOUNDARY)
    }

    fn as_ram_address(&self, address: u16) -> u16 {
        return address & RAM_MIRROR_BOUNDARY
    }

    fn as_ppu_address(&self, address: u16) -> u16 {
        return address & PPU_MIRROR_BOUNDARY
    }
}
