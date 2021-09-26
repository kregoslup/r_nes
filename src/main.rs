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
use crate::console::Console;
use std::fs::File;

mod cpu;
mod op_code;
mod bus;
mod addressing;
mod util;
mod flags;
mod cartridge;
mod ppu;
mod screen;
mod console;

fn main() {
    configure_logging();
    let cartridge_path = Path::new("rom/nestest.nes");
    let logfile = File::create("testing/output.txt").unwrap();
    Console::power(&cartridge_path, &logfile);
}

fn configure_logging() -> Handle {
    let stdout = ConsoleAppender::builder().build();
    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();
    log4rs::init_config(config).unwrap()
}
