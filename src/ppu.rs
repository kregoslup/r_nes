use crate::screen::Screen;
use crate::util::{nth_bit, combine_u8};

use log::{info, warn};
use std::num::Wrapping;
use crate::ppu::NameTableMirroring::{HORIZONTAL, VERTICAL};

static PPU_ADDRESSABLE_RANGE: u16 = 0x3FF;

static PALETTE: &'static [(u8, u8, u8)] = &[
    (84,  84,  84),    (0,  30, 116),   ( 8,  16, 144),   (48,   0, 136),   (68,   0, 100),   (92,   0,  48),   (84,   4,   0),   (60,  24,   0),   (32,  42,   0),   ( 8,  58,   0),   ( 0,  64,   0),   ( 0,  60,   0),   ( 0,  50,  60),   ( 0,   0,   0), (0, 0, 0), (0, 0, 0),
    (152, 150, 152),    (8,  76, 196),  ( 48,  50, 236),  ( 92,  30, 228),  (136,  20, 176),  (160,  20, 100),  (152,  34,  32),  (120,  60,   0),  ( 84,  90,   0),  ( 40, 114,   0),  (  8, 124,   0),  (  0, 118,  40),  (  0, 102, 120),  (  0,   0,   0), (0, 0, 0), (0, 0, 0),
    (236, 238, 236),   (76, 154, 236),  (120, 124, 236),  (176,  98, 236),  (228,  84, 236),  (236,  88, 180),  (236, 106, 100),  (212, 136,  32),  (160, 170,   0),  (116, 196,   0),  ( 76, 208,  32),  ( 56, 204, 108),  ( 56, 180, 204),  ( 60,  60,  60), (0, 0, 0), (0, 0, 0),
    (236, 238, 236),  (168, 204, 236),  (188, 188, 236),  (212, 178, 236),  (236, 174, 236),  (236, 174, 212),  (236, 180, 176),  (228, 196, 144),  (204, 210, 120),  (180, 222, 120),  (168, 226, 144),  (152, 226, 180),  (160, 214, 228),  (160, 162, 160), (0, 0, 0), (0, 0, 0),
];

static STARTUP_CYCLES: u64 = 1_000_000;

#[derive(Clone, Copy, Debug)]
pub enum NameTableMirroring {
    HORIZONTAL, VERTICAL
}

#[derive(Clone, Debug, Copy)]
pub struct Colour {
    pub(crate) r: u8,
    pub(crate) g: u8,
    pub(crate) b: u8
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
    status: u8,
    current_pixel: u16,
    frame: Vec<(u16, u16, Colour)>,
    internal_buffer: u8,
    oam: Vec<u8>,
    oam_address: u8,
    total_cycles: u64
}

impl Ppu {
    pub fn new(mut chr_rom: Vec<u8>, mirroring: NameTableMirroring) -> Ppu {
        chr_rom.append(vec![0 as u8; 0x1FFF].as_mut());
        return Ppu {
            cycles: 0,
            scanline: 0,
            ppu_status: 0,
            ram: chr_rom,
            nametable_mirroring: mirroring,
            latch: 0,
            vram_address: 0,
            last_register: 0,
            nmi_occurred: false,
            status: 0,
            current_pixel: 0,
            frame: Ppu::new_frame(),
            internal_buffer: 0,
            oam: vec![0 as u8; 256],
            oam_address: 0,
            total_cycles: 0
        }
    }

    fn new_frame() -> Vec<(u16, u16, Colour)> {
        vec![(0, 0, Colour{r: 0, g: 0, b: 0}); 960]
    }

    pub fn tick(&mut self, screen: &mut Screen) {
        if self.get_nmi_output() && self.nmi_occurred {
            info!("[PPU]: NMI OCCURRED");
        }
        self.cycles += 1;
        self.total_cycles = (Wrapping(self.total_cycles) + Wrapping(1)).0;

        if self.cycles == 341 {
            self.scanline += 1;
            self.cycles = 0;
        }

        if (0 <= self.scanline) && (self.scanline <= 239) {

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

    fn draw(&mut self, screen: &mut Screen) {
        if self.total_cycles > STARTUP_CYCLES {
            self.draw_tile(screen);
            if self.current_pixel >= 960 {
                self.current_pixel = 0;
                self.draw_sprites(screen);
                screen.draw_pixels(&self.frame);
                self.frame = Ppu::new_frame();
            }
        }
    }

    pub fn get_sprite_color(&mut self, palette_idx: u8) -> Vec<u16> {
        let background: u16 = 0x3F00;
        let palette_increment: u16 = (0x11 + (4 * (palette_idx))) as u16;
        return Vec::from([background, background + palette_increment, background + palette_increment + 1, background + palette_increment + 2]);
    }

    pub fn draw_sprites(&mut self, screen: &mut Screen) {
        for i in (0..self.oam.len()).step_by(4) {
            let tile_y = self.oam[i] as u16;
            let tile_idx = self.oam[i + 1] as u16;
            let attributes = self.oam[i + 2];
            let tile_x = self.oam[i + 3] as u16;

            let flip_horizontal = nth_bit(attributes, 6);
            let flip_vertical = nth_bit(attributes, 7);

            let palette_idx = 0b11 & attributes;
            let colors = self.get_sprite_color(palette_idx);

            let pattern_table_start = self.get_sprite_pattern_table();
            let tile = &self.ram[(pattern_table_start + (tile_idx * 16)) as usize..=(pattern_table_start + 15 + (tile_idx * 16)) as usize];

            if tile_y > 239 { continue };
            if tile_x > 249 { continue };

            for x in 0..=7 {
                let left = tile[x as usize];
                let right = tile[(x + 8) as usize];

                for y in 0..=7 {
                    let left_pixel = nth_bit(left, 7 - y);
                    let right_pixel = nth_bit(right, 7 - y);

                    let pixel = ((right_pixel as u8) << 1) | left_pixel as u8;
                    if pixel == 0 { continue };
                    let chosen_colour = self.ram[colors[pixel as usize] as usize];
                    let rgb = PALETTE[chosen_colour as usize];

                    let (cor_x, cor_y) = match (flip_horizontal, flip_vertical) {
                        (true, false) => {(tile_x + x as u16, tile_y + y as u16)},
                        (true, true) => {(tile_x + x as u16, tile_y + y as u16)},
                        (false, false) => {(tile_x + x as u16, tile_y + y as u16)},
                        (false, true) => {(tile_x + x as u16, tile_y + y as u16)},
                    };
                    self.frame.push((cor_x as u16, cor_y as u16, Colour{r: rgb.0, g: rgb.1, b: rgb.2}))
                }
            }
        }
    }

    pub fn draw_tile(&mut self, screen: &mut Screen) {
        let address = self.get_base_nametable_address() + self.current_pixel as u16;
        let tile_address = self.ram[address as usize] as u16;
        let pattern_idx = self.get_background_pattern_table() as u16 + (tile_address * 16);
        let tile = &self.ram[(pattern_idx) as usize..=((pattern_idx) + 15) as usize];
        let mut tile_row = self.current_pixel % 32 as u16;
        let mut tile_column = self.current_pixel / 32 as u16;
        let colours = self.get_background_colour(tile_row, tile_column);
        let mut cor_x = tile_row * 8;
        let mut cor_y = tile_column * 8;

        for x in 0..=7 {
            let left = tile[x as usize];
            let right = tile[(x + 8) as usize];

            for y in 0..=7 {
                let left_pixel = nth_bit(left, 7 - y);
                let right_pixel = nth_bit(right, 7 - y);

                let pixel = ((right_pixel as u8) << 1) | left_pixel as u8;
                let chosen_colour = self.ram[colours[pixel as usize] as usize];
                let rgb = PALETTE[chosen_colour as usize];

                let cor_x = cor_x + y as u16;
                let cor_y = cor_y + x as u16;
                let tuple = (cor_x, cor_y, Colour{r: rgb.0, g: rgb.1, b: rgb.2});
                self.frame.push(tuple);
            }
        }
        self.current_pixel += 1;
    }

    fn get_background_colour(&self, tile_row: u16, tile_column: u16) -> Vec<u16> {
        let attribute_table = self.get_base_nametable_address() + 0x03C0;
        let attribute_idx = tile_column / 4 * 8 + tile_row / 4;
        let attribute = self.ram[(attribute_table + attribute_idx) as usize];

        let palette_idx = match(tile_column % 4 / 2, tile_row % 4 / 2) {
            (0,0) => attribute,
            (1,0) => (attribute >> 2),
            (0,1) => (attribute >> 4),
            (1,1) => (attribute >> 6),
            _ => panic!("Unknown region")
        } & 0b11;
        let background: u16 = 0x3F00;
        let palette_increment: u16 = 1 + (4 * palette_idx) as u16;

        return Vec::from([background, background + palette_increment, background + palette_increment + 1, background + palette_increment + 2]);
    }

    pub fn write_oamdma(&mut self, memory: &[u8]) {
        self.oam.copy_from_slice(memory);
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
//                    warn!("Is vblank");
                    result |= 0b10_00_00_00;
                }
                self.clear_vblank();
                self.latch = 0;
                return result
            }, // PPUSTATUS
            0x2003 => self.latch, // OAMADDR
            0x2004 => self.latch, // OAMDATA
            0x2005 => self.latch, // PPUSCROLL
            0x2006 => self.latch, // PPUADDR
            0x2007 => {
                // TODO: Move to func, use in drawing
                let new_data = match self.vram_address {
                    0..=0x1FFF => {
                        let address = self.get_vram_address();
                        let result = self.ram[address];
                        self.increment_vram();
                        warn!("PPU read: {:#01X} from address {:#01X}", result, address);
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
                        self.increment_vram();
                        let result = self.ram[address as usize];
                        warn!("PPU read: {:#01X} from address {:#01X}", result, address);
                        return result
                    }
                    _ => panic!("Unknown vram address: {:#01X}", self.vram_address)
                };
                let result = self.internal_buffer;
                self.internal_buffer = new_data;
                result
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    pub fn save(&mut self, address: u16, value: u8) {
        info!("ppu save: {:#01X} at address {:#01X}", value, address);
        match address {
            0x2000 => {
                self.status = value;
            }, // PPUCTRL
            0x2001 => {}, // PPUMASK
            0x2002 => {
                self.latch = value;
            }, // PPUSTATUS
            0x2003 => {
                self.oam_address = value;
            }, // OAMADDR
            0x2004 => {
                let address = self.oam_address;
                self.oam[address as usize] = value;
                self.oam_address = (Wrapping(self.oam_address) + Wrapping(1)).0;
                self.latch = value;
            }, // OAMDATA
            0x2005 => {}, // PPUSCROLL
            0x2006 => {
                let address = combine_u8(value, self.latch);
                self.vram_address = address & 0x3FFF;
                self.latch = value;
            }, // PPUADDR
            0x2007 => {
                // TODO: Internal buffer
                let address = self.get_vram_address();
                if address >= 0x2000 {
                    self.ram[address] = value;
                    self.increment_vram();
                }
            }, // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
        self.latch = value;
    }

    fn get_vram_address(&mut self) -> usize {
        (self.vram_address & 0x3FFF) as usize
    }

    fn increment_vram(&mut self) {
        self.vram_address = (Wrapping(self.vram_address) + Wrapping(self.get_vram_increment() as u16)).0;
        self.vram_address &= 0x3FFF;
    }

    fn get_nmi_output(&mut self) -> bool {
        nth_bit(self.status, 7)
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

    fn get_base_nametable_address(&self) -> u16 {
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

    pub fn emulate(&mut self, screen: &mut Screen) {
        self.tick(screen);
        self.tick(screen);
        self.tick(screen);
    }
}