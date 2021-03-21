#[macro_use]
#[allow(warnings)]
extern crate bitflags;

use std::path::Path;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::Handle;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cartridge::CartridgeLoader;
use crate::ppu::Ppu;
use crate::util::read_file;

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
    configure_logging();
//    run_test()
    run_emulation()
}

fn run_test() {
//    startup("rom/nestest.nes", Some(0xC000))
    startup("rom/official_only.nes", Some(0x0000))
}

fn run_emulation() {
    startup("rom/Donkey Kong (World) (Rev A).nes", None)
}

fn startup(path: &str, program_counter: Option<u16>) {
    // TODO: Setup this properly
    let cartridge = CartridgeLoader::load_cartridge(read_file(Path::new(path)));
    let mut ppu = Ppu::new(cartridge.chr_rom.clone(), cartridge.nametable_mirroring);
    let bus = Bus::new( vec![0; 2048], ppu, cartridge);
    let mut emulator = Cpu::new(bus, program_counter);
    emulator.emulation_loop();
}

fn configure_logging() -> Handle {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    log4rs::init_config(config).unwrap()
}
