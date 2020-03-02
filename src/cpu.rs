use crate::op_code::OpCode;
use crate::addressing::AddressingMode::{IndexedIndirect, ZeroPage, Immediate, IndirectIndexed, ZeroPageIndexed, Absolute, AbsoluteIndexed, Accumulator};
use crate::bus::Bus;
use crate::addressing::{Addressing, AddressingMode, AddressingRegistry};
use crate::util::{combine_u8, msb};

use std::ops::{BitOr, BitAnd, BitXor};
use bitflags::_core::num::Wrapping;
use std::u8;

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

impl Default for Flags {
    fn default() -> Flags {
        Flags::PLACEHOLDER
    }
}

impl Flags {
    pub fn set_flag(&mut self, value: bool, flag: Flags) {
        if value {
            self.insert(flag)
        } else {
            self.remove(flag)
        }
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
            status: Default::default(),
            bus
        }
    }

    fn set_carry(&mut self, result: u16) {
        if result > 0xFF {
            self.status.insert(Flags::CARRY)
        } else {
            self.status.remove(Flags::CARRY)
        }
    }

    fn set_zero(&mut self, result: u16) {
        if result == 0x00 {
            self.status.insert(Flags::ZERO)
        } else {
            self.status.remove(Flags::ZERO)
        }
    }

    fn set_negative(&mut self, result: u16) {
        if msb(result as u8) != 0 {
            self.status.insert(Flags::NEGATIVE)
        } else {
            self.status.remove(Flags::NEGATIVE)
        }
    }

    fn set_overflow(&mut self, lhs: u8, rhs: u8, result: u16) {
        if self.overflow_occurred(lhs, rhs, (result as u8)) {
            self.status.insert(Flags::OVERFLOW)
        } else {
            self.status.remove(Flags::OVERFLOW)
        }
    }

    fn set_borrow(&mut self, result: u16) {
        if result < 0x100 {
            self.status.insert(Flags::CARRY)
        } else {
            self.status.remove(Flags::CARRY)
        }
    }

    fn indexed_indirect_address(&mut self) -> u16 {
        self.program_counter += 1;
        let indirect_address = self.fetch(self.program_counter + (self.reg_x as u16)) as u16;
        println!("Fetching indexed indirect address {:#01X}", indirect_address);
        let lsb = self.fetch(indirect_address);
        let msb = self.fetch(indirect_address + 1);
        let address = combine_u8(lsb, msb);
        println!("Fetching address {:#01X}", address);
        address
    }

    fn zero_page_address(&mut self) -> u16 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        let address = lsb as u16;
        address
    }

    fn immediate_address(&mut self) -> u16 {
        self.program_counter += 1;
        self.program_counter
    }

    fn absolute_address(&mut self) -> u16 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        self.program_counter += 1;
        let msb = self.fetch(self.program_counter);
        println!("Fetching absolute address {}", combine_u8(lsb, msb));
        combine_u8(lsb, msb)
    }

    fn indirect_indexed_address(&mut self) -> u16 {
        self.program_counter += 1;
        let indirect_address = (self.fetch(self.program_counter) as u16);
        println!("Fetched indirect address {:#01X} at memory location {:#01X}", indirect_address, self.program_counter);
        // TODO: Add carry
        let address = indirect_address + (self.reg_y as u16);
        println!("Created address {:#01X}", address);
        address
    }

    fn indexed_address(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        let base = self.fetch(self.program_counter);
        let to_add = match addressing.register {
            Some(AddressingRegistry::X) => self.reg_x,
            Some(AddressingRegistry::Y) => self.reg_y,
            None => panic!("Addressing registry has to be filled")
        };
        println!("Fetching indexed address {:#01X} created from {:#01X} and {:#01X}", (base as u16) + (to_add as u16), base, to_add);
        (base as u16) + (to_add as u16)
    }

    fn zero_page_indexed_address(&mut self, addressing: &Addressing) -> u16 {
        self.indexed_address(addressing) % 256
    }

    fn absolute_indexed_address(&mut self, addressing: &Addressing) -> u16 {
        self.indexed_address(addressing)
    }

    fn fetch_address(&mut self, addressing: &Addressing) -> u16 {
        match addressing.mode {
            IndexedIndirect => {
                self.indexed_indirect_address()
            },
            ZeroPage => {
                self.zero_page_address()
            },
            Immediate => {
                self.immediate_address()
            },
            Absolute => {
                self.absolute_address()
            },
            IndirectIndexed => {
                self.indirect_indexed_address()
            },
            ZeroPageIndexed => {
                self.zero_page_indexed_address(addressing)
            },
            AbsoluteIndexed => {
                self.absolute_indexed_address(addressing)
            },
            // TODO: Improve logging
            _ => panic!("Cannot fetch address with given address mode")
        }
    }

    fn fetch_with_addressing_mode(&mut self, addressing: &Addressing) -> u8 {
        if addressing.mode == Accumulator {
            return self.acc
        }
        let address = self.fetch_address(addressing);
        self.fetch(address)
    }

    fn fetch(&mut self, address: u16) -> u8 {
        self.bus.fetch(address)
    }

    fn store(&mut self, value: u8, address: u16) {
        self.bus.store(value, address);
    }

    fn extract_addressing(&mut self, mid_op_code: u8, lower_op_code: u8) -> Addressing {
        println!("Extracting addressing from {:#010b}", mid_op_code);
        match (mid_op_code, lower_op_code) {
            // c == 0b10
            (0b0, 0b01) => Addressing::indexed_indirect(),
            (0b001, 0b01) => Addressing::zero_page(),
            (0b010, 0b01) => Addressing::immediate(),
            (0b011, 0b01) => Addressing::absolute(),
            (0b100, 0b01) => Addressing::indirect_indexed(),
            (0b101, 0b01) => Addressing::zero_page_indexed(Some(AddressingRegistry::X), false),
            (0b110, 0b01) => Addressing::absolute_indexed(Some(AddressingRegistry::Y), true),
            (0b111, 0b01) => Addressing::absolute_indexed(Some(AddressingRegistry::X), false),
            // c == 0b10
            (0b000, 0b10) => Addressing::immediate(),
            (0b001, 0b10) => Addressing::zero_page(),
            (0b010, 0b10) => Addressing::accumulator(),
            (0b011, 0b10) => Addressing::absolute(),
            (0b101, 0b10) => Addressing::zero_page_indexed(Some(AddressingRegistry::X), false),
            (0b111, 0b10) => Addressing::absolute_indexed(Some(AddressingRegistry::X), false),
            // c == 00
            _ => panic!("Unknown addressing type")
        }
    }

    // Returns cycles
    pub fn evaluate(&mut self, op_code: OpCode) -> u8 {
        let addressing = self.extract_addressing(op_code.mid_op_code(), op_code.lower_op_code());
        println!("Evaluating op code, hex: {:#02X}, bin: {:#08b}", op_code.value, op_code.value);
        match (op_code.upper_op_code(), op_code.mid_op_code(), op_code.lower_op_code()) {
            (0b000, _, 0b1) => self.bitwise_instruction(addressing, BitOr::bitor, false),
            (0b001, _, 0b1) => self.bitwise_instruction(addressing, BitAnd::bitand, true),
            (0b010, _, 0b1) => self.bitwise_instruction(addressing, BitXor::bitxor, true),
            (0b011, _, 0b1) => self.add_with_carry(addressing),
            (0b111, _, 0b1) => self.sub_with_borrow(addressing),
            (0b110, _, 0b1) => self.compare(addressing),
            (0b100, _, 0b1) => self.store_accumulator(addressing),
            (0b101, _, 0b1) => self.load_accumulator(addressing),
            (_, _, 0b10) => self.shift_left(addressing),
            _ => panic!("Unknown op code")
        }
    }

    fn shift_left(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        if addressing.mode == Accumulator {
            self.acc = result
        } else {
            let address = self.fetch_address(&addressing);
            println!("Storing {} at address {}", value, address);
            self.store(result, address);
        }
        let value = self.fetch_with_addressing_mode(&addressing);
        let (result, carry) = value.overflowing_mul(2);


        self.set_negative(result as u16);
        self.set_zero(result as u16);
        self.status.set_flag(carry, Flags::CARRY);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn force_break(&mut self) -> u8 {
        println!("BRK op code");
        let cycles = 7;
        self.program_counter += 1;
        cycles
    }

    fn store_accumulator(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let address = self.fetch_address(&addressing);
        self.store(self.acc, address);
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn load_accumulator(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        self.set_negative(value as u16);
        self.set_zero(value as u16);
        self.acc = value;
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);
        cycles
    }

    fn carry_arithmetic(&mut self, operation: fn(u8, u8) -> (u8, bool), lhs: u8, rhs: u8) -> (u8, bool) {
        let (intermediate_result, carry_overflow) = operation(lhs, rhs);
        let (result, overflow) = operation(intermediate_result, self.status.contains(Flags::CARRY) as u8);
        (result, overflow | carry_overflow)
    }

    fn overflow_occurred(&self, lhs: u8, rhs: u8, result: u8) -> bool {
        (((lhs.bitxor(result)) & (rhs.bitxor(result))) & 0x80) != 0
    }

    fn compare(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        let mut result = (Wrapping(self.acc) - Wrapping(value)).0;
        let carry = result <= self.acc;
        let zero = result == 0;
        let negative = result > self.acc;
        self.status.set_flag(negative, Flags::NEGATIVE);
        self.status.set_flag(carry, Flags::CARRY);
        self.status.set_flag(zero, Flags::ZERO);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);

        cycles
    }

    fn add_with_carry(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        let mut result = (self.acc as u16) + (value as u16) + (self.status.contains(Flags::CARRY) as u16);
        println!("ADC result: {:#b}", result);
        self.set_carry(result);
        self.set_zero(result);
        self.set_negative(result);
        self.set_overflow(self.acc, value, result);

        self.acc = (result % 256) as u8;
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);

        cycles
    }

    fn sub_with_borrow(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        let borrow = self.get_borrow();
        let mut result = Wrapping(self.acc as u16) - (Wrapping(value as u16) - Wrapping(self.status.contains(Flags::CARRY) as u16));
        self.set_borrow(result.0);
        self.set_zero(result.0);
        self.set_negative(result.0);
        self.set_overflow(self.acc, value, result.0);

        self.acc = result.0 as u8;
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);

        cycles
    }

    fn bitwise_instruction(&mut self, addressing: Addressing, operation: fn(u8, u8) -> u8, additional_cycle: bool) -> u8 {
        println!("Executing bitwise operation");
        let mut cycles = 2;
        let value = self.fetch_with_addressing_mode(&addressing);
        self.acc = operation(self.acc, value);
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, additional_cycle);

        self.set_zero(self.acc as u16);
        self.set_negative(self.acc as u16);

        cycles
    }

    fn count_additional_cycles(&mut self, mut cycles: u8, add_cycles: bool, additional_cycle: bool) -> u8 {
        if (self.page_boundary_crossed(self.acc)) & additional_cycle {
            cycles += 1;
        }
        if (add_cycles) & (self.page_boundary_crossed(self.acc)) {
            cycles += 1;
        }
        cycles
    }

    fn page_boundary_crossed(&self, value: u8) -> bool {
        value > 0x00FF
    }

    fn get_borrow(&mut self) -> u16 {
        (self.status.contains(Flags::CARRY) == false) as u16
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_cpu(input: Vec<u8>) -> Cpu {
        let mut bus = Bus::new(input);
        Cpu::new(bus)
    }

    fn reset_cpu(cpu: &mut Cpu) {
        cpu.cycles = 0;
        cpu.acc = 0;
        cpu.reg_x = 0;
        cpu.reg_y = 0;
        cpu.program_counter = 1;
    }

    #[test]
    fn test_bit_or() {
        let mut cpu = create_test_cpu(vec![0x01, 0x03, 0x05, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x01));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_and() {
        let mut cpu = create_test_cpu(vec![0x21, 0x03, 0x05, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x21));
        assert_eq!(cpu.acc, 0b0000_0000);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_xor() {
        let mut cpu = create_test_cpu(vec![0x41, 0x03, 0x05, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.acc = 0b1111_1111;
        cpu.evaluate(OpCode::new(0x41));
        assert_eq!(cpu.acc, 0b0000_0000);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_adc() {
        let mut cpu = create_test_cpu(vec![0x61, 0x03, 0x05, 0x00, 2]);
        reset_cpu(&mut cpu);
        cpu.acc = 3;
        cpu.evaluate(OpCode::new(0x61));
        assert_eq!(cpu.acc, 5);
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_sbc() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 2]);
        reset_cpu(&mut cpu);
        cpu.acc = 3;
        cpu.evaluate(OpCode::new(0xE1));
        assert_eq!(cpu.acc, 1);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY)
    }

    #[test]
    fn test_overflow_sub() {
        // FIXME: Somehow this doesn't work
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 176]);
        reset_cpu(&mut cpu);
        cpu.acc = 80;
        cpu.evaluate(OpCode::new(0xE1));
        assert_eq!(cpu.acc, 160);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::OVERFLOW | Flags:: NEGATIVE)
    }

    #[test]
    fn test_overflow_add() {
        let mut cpu = create_test_cpu(vec![0x61, 0x03, 0x05, 0x00, 80]);
        reset_cpu(&mut cpu);
        cpu.acc = 80;
        cpu.evaluate(OpCode::new(0x61));
        assert_eq!(cpu.acc, 160);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::OVERFLOW | Flags::NEGATIVE)
    }

    #[test]
    fn test_carry() {
        let mut cpu = create_test_cpu(vec![0x61, 0x03, 0x05, 0x00, 80]);
        reset_cpu(&mut cpu);
        cpu.acc = 208;
        cpu.evaluate(OpCode::new(0x61));
        assert_eq!(cpu.acc, 32);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY)
    }

    #[test]
    fn test_borrow() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 2]);
        reset_cpu(&mut cpu);
        cpu.acc = 1;
        cpu.evaluate(OpCode::new(0xE1));
        assert_eq!(cpu.acc, 255);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE)
    }

    #[test]
    fn test_compare() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 10]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xC1));
        assert_eq!(cpu.acc, 10);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::ZERO | Flags::CARRY);

        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 9]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xC1));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);

        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 11]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xC1));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);
    }

    #[test]
    fn test_load_accumulator() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 180]);
        reset_cpu(&mut cpu);
        cpu.acc = 0;
        cpu.evaluate(OpCode::new(0xA1));
        assert_eq!(cpu.acc, 180);
        assert_eq!(cpu.acc, 20);
    }

    #[test]
    fn test_store_accumulator() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 180]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0x81));
        assert_eq!(cpu.fetch(5), 10);
    }

    #[test]
    fn test_shift_left() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x04, 0x00, 20, 5, 6, 7]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x0E));
        assert_eq!(cpu.fetch(4), 40);

        // acc
//        let mut cpu = create_test_cpu(vec![0x0A]);
//        reset_cpu(&mut cpu);
//        cpu.acc = 250;
//        cpu.evaluate(OpCode::new(0x0A));
//        assert_eq!(cpu.acc, 20);
//        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);
    }

    #[test]
    fn test_indexed_indirect() {
        let mut cpu = create_test_cpu(vec![0x01, 0x03, 0x05, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x01));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_zero_page() {
        let mut cpu = create_test_cpu(vec![0x05, 0x03, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x05));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_immediate() {
        let mut cpu = create_test_cpu(vec![0x05, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x09));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_absolute() {
        let mut cpu = create_test_cpu(vec![0x05, 0x4, 0x0, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x0D));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_indirect_indexed() {
        let mut cpu = create_test_cpu(vec![0x05, 0x0, 0b1111_1111, 0x0]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 3;
        cpu.evaluate(OpCode::new(0x11));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_zero_page_indexed() {
        let mut cpu = create_test_cpu(vec![0x05, 0x0, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 3;
        cpu.evaluate(OpCode::new(0x15));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_absolute_indexed() {
        let mut cpu = create_test_cpu(vec![0x05, 0x4, 0x0, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x19));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }
}
