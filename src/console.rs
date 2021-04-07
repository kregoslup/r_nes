extern crate winit;

use crate::screen::Screen;
use crate::bus::Bus;
use crate::cpu::Cpu;
use crate::cartridge::CartridgeLoader;
use crate::ppu::Ppu;
use crate::util::read_file;
use std::path::Path;
use std::fs::File;
use self::winit::event_loop::EventLoop;

pub struct Console {}

impl Console {
    pub fn power(cartridge_path: &Path, logfile: &File) {
        let cartridge = CartridgeLoader::load_cartridge(read_file(&cartridge_path));
        let mut ppu = Ppu::new(cartridge.chr_rom.clone(), cartridge.nametable_mirroring);
        let mut bus = Bus::new(vec![0; 2048], ppu, cartridge);
        let mut cpu = Cpu::new(bus, None);
        let event_loop = EventLoop::new();
        let mut screen = Screen::new(&event_loop);
        cpu.emulation_loop(logfile, &mut screen, event_loop)
    }
}