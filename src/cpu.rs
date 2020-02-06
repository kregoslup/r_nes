use crate::op_code::OpCode;
use crate::addressing::AddressingMode::{IndexedIndirect, ZeroPage, Immediate, IndirectIndexed, ZeroPageIndexed, Absolute, AbsoluteIndexed};
use crate::bus::Bus;
use crate::addressing::{Addressing, AddressingMode, AddressingRegistry};
use std::ops::BitOr;

#[derive(Debug)]
struct Cpu {
    stack_pointer: u8,
    program_counter: u16,
    acc: u8,
    reg_x: u8,
    reg_y: u8,
    status: Flags,
    cycles: u8,
    bus: Bus
}

bitflags! {
    struct Flags: u8 {
        const NEGATIVE = 0b1000_0000;
        const OVERFLOW = 0b0100_0000;
        const PLACEHOLDER = 0b0010_0000;
        const BRK = 0b0001_0000;
        const DECIMAL = 0b0000_1000;
        const IRQ_DIS = 0b0000_0100;
        const ZERO = 0b0000_0010;
        const CARRY = 0b0000_0001;
    }
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        Cpu {
            stack_pointer: 0,
            program_counter: 0,
            acc: 0,
            reg_x: 0,
            reg_y: 0,
            cycles: 0,
            status: Flags::PLACEHOLDER,
            bus
        }
    }

    fn fetch_indexed_indirect_argument(&mut self) -> u8 {
        self.cycles += 4;
        self.program_counter += 1;
        let indirect_address = self.fetch(self.program_counter + (self.reg_x as u16)) as u16;
        println!("Fetching indexed indirect address {:#01X}", indirect_address);
        let lsb = self.fetch(indirect_address);
        let msb = self.fetch(indirect_address + 1);
        let address = combine_u8(lsb, msb);
        println!("Fetching address {:#01X}", address);
        self.fetch(address)
    }

    fn fetch_zero_page_argument(&mut self) -> u8 {
        self.cycles += 1;
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        let address = lsb as u16;
        self.fetch(address)
    }

    fn fetch_immediate_argument(&mut self) -> u8 {
        self.program_counter += 1;
        self.fetch(self.program_counter)
    }

    fn fetch_absolute_argument(&mut self) -> u8 {
        self.cycles += 2;
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        self.program_counter += 1;
        let msb = self.fetch(self.program_counter);
        self.fetch(combine_u8(lsb, msb))
    }

    fn fetch_indirect_indexed(&mut self) -> u8 {
        self.cycles += 2;
        self.program_counter += 1;
        let indirect_address = (self.fetch(self.program_counter) as u16) + (self.reg_y as u16);
        let lsb = self.fetch(indirect_address);
        let msb = self.fetch(indirect_address + 1);
        let address = combine_u8(lsb, msb);
        println!("Fetching address {:#01X}", address);
        self.fetch(address)
    }

    fn

    fn fetch_zero_page_indexed(&mut self, addressing: &Addressing) -> u8 {
        self.cycles += 1;
        return 2;
    }

    fn fetch_with_addressing_mode(&mut self, addressing: &Addressing) -> u8 {
        match addressing.mode {
            IndexedIndirect => {
                self.fetch_indexed_indirect_argument()
            },
            ZeroPage => {
                self.fetch_zero_page_argument()
            },
            Immediate => {
                self.fetch_immediate_argument()
            },
            Absolute => {
                self.fetch_absolute_argument()
            },
            IndirectIndexed => {
                self.fetch_indirect_indexed()
            },
            ZeroPageIndexed => {
                self.fetch_zero_page_indexed(addressing)
            },
            AbsoluteIndexed => {
                self.fetch_absolute_indexed(addressing)
            }
        }
    }

    fn fetch(&self, address: u16) -> u8 {
        self.bus.fetch(address)
    }

    // Returns cycles
    pub fn evaluate(&mut self, op_code: OpCode) -> u8 {
        match (op_code.msb(), op_code.lsb()) {
            // BRK
            (0x0, 0x0) => self.force_break(),
            (0x0, 0x1) => self.or(Addressing::indexed_indirect()),
            (0x0, 0x5) => self.or(Addressing::zero_page()),
            (0x0, 0x9) => self.or(Addressing::immediate()),
            (0x0, 0xD) => self.or(Addressing::absolute()),
            (0x1, 0x1) => self.or(Addressing::indirect_indexed()),
            (0x1, 0x5) => self.or(Addressing::zero_page_indexed(Some(AddressingRegistry::X), false)),
            (0x1, 0x9) => self.or(Addressing::absolute_indexed(Some(AddressingRegistry::Y), true)),
            (0x1, 0xD) => self.or(Addressing::absolute_indexed(Some(AddressingRegistry::X), true)),
            _ => panic!("Unknown op code")
        }
    }

    fn force_break(&mut self) -> u8 {
        println!("BRK op code");
        let cycles = 7;
        self.program_counter += 1;
        cycles
    }

    fn or(&mut self, addressing: Addressing) -> u8 {
        println!("OR opcode");
        let cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        self.acc = self.acc | value;
        if (addressing.add_cycles) & (self.acc > 0x00FF) {
            cycles += 1;
        }
        cycles
    }
}

fn combine_u8(lsb: u8, msb: u8) -> u16 {
    ((msb << 7) as u16).bitor(lsb as u16)
}