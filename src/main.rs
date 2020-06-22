#[macro_use]
extern crate bitflags;

mod cpu;
mod op_code;
mod bus;
mod addressing;
mod util;
mod flags;
mod cartridge;
mod ppu;
mod emulator;

fn main() {
    println!("Hello, world!");
}
