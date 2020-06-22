static PPU_ADDRESSABLE_RANGE: u16 = 0x3FF;

#[derive(Debug)]
pub struct Ppu {
    pattern_table: Vec<u8>, // CHR_ROM - sprites
    name_table: Vec<u8> // VRAM, - layout of background
    // palette? - colours
}

// TODO: implement
impl Ppu {

    pub fn new() -> Ppu {
        return Ppu {
            pattern_table: vec![],
            name_table: vec![]
        }
    }

    pub fn tick() {
        unimplemented!();
    }

    pub fn fetch(&mut self, address: u16) -> u8 {
        match address {
            0x2000 => unimplemented!(), // PPUCTRL
            0x2001 => unimplemented!(), // PPUMASK
            0x2002 => unimplemented!(), // PPUSTATUS
            0x2003 => unimplemented!(), // OAMADDR
            0x2004 => unimplemented!(), // OAMDATA
            0x2005 => unimplemented!(), // PPUSCROLL
            0x2006 => unimplemented!(), // PPUADDR
            0x2007 => unimplemented!(), // PPUDATA
            _ => panic!("Ppu port not implemented")
        }
    }

    pub fn save(&mut self, address: u16, value: u8) {
        match address {
            0x2000 => unimplemented!(), // PPUCTRL
            0x2001 => unimplemented!(), // PPUMASK
            0x2002 => unimplemented!(), // PPUSTATUS
            0x2003 => unimplemented!(), // OAMADDR
            0x2004 => unimplemented!(), // OAMDATA
            0x2005 => unimplemented!(), // PPUSCROLL
            0x2006 => unimplemented!(), // PPUADDR
            0x2007 => unimplemented!(), // PPUDATA
            _ => panic!("Ppu port not implemented")
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

    fn get_tile(&mut self, address: u16) -> Vec<u8> {
        let left_plane: [u8; 16] = self.get_left();
        let right_plane: [u8; 16] = self.get_right();
        return left_plane.iter().zip(&right_plane).map(|a, b| a + b).collect();
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