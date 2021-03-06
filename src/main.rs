#[macro_use]
#[allow(warnings)]
extern crate bitflags;

use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cartridge::CartridgeLoader;
use crate::ppu::Ppu;

mod cpu;
mod op_code;
mod bus;
mod addressing;
mod util;
mod flags;
mod cartridge;
mod ppu;
mod screen;

fn main() {
    run_test()
//    run_emulation()
}

fn run_test() {
//    startup("rom/nestest.nes", Some(0xC000))
    startup("rom/nestest.nes", Some(0xC000))
}

fn run_emulation() {
    startup("rom/Super_mario_brothers.nes", None)
}

fn startup(path: &str, program_counter: Option<u16>) {
    let cartridge = CartridgeLoader::load_cartridge(read_file(Path::new(path)));
    let mut ppu = Ppu::new(cartridge.chr_rom.clone(), vec![]);
    let bus = Bus::new( vec![0; 2047], ppu, cartridge);
    let mut emulator = Cpu::new(bus, program_counter);
    emulator.emulation_loop();
}

fn read_file(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data);
    return data;
}
