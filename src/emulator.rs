use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::ppu::Ppu;
use crate::cartridge::{Cartridge, CartridgeLoader};
use std::path::Path;
use std::fs::File;
use std::io::Read;

struct Emulator {}

impl Emulator {
    pub fn emulate(path: Path) {
        let ppu = Ppu::new();
        let cartridge = CartridgeLoader::load_cartridge(read_file(&path));
        let bus = Bus::new(vec![], ppu, cartridge);
        let cpu = Cpu::new(bus);
        // cpu.emulate()
        // ppu.emulate()
        // while true {
        //     cpu.tick();
        //     ppu.tick();
        // }
    }

    fn tick(ppu: Ppu, cpu: Cpu) {
        cpu.tick();
        ppu.tick();
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
    use dirs::home_dir;

    // TODO: Needs refactor for CI
    #[test]
    #[ignore]
    fn test_read_file() {
        let mut tmp_dir = home_dir().unwrap();
        tmp_dir.push(".bash_history");
        assert_ne!(read_file(tmp_dir.as_path()).len(), 0)
    }
}