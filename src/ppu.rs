use crate::screen::Screen;
use crate::util::{nth_bit, combine_u8};

static PPU_ADDRESSABLE_RANGE: u16 = 0x3FF;

#[derive(Debug)]
pub struct Ppu {
    ram: Vec<u8>,
    screen: Screen,
    cycles: u16,
    scanline: u16,
    ppu_status: u8,
    vram_address: u16,
    latch: u8,
    last_register: u8,
    pub nmi_occurred: bool,
    status: u8
}

impl Ppu {
    pub fn new(mut pattern_table: Vec<u8>) -> Ppu {
        pattern_table.append(vec![0 as u8; 0x1FFF].as_mut());
        return Ppu {
            screen: Screen::new(),
            cycles: 0,
            scanline: 0, // or 0
            ppu_status: 0,
            ram: pattern_table,
            latch: 0,
            vram_address: 0,
            last_register: 0,
            nmi_occurred: false,
            status: 0
        }
    }

    pub fn tick(&mut self) {
        if self.get_nmi_output() && self.nmi_occurred {
            println!("[PPU]: NMI OCCURRED");
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
            println!("Setting vblank, nmi output: {}", self.get_nmi_output());
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
        println!("ppu fetch: {:#01X}", address);
        match address {
            0x2000 => self.latch, // PPUCTRL
            0x2001 => self.latch, // PPUMASK
            0x2002 => {
                let mut result = self.latch;
                result &= 0b00_01_11_11;
                if self.is_vblank() {
                    result |= 0b10_00_00_00;
                }
                // TODO: Sprite 0 hit and sprite overflow: https://wiki.nesdev.com/w/index.php/PPU_registers#PPUSTATUS
                self.clear_vblank();
                return result
            }, // PPUSTATUS
            0x2003 => self.latch, // OAMADDR
            0x2004 => self.latch, // OAMDATA
            0x2005 => self.latch, // PPUSCROLL
            0x2006 => self.latch, // PPUADDR
            0x2007 => {
                let result = self.ram[self.vram_address as usize];
                self.vram_address += self.get_vram_increment() as u16;
                return result
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    pub fn save(&mut self, address: u16, value: u8) {
        println!("ppu save: {:#01X} at address {:#01X}", value, address);
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
                self.vram_address = combine_u8(value, self.latch);
                self.latch = value;
            }, // PPUADDR
            0x2007 => {
                self.ram[self.vram_address as usize] = value;
                self.vram_address += self.get_vram_increment() as u16
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
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

    pub fn emulate(&mut self) {
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