use crate::op_code::OpCode;
use crate::addressing::AddressingMode::{IndexedIndirect, ZeroPage, Immediate, IndirectIndexed,
                                        ZeroPageIndexed, Absolute, AbsoluteIndexed, Accumulator,
                                        Indirect, Relative};
use crate::bus::Bus;
use crate::addressing::{Addressing, AddressingMode, AddressingRegistry};
use crate::util::{combine_u8, msb, lsb, nth_bit};

use std::ops::{BitOr, BitAnd, BitXor, Shl, Shr};
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

impl From<Flags> for u8 {
    fn from(flag: Flags) -> Self {
        flag.bits()
    }
}

impl From<u8> for Flags {
    fn from(item: u8) -> Self {
        Flags::from_bits_truncate(item)
    }
}

impl Cpu {
    pub fn new(bus: Bus) -> Cpu {
        Cpu {
            stack_pointer: 0xff,
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

    fn indirect_address(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        self.program_counter += 1;
        let msb = self.fetch(self.program_counter);

        let real_lsb = self.fetch(combine_u8(lsb, msb));
        let real_msb = self.fetch(combine_u8(lsb, msb) + 1);

        combine_u8(real_lsb, real_msb)
    }

    fn relative(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        self.program_counter
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
            Indirect => {
                self.indirect_address(addressing)
            },
            Relative => {
                self.relative(addressing)
            },
            // TODO: Improve logging
            _ => panic!("Cannot fetch address with given address mode")
        }
    }

    fn fetch_with_addressing_mode(&mut self, addressing: &Addressing) -> (u8, Option<u16>) {
        if addressing.mode == Accumulator {
            (self.acc, None)
        } else {
            let address = self.fetch_address(addressing);
            (self.fetch(address), Some(address))
        }
    }

    fn fetch(&mut self, address: u16) -> u8 {
        self.bus.fetch(address)
    }

    fn store(&mut self, value: u8, address: Option<u16>) {
        match address {
            None => self.acc = value,
            Some(add) => self.bus.store(value, add)
        }
    }

    fn push_flags_on_stack(&mut self) {
        self.push_on_stack(u8::from(self.status));
    }

    fn push_status_on_stack(&mut self) {
        self.push_program_counter_on_stack();
        self.push_flags_on_stack();
    }

    fn push_program_counter_on_stack(&mut self) {
        self.push_on_stack((self.program_counter >> 8) as u8);
        self.push_on_stack(self.program_counter as u8);
    }

    fn push_on_stack(&mut self, value: u8) {
        self.store(value, Some(self.stack_pointer as u16));
        self.stack_pointer -= 1;
    }

    fn read_flags_from_stack(&mut self) -> Flags {
        self.stack_pointer += 1;
        self.fetch(self.stack_pointer as u16).into()
    }

    fn read_pc_from_stack(&mut self) -> u16 {
        self.stack_pointer += 1;
        let lsb =  self.fetch(self.stack_pointer as u16);
        self.stack_pointer += 1;
        let msb =  self.fetch(self.stack_pointer as u16);
        combine_u8(lsb, msb)
    }

    // Returns cycles
    pub fn evaluate(&mut self, op_code: OpCode) -> u8 {
        println!("Evaluating op code, hex: {:#02X}, bin: {:#08b}", op_code.value, op_code.value);
        match op_code.value {
            0x00 => self.force_break(),
            0x08 => self.push_processor_status(),
            0x28 => self.pull_processor_status(),
            0x40 => self.return_from(true),
            0x60 => self.return_from(false),
            0x20 => self.jump_to_subroutine(Addressing::absolute()),
            _ => self.decode_op_code(op_code)
        }
    }

    fn decode_op_code(&mut self, op_code: OpCode) -> u8 {
        let addressing = Addressing::from_op_code(op_code.mid_op_code(), op_code.lower_op_code());
        match (op_code.upper_op_code(), op_code.mid_op_code(), op_code.lower_op_code()) {
            (upper_op_code, 0b100, 0b000) => self.branch(addressing, upper_op_code),
            (0b000, _, 0b1) => self.bitwise_instruction(addressing, BitOr::bitor, false),
            (0b001, _, 0b1) => self.bitwise_instruction(addressing, BitAnd::bitand, true),
            (0b010, _, 0b1) => self.bitwise_instruction(addressing, BitXor::bitxor, true),
            (0b011, _, 0b1) => self.add_with_carry(addressing),
            (0b111, _, 0b1) => self.sub_with_borrow(addressing),
            (0b110, _, 0b1) => self.compare(addressing, self.acc),
            (0b100, _, 0b1) => self.store_accumulator(addressing),
            (0b101, _, 0b1) => self.load_accumulator(addressing),
            (0b000, _, 0b10) => self.shift_left(addressing),
            (0b001, _, 0b10) => self.rotate_left(addressing),
            (0b010, _, 0b10) => self.logical_shift_right(addressing),
            (0b011, _, 0b10) => self.rotate_right(addressing),
            (0b100, _, 0b10) => self.store_register(addressing, self.reg_x),
            (0b101, _, 0b10) => self.load_register(addressing, AddressingRegistry::X),
            (0b110, _, 0b10) => self.offset_by_one(addressing, false),
            (0b111, _, 0b10) => self.offset_by_one(addressing, true),
            (0b001, _, 0b00) => self.bit_test(addressing),
            (0b010, _, 0b00) => self.jump(Addressing::absolute()),
            (0b011, _, 0b00) => self.jump(Addressing::indirect()),
            (0b100, _, 0b00) => self.store_register(addressing, self.reg_y),
            (0b101, _, 0b00) => self.load_register(addressing, AddressingRegistry::Y),
            (0b110, _, 0b00) => self.compare(addressing, self.reg_y),
            (0b111, _, 0b00) => self.compare(addressing, self.reg_x),
            _ => panic!("Unknown op code")
        }
    }

    fn push_processor_status(&mut self) -> u8 {
        let cycles = 3;
        self.push_flags_on_stack();
        cycles
    }

    fn pull_processor_status(&mut self) -> u8 {
        let cycles = 3;
        self.status = self.read_flags_from_stack();
        cycles
    }

    fn branch(&mut self, addressing: Addressing, branch_instruction: u8) -> u8 {
        println!("Branching...");
        let mut cycles = 2;
        let branch_flag = self.extract_branch_flag(branch_instruction);
        let branch_equality = self.extract_branch_equality(branch_instruction);
        let (succeeded, new_page) = match branch_flag {
            0b00 => self.branch_on_flag(addressing, branch_equality, Flags::NEGATIVE),
            0b01 => self.branch_on_flag(addressing, branch_equality, Flags::OVERFLOW),
            0b10 => self.branch_on_flag(addressing, branch_equality, Flags::CARRY),
            0b11 => self.branch_on_flag(addressing, branch_equality, Flags::ZERO),
            _ => panic!("Unknown branch type")
        };
        if succeeded {
            cycles += 1;
        }
        if new_page {
            cycles += 1;
        }
        cycles
    }

    fn extract_branch_flag(&mut self, branch_instruction: u8) -> u8 {
        let higher_bit = nth_bit(branch_instruction, 1) as u8;
        let lower_bit = nth_bit(branch_instruction, 2) as u8;
        ((1 << higher_bit) as u8) | lower_bit
    }

    fn extract_branch_equality(&mut self, branch_instruction: u8) -> bool {
        nth_bit(branch_instruction, 3)
    }

    fn branch_on_flag(&mut self, addressing: Addressing, branch_equality: bool, flag: Flags) -> (bool, bool) {
        let mut succeeded = false;
        let (raw_branch_offset, _) = self.fetch_with_addressing_mode(&addressing);
        let branch_offset = raw_branch_offset as i8;
        println!("Branch offset: {}", branch_offset);
        let flag = self.status.contains(flag);
        if (branch_equality && flag) | (!branch_equality & !flag) {
            self.program_counter += branch_offset as u16;
            succeeded = true;
        };
        (succeeded, self.is_on_different_page(self.program_counter, self.program_counter + 1))
    }

    fn bit_test(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 3;
        let (to_test, _) = self.fetch_with_addressing_mode(&addressing);
        let zero = (to_test & self.acc) == 0;
        let negative = msb(to_test) == 1;
        let overflow = nth_bit(to_test, 6);

        self.status.set_flag(zero, Flags::ZERO);
        self.status.set_flag(negative, Flags::NEGATIVE);
        self.status.set_flag(overflow, Flags::OVERFLOW);

        if addressing.mode == Absolute {
            cycles += 1;
        }
        cycles
    }

    fn jump(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 3;
        self.program_counter = self.fetch_address(&addressing);

        if addressing.mode == Indirect {
            cycles += 2;
        }
        cycles
    }

    fn jump_to_subroutine(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 6;
        let new_pc = self.fetch_address(&addressing);
        self.program_counter -= 3;
        self.push_program_counter_on_stack();
        self.program_counter = new_pc;
        cycles
    }

    fn return_from(&mut self, read_flags: bool) -> u8 {
        let mut cycles = 6;
        if read_flags {
            self.status = self.read_flags_from_stack();
        }
        let mut new_pc = self.read_pc_from_stack();
        if !read_flags {
            new_pc -= 1;
        }
        self.program_counter = new_pc;
        cycles
    }

    fn store_register(&mut self, addressing: Addressing, target: u8) -> u8 {
        let mut cycles = 3;
        let fixed_addressing = addressing.to_register_specific_addressing();
        let address = self.fetch_address(&fixed_addressing);
        self.store(target, Some(address));

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn offset_by_one(&mut self, addressing: Addressing, increment: bool) -> u8 {
        let mut cycles = 2;
        let (mut value, address) = self.fetch_with_addressing_mode(&addressing);
        if increment {
            value = (Wrapping(value) + Wrapping(1)).0;
        } else {
            value = (Wrapping(value) - Wrapping(1)).0;
        }
        self.store(value, address);
        self.set_negative(value as u16);
        self.set_zero(value as u16);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn load_register(&mut self, addressing: Addressing, target: AddressingRegistry) -> u8 {
        let mut cycles = 2;
        let fixed_addressing = addressing.to_register_specific_addressing();
        let (value, _) = self.fetch_with_addressing_mode(&fixed_addressing);
        self.set_negative(value as u16);
        self.set_zero(value as u16);
        if target == AddressingRegistry::X {
            self.reg_x = value;
        } else {
            self.reg_y = value;
        }
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);
        cycles
    }

    fn shift_left(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, address) = self.fetch_with_addressing_mode(&addressing);
        let (result, carry) = value.overflowing_mul(2);
        self.store(result, address);
        self.set_negative(result as u16);
        self.set_zero(result as u16);
        self.status.set_flag(carry, Flags::CARRY);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn logical_shift_right(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, address) = self.fetch_with_addressing_mode(&addressing);
        let carry = lsb(value) == 1;
        let (result, _) = value.overflowing_div(2);
        self.store(result, address);
        self.set_negative(result as u16);
        self.set_zero(result as u16);
        self.status.set_flag(carry, Flags::CARRY);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn rotate_left(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, address) = self.fetch_with_addressing_mode(&addressing);
        let carry = msb(value) == 1;
        let mut result = value.shl(1);
        if self.status.contains(Flags::CARRY) {
            result = result | Flags::CARRY.bits();
        }
        self.store(result, address);
        self.set_rotation_flags(result, carry);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn rotate_right(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, address) = self.fetch_with_addressing_mode(&addressing);
        let carry = lsb(value) == 1;
        let mut result = value.shr(1);
        if self.status.contains(Flags::CARRY) {
            result = result | (Flags::CARRY.bits() << 7);
        }
        self.store(result, address);
        self.set_rotation_flags(result, carry);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn set_rotation_flags(&mut self, result: u8, carry: bool) {
        self.set_negative(result as u16);
        self.set_zero(result as u16);
        self.status.set_flag(carry, Flags::CARRY);
    }

    fn force_break(&mut self) -> u8 {
        let cycles = 7;
        self.program_counter += 1;
        self.status.set_flag(true, Flags::BRK);
        self.status.set_flag(true, Flags::IRQ_DIS);
        self.push_status_on_stack();

        let msb = self.fetch(0xFFFE);
        let lsb = self.fetch(0xFFFF);
        self.program_counter = combine_u8(lsb, msb);
        self.status.set_flag(false, Flags::BRK);
        cycles
    }

    fn store_accumulator(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let address = self.fetch_address(&addressing);
        self.store(self.acc, Some(address));
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn load_accumulator(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
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

    fn compare(&mut self, addressing: Addressing, target: u8) -> u8 {
        let mut cycles = 2;
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
        let mut result = (Wrapping(target) - Wrapping(value)).0;
        let carry = value <= target;
        let zero = result == 0;
        let negative = msb(result) == 1;
        self.status.set_flag(negative, Flags::NEGATIVE);
        self.status.set_flag(carry, Flags::CARRY);
        self.status.set_flag(zero, Flags::ZERO);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        cycles
    }

    fn add_with_carry(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
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
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
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
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
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

    fn is_on_different_page(&self, lhs: u16, rhs: u16) -> bool {
        let lhs_page = lhs % 255;
        let rhs_page = rhs % 255;
        lhs == rhs
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

    // TODO: OVERFLOW??
    #[test]
    #[ignore]
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

    // TODO: OVERFLOW??
    #[test]
    #[ignore]
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

        // cpy
        let mut cpu = create_test_cpu(vec![0xCC, 0x04, 0x00, 11]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 10;
        cpu.evaluate(OpCode::new(0xCC));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);

        //cpx
        let mut cpu = create_test_cpu(vec![0xCC, 0x04, 0x00, 11]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 10;
        cpu.evaluate(OpCode::new(0xCC));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);
    }

    #[test]
    fn test_load_accumulator() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x03, 0x05, 0x00, 180]);
        reset_cpu(&mut cpu);
        cpu.acc = 0;
        cpu.evaluate(OpCode::new(0xA1));
        assert_eq!(cpu.acc, 180);
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
        let mut cpu = create_test_cpu(vec![0x0E, 0x04, 0x00, 20, 5, 6, 7]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x0E));
        assert_eq!(cpu.fetch(4), 40);

//         acc
        let mut cpu = create_test_cpu(vec![0x0A]);
        reset_cpu(&mut cpu);
        cpu.acc = 250;
        cpu.evaluate(OpCode::new(0x0A));
        assert_eq!(cpu.acc, 244);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY | Flags::NEGATIVE);
    }

    #[test]
    fn test_rotate_left() {
        let mut cpu = create_test_cpu(vec![0x2E, 0x04, 0x00, 0b1000_0000]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::CARRY;
        cpu.evaluate(OpCode::new(0x2E));
        assert_eq!(cpu.fetch(4), 0b0000_0001);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);
    }

    #[test]
    fn test_logical_shift_right() {
        let mut cpu = create_test_cpu(vec![0x4E, 0x04, 0x00, 0b1000_0001]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x4E));
        assert_eq!(cpu.fetch(4), 0b0100_0000);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);
    }

    #[test]
    fn test_rotate_right() {
        let mut cpu = create_test_cpu(vec![0x6E, 0x04, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::CARRY;
        cpu.evaluate(OpCode::new(0x6E));
        assert_eq!(cpu.fetch(4), 0b1000_0000);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);
    }

    #[test]
    fn test_store_register_x() {
        let mut cpu = create_test_cpu(vec![0x96, 0x04, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 10;
        cpu.evaluate(OpCode::new(0x96));
        assert_eq!(cpu.fetch(4), 10);
    }

    #[test]
    fn test_load_register_x() {
        let mut cpu = create_test_cpu(vec![0xB6, 0x04, 0x00, 150]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 10;
        cpu.evaluate(OpCode::new(0xB6));
        assert_eq!(cpu.reg_x, 150);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    //TODO: Add test for ZP, X
    #[test]
    fn test_store_register_y() {
        let mut cpu = create_test_cpu(vec![0x8C, 0x04, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 10;
        cpu.evaluate(OpCode::new(0x8C));
        assert_eq!(cpu.fetch(4), 10);
    }

    //TODO: Add test for ZP, X
    #[test]
    fn test_load_register_y() {
        let mut cpu = create_test_cpu(vec![0xAC, 0x04, 0x00, 150]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 10;
        cpu.evaluate(OpCode::new(0xAC));
        assert_eq!(cpu.reg_y, 150);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_increment() {
        let mut cpu = create_test_cpu(vec![0xEE, 0x04, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0xFE));
        assert_eq!(cpu.fetch(4), 1);
    }

    #[test]
    fn test_decrement() {
        let mut cpu = create_test_cpu(vec![0xCE, 0x04, 0x00, 1]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0xCE));
        assert_eq!(cpu.fetch(4), 0);
    }

    #[test]
    fn test_bit_test() {
        let mut cpu = create_test_cpu(vec![0x2C, 0x04, 0x00, 0b1100_0000]);
        reset_cpu(&mut cpu);
        cpu.acc == 0;
        cpu.evaluate(OpCode::new(0x2C));
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER | Flags::OVERFLOW | Flags::ZERO)
    }

    #[test]
    fn test_jmp() {
        let mut cpu = create_test_cpu(vec![0x4C, 0x04, 0x00]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x4C));
        assert_eq!(cpu.program_counter, 4)
    }

    #[test]
    fn test_jmp_indirect() {
        let mut cpu = create_test_cpu(vec![0x6C, 0x04, 0x00, 20, 0]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x6C));
        assert_eq!(cpu.program_counter, 20)
    }

    #[test]
    fn test_branch_on_flag() {
        let mut cpu = create_test_cpu(vec![0x30, 0x04, 0x00, 20, 0]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::NEGATIVE;
        cpu.evaluate(OpCode::new(0x30));
        assert_eq!(cpu.program_counter, 6)
    }

    #[test]
    fn test_break() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        // TODO: Fix off by one in tests
        memory[0xFFFD] = 0x44;
        memory[0xFFFE] = 0x66;
        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.program_counter = 0x1234;
        cpu.evaluate(OpCode::new(0x00));
        assert_eq!(cpu.program_counter, 0x4466);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::IRQ_DIS);

        let stored_flags: Flags = cpu.fetch((cpu.stack_pointer + 1) as u16).into();
        assert_eq!(
            stored_flags,
            Flags::IRQ_DIS | Flags::BRK | Flags::PLACEHOLDER
        );

        let lsb_stored_program_counter =  cpu.fetch((cpu.stack_pointer + 2) as u16);
        let msb_stored_program_counter =  cpu.fetch((cpu.stack_pointer + 3) as u16);
        assert_eq!(combine_u8(lsb_stored_program_counter, msb_stored_program_counter), 0x1235)
    }

    #[test]
    fn test_jsr() {
        let len = 0xFFFF;
        let mut memory = vec![0; len];
        memory[0] = 0x60;
        memory[1] = 0x20;
        memory[3] = 0x03;

        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.program_counter = 3;
        cpu.evaluate(OpCode::new(0x20));
        assert_eq!(cpu.program_counter, 0x03);

        let lsb_stored_program_counter =  cpu.fetch((cpu.stack_pointer + 1) as u16);
        let msb_stored_program_counter =  cpu.fetch((cpu.stack_pointer + 2) as u16);
        assert_eq!(combine_u8(lsb_stored_program_counter, msb_stored_program_counter), 2)
    }

    #[test]
    fn test_rti() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        let flags_on_stack = Flags::NEGATIVE | Flags::PLACEHOLDER | Flags::OVERFLOW;
        memory[0] = 0x40;
        memory[0x00FE] = 0x44;
        memory[0x00FD] = 0x66;
        memory[0x00FC] = flags_on_stack.bits();

        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.stack_pointer -= 3;
        cpu.program_counter = 2;

        cpu.evaluate(OpCode::new(0x40));
        assert_eq!(cpu.program_counter, 0x4466);
        assert_eq!(cpu.status, flags_on_stack)
    }

    #[test]
    fn test_rts() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        let flags_on_stack = Flags::NEGATIVE | Flags::PLACEHOLDER | Flags::OVERFLOW;
        memory[0] = 0x40;
        memory[0x00FE] = 0x44;
        memory[0x00FD] = 0x66;

        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.stack_pointer -= 2;
        cpu.program_counter = 2;

        cpu.evaluate(OpCode::new(0x60));
        assert_eq!(cpu.program_counter, 0x4465);
    }

    #[test]
    fn test_php() {
        let len = 0x10000;
        let mut memory = vec![0; len];

        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        let current_flags = Flags::NEGATIVE | Flags::PLACEHOLDER | Flags::OVERFLOW;
        cpu.status = current_flags;
        cpu.evaluate(OpCode::new(0x08));

        let stored_flags: Flags = cpu.fetch((cpu.stack_pointer + 1) as u16).into();
        assert_eq!(stored_flags, current_flags)
    }

    #[test]
    fn test_plp() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        let stored_flags = Flags::NEGATIVE | Flags::PLACEHOLDER | Flags::OVERFLOW;
        memory[0x00FE] = stored_flags.bits();

        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.stack_pointer -= 1;

        cpu.evaluate(OpCode::new(0x28));

        assert_eq!(cpu.status, stored_flags)
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
