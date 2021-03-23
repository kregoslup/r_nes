use crate::screen::Screen;
use crate::util::{nth_bit, combine_u8};

use log::{info, warn};
use std::num::Wrapping;
use crate::ppu::NameTableMirroring::{HORIZONTAL, VERTICAL};

static PPU_ADDRESSABLE_RANGE: u16 = 0x3FF;

#[derive(Clone, Copy, Debug)]
pub enum NameTableMirroring {
    HORIZONTAL, VERTICAL
}

#[derive(Debug)]
pub struct Ppu {
    ram: Vec<u8>,
    cycles: u16,
    scanline: u16,
    ppu_status: u8,
    vram_address: u16,
    nametable_mirroring: NameTableMirroring,
    latch: u8,
    last_register: u8,
    pub nmi_occurred: bool,
    status: u8
}

impl Ppu {
    pub fn new(mut chr_rom: Vec<u8>, mirroring: NameTableMirroring) -> Ppu {
        chr_rom.append(vec![0 as u8; 0x1FFF].as_mut());
        return Ppu {
            cycles: 0,
            scanline: 0, // or 0
            ppu_status: 0,
            ram: chr_rom,
            nametable_mirroring: mirroring,
            latch: 0,
            vram_address: 0,
            last_register: 0,
            nmi_occurred: false,
            status: 0
        }
    }

    pub fn tick(&mut self) {
        if self.get_nmi_output() && self.nmi_occurred {
            info!("[PPU]: NMI OCCURRED");
        }
        self.cycles += 1;

        if self.cycles == 341 {
            self.scanline += 1;
            self.cycles = 0;
        }

        if (0 <= self.scanline) && (self.scanline >= 239) {
            // draw
        }

        if (self.scanline == 261) && (self.cycles == 1) {
            self.scanline = 0;
            self.clear_vblank();
            self.nmi_occurred = false;
        } else if (self.scanline == 241) && (self.cycles == 1) {
            info!("Setting vblank, nmi output: {}", self.get_nmi_output());
            self.set_vblank();
            self.nmi_occurred = true;
        }
    }

    fn set_vblank(&mut self) {
        self.ppu_status |= 0b1000_0000
    }

    fn clear_vblank(&mut self) {
        self.ppu_status &= 0b0000_0000
    }

    fn is_vblank(&self) -> bool {
        (self.ppu_status & 0b1000_0000) != 0
    }

    pub fn fetch(&mut self, address: u16) -> u8 {
        info!("ppu fetch: {:#01X}", address);
        match address {
            0x2000 => self.latch, // PPUCTRL
            0x2001 => self.latch, // PPUMASK
            0x2002 => {
                let mut result = self.latch;
                result &= 0b00_01_11_11;
                if self.is_vblank() {
                    result |= 0b10_00_00_00;
                }
                self.clear_vblank();
                return result
            }, // PPUSTATUS
            0x2003 => self.latch, // OAMADDR
            0x2004 => self.latch, // OAMDATA
            0x2005 => self.latch, // PPUSCROLL
            0x2006 => self.latch, // PPUADDR
            0x2007 => {
                match self.vram_address {
                    0..=0x1FFF => {
                        let address = self.get_vram_address();
                        let result = self.ram[address];
                        self.increment_vram();
                        result
                    },
                    0x2000..=0x2FFF => {
                        let mut mirrored_down = self.get_vram_address() & 0x2FFF;
                        let vram_table = mirrored_down / 0x400;
                        let address = match (self.nametable_mirroring, vram_table) {
                            (HORIZONTAL, 1) | (HORIZONTAL, 3) => mirrored_down - 0x400,
                            (VERTICAL, 1) | (VERTICAL, 3) => mirrored_down - 0x800,
                            _ => mirrored_down
                        };
                        self.ram[address as usize]
                    }
                    _ => panic!("Unkown vram address: {:#01X}", self.vram_address)
                }

            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    pub fn save(&mut self, address: u16, value: u8) {
        info!("ppu save: {:#01X} at address {:#01X}", value, address);
        self.latch = value;
        match address {
            0x2000 => {
                self.status = value;
            }, // PPUCTRL
            0x2001 => {}, // PPUMASK
            0x2002 => {
                self.latch = value;
            }, // PPUSTATUS
            0x2003 => {}, // OAMADDR
            0x2004 => {}, // OAMDATA
            0x2005 => {}, // PPUSCROLL
            0x2006 => {
                self.vram_address = combine_u8(self.latch, value);
                self.latch = value;
            }, // PPUADDR
            0x2007 => {
                let address = self.get_vram_address();
                info!("PPU 2007 saving {:#01X} at address {:#01X}", value, address);
                self.ram[address] = value;
                self.increment_vram();
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    fn get_vram_address(&mut self) -> usize {
        (self.vram_address & 0x3FFF) as usize
    }

    fn increment_vram(&mut self) {
        self.vram_address = (Wrapping(self.vram_address) + Wrapping(self.get_vram_increment() as u16)).0;
    }

    fn get_nmi_output(&mut self) -> bool {
        nth_bit(self.status, 7)
    }

    fn ppu_master_slave(&mut self) {
        // TODO
    }

    fn get_sprite_size(&mut self) -> u8 {
        if nth_bit(self.status, 3) {
            16
        } else {
            8
        }
    }

    fn get_sprite_pattern_table(&mut self) -> u16 {
        if nth_bit(self.status, 3) {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_background_pattern_table(&mut self) -> u16 {
        if nth_bit(self.status, 4) {
            0x1000
        } else {
            0x0000
        }
    }

    fn get_base_nametable_address(&mut self) -> u16 {
        match self.status & 0b0000_0011 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => panic!("Invalid nametable address {:#01X}", self.status)
        }
    }

    fn get_vram_increment(&mut self) -> u8 {
        if nth_bit(self.status, 2) {
            32
        } else {
            1
        }
    }

    pub fn fetch_internal(&mut self, address: u16) -> u8 {
        let real_address = address & PPU_ADDRESSABLE_RANGE;
        return 0;
    }

    pub fn save_internal(&mut self, address: u16, value: u8) {
        let real_address = address & PPU_ADDRESSABLE_RANGE;
    }

    fn evaluate_background(&mut self) {

    }

    pub fn emulate(&mut self, screen: &Screen) {
        self.tick();
        self.tick();
        self.tick();
    }

    fn get_tile(&mut self, address: u16) -> Vec<u8> {
        return Vec::new()
//        let left_plane: [u8; 16] = self.get_left();
//        let right_plane: [u8; 16] = self.get_right();
//        return left_plane.iter().zip(&right_plane).map(|a, b| a + b).collect();
    }

    fn get_colour(address: u16) -> u8 {
        let palette_lower_boundary = 0x3F00;
        let palette_upper_boundary = 0x3F1D;
        if address >= palette_upper_boundary || address <= palette_lower_boundary {
            panic!("Unknown colour");
        }

        return 0
    }
}