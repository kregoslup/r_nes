pub struct Cartridge {
    prg_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    mapper: u8
}

impl Cartridge {
    pub fn load_cartridge(payload: Vec<u8>) -> Cartridge {
        let mapper = Cartridge::load_mapper(payload);
        let prg_rom = Cartridge::load_prg(&payload);
        let chr_rom = Cartridge::load_chr(&payload);
        return Cartridge {
            prg_rom,
            chr_rom,
            mapper: 0
        }
    }

    fn load_mapper(payload: &Vec<u8>) -> u8 {
        MAPPER_FLAG = 5;

    }

    fn load_prg(payload: &Vec<u8>) -> Vec<u8> {

    }

    fn load_chr(payload: &Vec<u8>) -> Vec<u8> {

    }
}
