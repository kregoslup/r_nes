use crate::op_code::OpCode;
use crate::addressing::AddressingMode::{IndexedIndirect, ZeroPage, Immediate, IndirectIndexed, ZeroPageIndexed, Absolute, AbsoluteIndexed};
use crate::bus::Bus;
use crate::addressing::{Addressing, AddressingMode, AddressingRegistry};
use std::ops::{BitOr, BitAnd, BitXor};
use std::ops::Add;

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
    pub struct Flags: u8 {
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
            program_counter: 1,
            acc: 0,
            reg_x: 0,
            reg_y: 0,
            cycles: 0,
            status: Flags::PLACEHOLDER,
            bus
        }
    }

    fn set_flag(&mut self, acc: u8, flag: Flags) {
        match flag {
            Flags::NEGATIVE => {
                if msb(acc) != 0 {
                    self.status.toggle(Flags::NEGATIVE)
                }
            },
            Flags::ZERO => {
                if acc == 0 {
                    self.status.insert(Flags::ZERO)
                } else {
                    self.status.remove(Flags::ZERO)
                }
            }
            _ => panic!("Trying to set not supported flag")
        }
    }

    fn fetch_indexed_indirect(&mut self) -> u8 {
        self.program_counter += 1;
        let indirect_address = self.fetch(self.program_counter + (self.reg_x as u16)) as u16;
        println!("Fetching indexed indirect address {:#01X}", indirect_address);
        let lsb = self.fetch(indirect_address);
        let msb = self.fetch(indirect_address + 1);
        let address = combine_u8(lsb, msb);
        println!("Fetching address {:#01X}", address);
        self.fetch(address)
    }

    fn fetch_zero_page(&mut self) -> u8 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        let address = lsb as u16;
        self.fetch(address)
    }

    fn fetch_immediate(&mut self) -> u8 {
        self.program_counter += 1;
        self.fetch(self.program_counter)
    }

    fn fetch_absolute(&mut self) -> u8 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        self.program_counter += 1;
        let msb = self.fetch(self.program_counter);
        self.fetch(combine_u8(lsb, msb))
    }

    fn fetch_indirect_indexed(&mut self) -> u8 {
        self.program_counter += 1;
        let indirect_address = (self.fetch(self.program_counter) as u16) + (self.reg_y as u16);
        let lsb = self.fetch(indirect_address);
        let msb = self.fetch(indirect_address + 1);
        let address = combine_u8(lsb, msb);
        println!("Fetching address {:#01X}", address);
        self.fetch(address)
    }

    fn fetch_indexed(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        let base = self.fetch(self.program_counter);
        let to_add = match addressing.register {
            Some(AddressingRegistry::X) => self.reg_x,
            Some(AddressingRegistry::Y) => self.reg_y,
            None => panic!("Addressing registry has to be filled")
        };
        (base as u16) + (to_add as u16)
    }

    fn fetch_zero_page_indexed(&mut self, addressing: &Addressing) -> u8 {
        let mut address = self.fetch_indexed(addressing) % 256;
        self.fetch(address)
    }

    fn fetch_absolute_indexed(&mut self, addressing: &Addressing) -> u8 {
        let address = self.fetch_indexed(addressing);
        self.fetch(address)
    }

    fn fetch_with_addressing_mode(&mut self, addressing: &Addressing) -> u8 {
        match addressing.mode {
            IndexedIndirect => {
                self.fetch_indexed_indirect()
            },
            ZeroPage => {
                self.fetch_zero_page()
            },
            Immediate => {
                self.fetch_immediate()
            },
            Absolute => {
                self.fetch_absolute()
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

    fn fetch(&mut self, address: u16) -> u8 {
        self.bus.fetch(address)
    }

    fn extract_addressing(&mut self, mid_op_code: u8, lower_op_code: u8) -> Addressing {
        match mid_op_code {
            0b0 => Addressing::indexed_indirect(),
            0b001 => Addressing::zero_page(),
            0b010 => Addressing::immediate(),
            0b011 => Addressing::absolute(),
            0b100 => Addressing::indirect_indexed(),
            0b101 => Addressing::zero_page_indexed(Some(AddressingRegistry::X), false),
            0b110 => Addressing::absolute_indexed(Some(AddressingRegistry::Y), true),
            0b111 => Addressing::absolute_indexed(Some(AddressingRegistry::X), false),
            _ => panic!("Unknown addressing type")
        }
    }

    // Returns cycles
    pub fn evaluate(&mut self, op_code: OpCode) -> u8 {
        let addressing = self.extract_addressing(op_code.mid_op_code(), op_code.lower_op_code());
        match (op_code.upper_op_code(), op_code.mid_op_code(), op_code.lower_op_code()) {
            (0b000, _, 0b1) => self.bitwise_instruction(addressing, BitOr::bitor, false),
            (0b001, _, 0b1) => self.bitwise_instruction(addressing, BitAnd::bitand, true),
            (0b010, _, 0b1) => self.bitwise_instruction(addressing, BitXor::bitxor, true),
            (0b011, _, 0b1) => self.arithmetic_instruction(addressing, Add::add, true),
            _ => panic!("Unknown op code")
        }
    }

    fn force_break(&mut self) -> u8 {
        println!("BRK op code");
        let cycles = 7;
        self.program_counter += 1;
        cycles
    }

    fn arithmetic_instruction(&mut self, addressing: Addressing, operation: fn(u8, u8) -> u8, additional_cycle: bool) -> u8 {
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        self.acc = operation(self.acc, value);
        if (self.page_boundary_crossed(self.acc)) & additional_cycle {
            cycles += 1;
        }
        if (addressing.add_cycles) & (self.page_boundary_crossed(self.acc)) {
            cycles += 1;
        }
        self.set_flag(self.acc, Flags::ZERO);
        self.set_flag(self.acc, Flags::NEGATIVE);
        self.set_flag(self.acc, Flags::CARRY);
        self.set_flag(self.acc, Flags::OVERFLOW);

        cycles
    }

    fn bitwise_instruction(&mut self, addressing: Addressing, operation: fn(u8, u8) -> u8, additional_cycle: bool) -> u8 {
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        self.acc = operation(self.acc, value);
        if (self.page_boundary_crossed(self.acc)) & additional_cycle {
            cycles += 1;
        }
        if (addressing.add_cycles) & (self.page_boundary_crossed(self.acc)) {
            cycles += 1;
        }
        self.set_flag(self.acc, Flags::ZERO);
        self.set_flag(self.acc, Flags::NEGATIVE);
        cycles
    }

    fn page_boundary_crossed(&self, value: u8) -> bool {
        value > 0x00FF
    }
}

fn combine_u8(lsb: u8, msb: u8) -> u16 {
    ((msb << 7) as u16).bitor(lsb as u16)
}

fn msb(value: u8) -> u8 {
    value >> 7
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu(input: Vec<u8>) -> Cpu {
        let mut bus = Bus::new(input);
        Cpu::new(bus)
    }

    #[test]
    fn test_bit_or() {
        let mut cpu = create_test_cpu(vec![0x01, 0x03, 0x05, 0x00, 0b1111_1111]);
        cpu.acc = 0;
        cpu.reg_x = 0;
        cpu.evaluate(OpCode::new(0x01));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_and() {
        let mut cpu = create_test_cpu(vec![0x01, 0x03, 0x05, 0x00, 0b1111_1111]);
        cpu.acc = 0;
        cpu.reg_x = 0;
        cpu.evaluate(OpCode::new(0x21));
        assert_eq!(cpu.acc, 0b0000_0000);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_xor() {
        let mut cpu = create_test_cpu(vec![0x01, 0x03, 0x05, 0x00, 0b1111_1111]);
        cpu.acc = 0b1111_1111;
        cpu.reg_x = 0;
        cpu.evaluate(OpCode::new(0x41));
        assert_eq!(cpu.acc, 0b0000_0000);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_indexed_indirect() {
        let mut cpu = create_test_cpu(vec![0x01, 0x03, 0x05, 0x00, 0b1111_1111]);
        cpu.acc = 0;
        cpu.reg_x = 0;
        cpu.evaluate(OpCode::new(0x01));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_zero_page() {

    }

    #[test]
    fn test_immediate() {

    }

    #[test]
    fn test_absolute() {

    }

    #[test]
    fn test_indirect_indexed() {

    }

    #[test]
    fn test_zero_page_indexed() {

    }

    #[test]
    fn test_absolute_indexed() {

    }
}