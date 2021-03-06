use crate::screen::Screen;
use crate::util::{nth_bit, combine_u8};

static PPU_ADDRESSABLE_RANGE: u16 = 0x3FF;

#[derive(Debug)]
pub struct Ppu {
    pattern_table: Vec<u8>, // CHR_ROM - sprites
    name_table: Vec<u8>, // VRAM, - layout of background,
    ram: Vec<u8>,
    screen: Screen,
    cycles: u16,
    scanline: u16,
    ppu_status: u8,
    vram_address: u16,
    latch: u8,
    last_register: u8,
    pub nmi_occurred: bool,
    nmi_output: bool,
    base_name_table_address: u32,
    vram_increment: u8,
    sprite_size: u8,
    sprite_pattern_table: u16,
    background_pattern_table_address: u16
    // palette? - colours
}

impl Ppu {
    pub fn new(pattern_table: Vec<u8>, name_table: Vec<u8>) -> Ppu {
        return Ppu {
            pattern_table,
            name_table,
            screen: Screen::new(),
            cycles: 0,
            scanline: 0, // or 0
            ppu_status: 0,
            ram: vec![],
            latch: 0,
            vram_address: 0,
            last_register: 0,
            nmi_occurred: false,
            nmi_output: false,
            base_name_table_address: 0,
            vram_increment: 0,
            sprite_size: 0,
            sprite_pattern_table: 0,
            background_pattern_table_address: 0
        }
    }

    pub fn tick(&mut self) {
        if self.nmi_output && self.nmi_occurred {
            panic!("NMI OCCURRED")
        }
        self.cycles += 1;

//        println!("Cycles: {} scanline: {}", self.cycles, self.scanline);
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
            println!("Setting vblank, nmi ouput: {}", self.nmi_output);
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

    pub fn fetch(&mut self, address: u16) -> u8 {
        println!("ppu fetch: {:#01X}", address);
        match address {
            0x2000 => self.latch, // PPUCTRL
            0x2001 => self.latch, // PPUMASK
            0x2002 => {
                self.latch = 0;
                if nth_bit(self.ppu_status, 7) {
                    0b1000_0000 | (0b0001_1111 & self.latch)
                } else {
                    0
                }
            }, // PPUSTATUS
            0x2003 => self.latch, // OAMADDR
            0x2004 => self.latch, // OAMDATA
            0x2005 => self.latch, // PPUSCROLL
            0x2006 => self.latch, // PPUADDR
            0x2007 => {
                let result = self.pattern_table[self.vram_address as usize];
                self.vram_address += self.vram_increment as u16;
                return result
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    pub fn save(&mut self, address: u16, value: u8) {
        println!("ppu save: {:#01X}", address);
        self.latch = value;
        match address {
            0x2000 => {
                self.set_base_nametable_address(value);
                self.set_vram_increment(value);
                self.set_sprite_pattern_table(value);
                self.set_background_pattern_table(value);
                self.set_sprite_size(value);
                self.ppu_master_slave(value);
                self.set_nmi_output(value);
            }, // PPUCTRL
//            0x2001 => unimplemented!(), // PPUMASK
            0x2002 => {
                self.latch = value;
            }, // PPUSTATUS
//            0x2003 => unimplemented!(), // OAMADDR
//            0x2004 => unimplemented!(), // OAMDATA
//            0x2005 => unimplemented!(), // PPUSCROLL
            0x2006 => {
                self.vram_address = combine_u8(value, self.latch);
                self.latch = value;
            }, // PPUADDR
            0x2007 => {
                self.ram[self.vram_address as usize] = value;
                self.vram_address += self.vram_increment as u16
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    fn set_nmi_output(&mut self, value: u8) {
        self.nmi_output = nth_bit(value, 7)
    }

    fn ppu_master_slave(&mut self, value: u8) {
        // TODO
    }

    fn set_sprite_size(&mut self, value: u8) {
        if nth_bit(value, 3) {
            self.sprite_pattern_table = 16
        } else {
            self.sprite_pattern_table = 8
        }
    }

    fn set_sprite_pattern_table(&mut self, value: u8) {
        if nth_bit(value, 3) {
            self.sprite_pattern_table = 0x1000
        } else {
            self.sprite_pattern_table = 0x0000
        }
    }

    fn set_background_pattern_table(&mut self, value: u8) {
        if nth_bit(value, 4) {
            self.background_pattern_table_address = 0x1000
        } else {
            self.background_pattern_table_address = 0x0000
        }
    }

    fn set_base_nametable_address(&mut self, value: u8) {
        match value & 0b0000_0011 {
            0 => self.base_name_table_address = 0x2000,
            1 => self.base_name_table_address = 0x2400,
            2 => self.base_name_table_address = 0x2800,
            3 => self.base_name_table_address = 0x2C00,
            _ => panic!("Invalid nametable address {:#01X}", value)
        };
    }

    fn set_vram_increment(&mut self, value: u8) {
        if nth_bit(value, 2) {
            self.vram_increment = 32;
        } else {
            self.vram_increment = 1;
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