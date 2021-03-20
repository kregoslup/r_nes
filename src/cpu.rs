// TODO: Disable warnings
use crate::op_code::OpCode;
use crate::addressing::AddressingMode::{IndexedIndirect, ZeroPage, Immediate, IndirectIndexed, ZeroPageIndexed, Absolute, AbsoluteIndexed, Accumulator, Indirect, Relative, Implied};
use crate::bus::Bus;
use crate::addressing::{Addressing, AddressingMode, AddressingRegistry};
use crate::util::{combine_u8, msb, lsb, nth_bit};
use crate::flags::Flags;

use std::ops::{BitOr, BitAnd, BitXor, Shl, Shr};
use bitflags::_core::num::Wrapping;
use std::{u8, fmt};
use std::borrow::Borrow;
use bitflags::_core::fmt::{Formatter, Error};
use crate::cartridge::{CartridgeLoader, Cartridge};
use std::path::Path;
use std::fs::File;
use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;
use std::fmt::UpperHex;

pub struct Cpu {
    stack_pointer: u8,
    program_counter: u16,
    acc: u8,
    reg_x: u8,
    reg_y: u8,
    status: Flags,
    cycles: u8,
    bus: Bus,
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_struct("Cpu")
            .field("stack_pointer_hex", &format_args!("{:#01X}", self.stack_pointer))
            .field("program_counter_hex", &format_args!("{:#01X}", self.program_counter))
            .field("acc", &format_args!("{:#01X}", self.acc))
            .field("reg_x", &format_args!("{:#01X}", self.reg_x))
            .field("reg_y", &format_args!("{:#01X}", self.reg_y))
            .field("status", &format_args!("{:?}", self.status))
            .field("status", &format_args!("{:#01X}", self.status))
            .field("cycles", &self.cycles)
            .finish()
    }
}

impl Cpu {
    pub fn new(bus: Bus, program_counter: Option<u16>) -> Cpu {
        let mut cpu = Cpu {
            stack_pointer: 0xfd,
            program_counter: 0,
            acc: 0,
            reg_x: 0,
            reg_y: 0,
            cycles: 0,
            status: Default::default(),
            bus
        };
        match program_counter { // TODO: Implement reset vector handling
            Some(x) => cpu.program_counter = x,
            None          => cpu.startup()
        }
        cpu
    }

    pub fn emulation_loop(&mut self) {
        let mut counter = 0;
        let path = "testing/output.txt";
        let mut output = File::create(path).unwrap();
        loop { // TODO: Turning off, exiting, etc
            self.emulate(&output);
            self.bus.emulate();
        }
    }

    fn startup(&mut self) { // TODO: Reset vector?
        let msb = self.fetch(0xFFFD);
        let lsb = self.fetch(0xFFFC);
        self.program_counter = combine_u8(lsb, msb);
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

    fn set_overflow(&mut self, lhs: u8, rhs: u8, result: u16, add: bool) {
        if self.overflow_occurred(lhs, rhs, (result as u8), add) {
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
        let op_code_arg = self.fetch(self.program_counter);
        let lsb = self.fetch(((Wrapping(op_code_arg) + Wrapping(self.reg_x)).0 & 0xFF) as u16);
        let msb = self.fetch(((Wrapping(op_code_arg) + Wrapping(self.reg_x) + Wrapping(1)).0 & 0xFF) as u16);
        let address = combine_u8(lsb, msb);
        address
    }

    fn indirect_indexed_address(&mut self) -> u16 {
        self.program_counter += 1;
        let op_code_arg = self.fetch(self.program_counter);
        let lsb = self.fetch(op_code_arg as u16);
        let msb = self.fetch((Wrapping(op_code_arg) + Wrapping(1)).0 as u16);
        let address = (Wrapping(combine_u8(lsb, msb)) + Wrapping(self.reg_y as u16)).0 as u16;
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
        combine_u8(lsb, msb)
    }

    fn indexed_address(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        self.program_counter += 1;
        let msb = self.fetch(self.program_counter);
        let to_add = match addressing.register {
            Some(AddressingRegistry::X) => self.reg_x,
            Some(AddressingRegistry::Y) => self.reg_y,
            _ => panic!("Addressing registry has to be filled")
        };
        (Wrapping((combine_u8(lsb, msb) as u16)) + Wrapping((to_add as u16))).0
    }

    fn zero_page_indexed_address(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        let base = self.fetch(self.program_counter);
        let to_add = match addressing.register {
            Some(AddressingRegistry::X) => self.reg_x,
            Some(AddressingRegistry::Y) => self.reg_y,
            _ => panic!("Addressing registry has to be filled")
        };
        // TODO: +1 to cycle if wrapped
        println!("base {:#01X} add {:#01X} result {:#01X}", base, to_add, (Wrapping(base) + Wrapping(to_add)).0 as u16);
        (Wrapping(base) + Wrapping(to_add)).0 as u16
    }

    fn absolute_indexed_address(&mut self, addressing: &Addressing) -> u16 {
        self.indexed_address(addressing)
    }

    fn indirect_address(&mut self, addressing: &Addressing) -> u16 {
        self.program_counter += 1;
        let lsb = self.fetch(self.program_counter);
        self.program_counter += 1;
        let msb = self.fetch(self.program_counter);

        let real_lsb_address = combine_u8(lsb, msb);
        let real_msb_address = combine_u8((Wrapping(lsb) + Wrapping(1)).0, msb);
        let real_lsb = self.fetch(real_lsb_address);
        let real_msb = self.fetch(real_msb_address);

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
        let value = self.bus.fetch(address);
        return value
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
        self.store(value, Some((self.stack_pointer as u16) + 0x100));
        self.stack_pointer -= 1;
    }

    fn read_flags_from_stack(&mut self) -> Flags {
        let mut flags: Flags = self.pull_from_stack().into();
        flags.remove(Flags::BRK);
        flags.insert(Flags::PLACEHOLDER);
        return flags
    }

    fn read_pc_from_stack(&mut self) -> u16 {
        let lsb =  self.pull_from_stack();
        let msb =  self.pull_from_stack();
        combine_u8(lsb, msb)
    }

    fn pull_from_stack(&mut self) -> u8 {
        self.stack_pointer += 1;
        // TODO: Find a fancier way to offset stack pointer
        self.fetch((self.stack_pointer as u16 + 0x100))
    }

    pub fn emulate(&mut self, mut logfile: &File) {
        if self.cycles != 0 {
            self.cycles -= 1;
        } else {
            if self.bus.nmi {
                self.cycles += self.nmi_interrupt();
            } else {
                let op_code = self.fetch(self.program_counter);
                writeln!(
                    logfile,
                    // TODO: Fix length, add padding
                    "{:01X} {} A:{} X:{} Y:{} P:{} SP:{}",
                    op_code,
                    self.debug_format(self.program_counter),
                    self.debug_format(self.acc),
                    self.debug_format(self.reg_x),
                    self.debug_format(self.reg_y),
                    self.debug_format(self.status),
                    self.debug_format(self.stack_pointer)
                );
                println!("cpu before: {:?}", self);
                let result = self.evaluate(OpCode::new(op_code));
                println!("cpu after: {:?}\n", self);
                self.cycles += result;
            }
        }
    }

    fn debug_format(&mut self, value: impl UpperHex) -> String {
        let formatted = format!("{:#01X}", value);
        let stripped = formatted.strip_prefix("0x").unwrap();
        if stripped.len() == 1 {
            return String::from("0".to_owned() + stripped)
        }
        return String::from(stripped)
    }

    pub fn evaluate(&mut self, op_code: OpCode) -> u8 {
        println!("Evaluating op code, hex: {:#02X}, bin: {:#08b}", op_code.value, op_code.value);
        return match op_code.value {
            0x18 => self.clear_flag(Flags::CARRY),
            0xD8 => self.clear_flag(Flags::DECIMAL),
            0x58 => self.clear_flag(Flags::IRQ_DIS),
            0xB8 => self.clear_flag(Flags::OVERFLOW),
            0xF8 => self.set_flag(Flags::DECIMAL),
            0x78 => self.set_flag(Flags::IRQ_DIS),
            0x38 => self.set_flag(Flags::CARRY),
            0xEA => self.noop(),
            0xAA => self.transfer(AddressingRegistry::Acc, AddressingRegistry::X),
            0xA8 => self.transfer(AddressingRegistry::Acc, AddressingRegistry::Y),
            0xBA => self.transfer(AddressingRegistry::StackPtr, AddressingRegistry::X),
            0x8A => self.transfer(AddressingRegistry::X, AddressingRegistry::Acc),
            0x9A => self.transfer(AddressingRegistry::X, AddressingRegistry::StackPtr),
            0x98 => self.transfer(AddressingRegistry::Y, AddressingRegistry::Acc),
            0x00 => self.force_break(),
            0x08 => self.push_processor_status(),
            0x28 => self.pull_processor_status(),
            0x48 => self.push_accumulator(),
            0x68 => self.pull_accumulator(),
            0x40 => self.return_from(true),
            0x60 => self.return_from(false),
            0xCA => self.offset_register_by_one(Addressing::immediate(Option::from(AddressingRegistry::X)), false),
            0x88 => self.offset_register_by_one(Addressing::immediate(Option::from(AddressingRegistry::Y)), false),
            0xC8 => self.offset_register_by_one(Addressing::immediate(Option::from(AddressingRegistry::Y)), true),
            0xE8 => self.offset_register_by_one(Addressing::immediate(Option::from(AddressingRegistry::X)), true),
            0x20 => self.jump_to_subroutine(Addressing::absolute()),
            _ => self.decode_op_code(op_code)
        }
    }

    fn decode_op_code(&mut self, op_code: OpCode) -> u8 {
        let addressing = Addressing::from_op_code(op_code.mid_op_code(), op_code.lower_op_code());
        return match (op_code.upper_op_code(), op_code.mid_op_code(), op_code.lower_op_code()) {
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
            (0b100, _, 0b10) => self.store_register(addressing, AddressingRegistry::X),
            (0b101, _, 0b10) => self.load_register(addressing, AddressingRegistry::X),
            (0b110, _, 0b10) => self.offset_memory_by_one(addressing, false),
            (0b111, _, 0b10) => self.offset_memory_by_one(addressing, true),
            (0b001, _, 0b00) => self.bit_test(addressing),
            (0b010, _, 0b00) => self.jump(Addressing::absolute()),
            (0b011, _, 0b00) => self.jump(Addressing::indirect()),
            (0b100, _, 0b00) => self.store_register(addressing, AddressingRegistry::Y),
            (0b101, _, 0b00) => self.load_register(addressing, AddressingRegistry::Y),
            (0b110, _, 0b00) => self.compare(addressing, self.reg_y),
            (0b111, _, 0b00) => self.compare(addressing, self.reg_x),
            _ => panic!("Unknown op code")
        }
    }

    fn noop(&mut self) -> u8 {
        let cycles = 2;
        self.program_counter += 1;
        cycles
    }

    fn clear_flag(&mut self, flag: Flags) -> u8 {
        let cycles = 2;
        self.status.set_flag(false, flag);
        self.program_counter += 1;
        cycles
    }

    fn set_flag(&mut self, flag: Flags) -> u8 {
        let cycles = 2;
        self.status.set_flag(true, flag);
        self.program_counter += 1;
        cycles
    }

    fn transfer(&mut self, from: AddressingRegistry, into: AddressingRegistry) -> u8 {
        let cycles = 2;
        let from = match from {
            AddressingRegistry::X => self.reg_x,
            AddressingRegistry::Y => self.reg_y,
            AddressingRegistry::Acc => self.acc,
            AddressingRegistry::StackPtr => self.stack_pointer
        };
        match into {
            AddressingRegistry::X => self.reg_x = from,
            AddressingRegistry::Y => self.reg_y = from,
            AddressingRegistry::Acc => self.acc = from,
            AddressingRegistry::StackPtr => self.stack_pointer = from
        };
        if !(into == AddressingRegistry::StackPtr) {
            self.set_zero(from as u16);
            self.set_negative(from as u16);
        }
        self.program_counter += 1;
        cycles
    }

    fn nmi_interrupt(&mut self) -> u8 {
        println!("Handling NMI interrupt");
        let cycles = 2;
        self.push_program_counter_on_stack();
        self.push_flags_on_stack();
        self.status.insert(Flags::IRQ_DIS);
        let lsb = self.fetch(0xFFFA);
        let msb = self.fetch(0xFFFB);
        self.program_counter = combine_u8(lsb, msb);
        println!("NMI program counter {:01X}", self.program_counter);
        cycles
    }

    fn push_accumulator(&mut self) -> u8 {
        let cycles = 3;
        self.push_on_stack(self.acc);
        self.program_counter += 1;
        cycles
    }

    fn pull_accumulator(&mut self) -> u8 {
        let cycles = 3;
        println!("stack: {:01X}", self.fetch(self.stack_pointer as u16));
        self.acc = self.pull_from_stack();
        self.set_zero(self.acc as u16);
        self.set_negative(self.acc as u16);
        self.program_counter += 1;
        cycles
    }

    fn push_processor_status(&mut self) -> u8 {
        let cycles = 3;
        self.push_flags_on_stack();
        self.program_counter += 1;
        cycles
    }

    fn pull_processor_status(&mut self) -> u8 {
        let cycles = 3;
        self.status = self.read_flags_from_stack();
        self.program_counter += 1;
        cycles
    }

    fn branch(&mut self, addressing: Addressing, branch_instruction: u8) -> u8 {
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
        self.program_counter += 1;
        cycles
    }

    fn extract_branch_flag(&mut self, branch_instruction: u8) -> u8 {
        let higher_bit = nth_bit(branch_instruction, 2) as u8;
        let lower_bit = nth_bit(branch_instruction, 1) as u8;
        ((higher_bit << 1) as u8) | lower_bit
    }

    fn extract_branch_equality(&mut self, branch_instruction: u8) -> bool {
        nth_bit(branch_instruction, 0)
    }

    fn branch_on_flag(&mut self, addressing: Addressing, branch_equality: bool, flag: Flags) -> (bool, bool) {
        let mut succeeded = false;
        let (raw_branch_offset, _) = self.fetch_with_addressing_mode(&addressing);
        let branch_offset = raw_branch_offset as i8;
        let flag = self.status.contains(flag);
        if flag == branch_equality {
            self.program_counter = self.program_counter.wrapping_add(branch_offset as u16);
            succeeded = true;
        };
        (succeeded, self.is_on_different_page(self.program_counter, self.program_counter + 1))
    }

    fn bit_test(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 3;
        let (to_test, _) = self.fetch_with_addressing_mode(&addressing);
        println!("to test: {:#01X}", to_test);
        let zero = (to_test & self.acc) == 0;
        let negative = msb(to_test) == 1;
        let overflow = nth_bit(to_test, 6); // TODO: Check if true

        self.status.set_flag(zero, Flags::ZERO);
        self.status.set_flag(negative, Flags::NEGATIVE);
        self.status.set_flag(overflow, Flags::OVERFLOW);

        if addressing.mode == Absolute {
            cycles += 1;
        }
        self.program_counter += 1;
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
        // TODO: Possible error here?
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
        if read_flags {
            // TODO: Probably error here
            new_pc -= 1;
        }
        self.program_counter = new_pc + 1;
        cycles
    }

    fn store_register(&mut self, addressing: Addressing, target: AddressingRegistry) -> u8 {
        let mut cycles = 3;
        let adjusted_addressing = self.adjust_addressing(addressing, target);
        let address = self.fetch_address(&adjusted_addressing);
        let register_value = if target == AddressingRegistry::X {
            self.reg_x
        } else {
            self.reg_y
        };
        println!("Storing {:#01X} at address  {:#01X}", register_value, address);
        self.store(register_value, Some(address));

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        self.program_counter += 1;
        cycles
    }

    fn offset_register_by_one(&mut self, addressing: Addressing, increment: bool) -> u8 {
        let mut cycles = 2;
        match addressing.register {
            Some(AddressingRegistry::X) => {
                let result = self.offset_by_one(self.reg_x, increment);
                self.reg_x = result;
            },
            Some(AddressingRegistry::Y) => {
                let result = self.offset_by_one(self.reg_y, increment);
                self.reg_y = result;
            }
            _ => {
                let result = self.offset_by_one(self.acc, increment);
                self.acc = result;
            }
        };
        self.program_counter += 1;
        cycles
    }

    fn offset_memory_by_one(&mut self, addressing: Addressing, increment: bool) -> u8 {
        let mut cycles = 2;
        let (mut value, address) = self.fetch_with_addressing_mode(&addressing);
        let result = self.offset_by_one(value, increment);
        self.store(result, address);

        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        self.program_counter += 1;
        cycles
    }

    fn offset_by_one(&mut self, value: u8,  increment: bool) -> u8 {
        let mut result = value;
        if increment {
            result = (Wrapping(result) + Wrapping(1)).0;
        } else {
            result = (Wrapping(result) - Wrapping(1)).0;
        }
//        self.store(result, address);
        self.set_negative(result as u16);
        self.set_zero(result as u16);
        result
    }

    fn adjust_addressing(&mut self, addressing: Addressing, target: AddressingRegistry) -> Addressing {
        // TODO: Move to addressing.rs. Use only in STX and LDX
        if target == AddressingRegistry::X {
            if addressing.mode == ZeroPageIndexed && addressing.register == Some(AddressingRegistry::X) {
                return Addressing::zero_page_indexed(Some(AddressingRegistry::Y), false)
            }
            if addressing.mode == AbsoluteIndexed && addressing.register == Some(AddressingRegistry::X) {
                return Addressing::absolute_indexed(Some(AddressingRegistry::Y), false)
            }
        }
        return addressing
    }

    fn load_register(&mut self, addressing: Addressing, target: AddressingRegistry) -> u8 {
        println!("cpu before: {:?}", self);
        let mut cycles = 2;
        let adjusted_addressing = self.adjust_addressing(addressing, target);
        let (value, _) = self.fetch_with_addressing_mode(&adjusted_addressing);
        self.set_negative(value as u16);
        self.set_zero(value as u16);
        if target == AddressingRegistry::X {
            self.reg_x = value;
        } else {
            self.reg_y = value;
        }
        println!("cpu after: {:?}", self);
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);
        self.program_counter += 1;
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
        self.program_counter += 1;
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
        self.program_counter += 1;
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
        self.program_counter += 1;
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
        self.program_counter += 1;
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
        self.program_counter += 1;
        cycles
    }

    fn store_accumulator(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let address = self.fetch_address(&addressing);
        self.store(self.acc, Some(address));
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, false);
        self.program_counter += 1;
        cycles
    }

    fn load_accumulator(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, address) = self.fetch_with_addressing_mode(&addressing);
        println!("Loading {:#01X} from {:#01X}", value, address.unwrap());
        self.set_negative(value as u16);
        self.set_zero(value as u16);
        self.acc = value;
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);
        self.program_counter += 1;
        cycles
    }

    fn carry_arithmetic(&mut self, operation: fn(u8, u8) -> (u8, bool), lhs: u8, rhs: u8) -> (u8, bool) {
        let (intermediate_result, carry_overflow) = operation(lhs, rhs);
        let (result, overflow) = operation(intermediate_result, self.status.contains(Flags::CARRY) as u8);
        (result, overflow | carry_overflow)
    }

    fn overflow_occurred(&self, lhs: u8, rhs: u8, result: u8, add: bool) -> bool {
        if add {
            (((lhs.bitxor(result)) & (rhs.bitxor(result))) & 0x80) != 0
        } else {
            ((lhs.bitxor(result)) & (lhs.bitxor(rhs)) & 0x80) != 0
        }
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
        self.program_counter += 1;
        cycles
    }

    fn add_with_carry(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
        let mut result = (self.acc as u16) + (value as u16) + (self.status.contains(Flags::CARRY) as u16);
        self.set_carry(result);
        self.set_zero(result % 256);
        self.set_negative(result);
        self.set_overflow(self.acc, value, result, true);
        self.acc = (result % 256) as u8;
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);
        self.program_counter += 1;
        cycles
    }

    fn sub_with_borrow(&mut self, addressing: Addressing) -> u8 {
        let mut cycles = 2;
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
        let carry = if self.status.contains(Flags::CARRY) { 0 } else { 1 };
        let mut result = (Wrapping(self.acc as u16) - (Wrapping(value as u16)) - Wrapping(carry));
        self.set_borrow(result.0);
        self.set_zero(result.0 % 256);
        self.set_negative(result.0 % 256);
        self.set_overflow(self.acc, value, result.0, false);

        self.acc = (result.0 % 256) as u8;
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, true);
        self.program_counter += 1;
        cycles
    }

    fn bitwise_instruction(&mut self, addressing: Addressing, operation: fn(u8, u8) -> u8, additional_cycle: bool) -> u8 {
        let mut cycles = 2;
        let (value, _) = self.fetch_with_addressing_mode(&addressing);
        self.acc = operation(self.acc, value);
        cycles += self.count_additional_cycles(cycles, addressing.add_cycles, additional_cycle);

        self.set_zero(self.acc as u16);
        self.set_negative(self.acc as u16);
        self.program_counter += 1;
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
    use crate::ppu::Ppu;
    use crate::cartridge::Cartridge;

    fn create_test_bus(input: Vec<u8>) -> Bus {
        let ppu = Ppu::new(vec![]);
        let cartridge = Cartridge::new();
        return Bus::new(input, ppu, cartridge);
    }

    fn create_test_cpu(input: Vec<u8>) -> Cpu {
        let mut bus = create_test_bus(input);
        Cpu::new(bus)
    }

    fn reset_cpu(cpu: &mut Cpu) {
        cpu.cycles = 0;
        cpu.acc = 0;
        cpu.reg_x = 0;
        cpu.reg_y = 0;
        cpu.program_counter = 0;
    }

    #[test]
    fn test_bit_or() {
        let mut cpu = create_test_cpu(vec![0x01, 0x02, 0x04, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x01));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_and() {
        let mut cpu = create_test_cpu(vec![0x21, 0x02, 0x04, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x21));
        assert_eq!(cpu.acc, 0b0000_0000);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_xor() {
        let mut cpu = create_test_cpu(vec![0x41, 0x02, 0x04, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.acc = 0b1111_1111;
        cpu.evaluate(OpCode::new(0x41));
        assert_eq!(cpu.acc, 0b0000_0000);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_adc() {
        let mut cpu = create_test_cpu(vec![0x61, 0x02, 0x04, 0x00, 2]);
        reset_cpu(&mut cpu);
        cpu.acc = 3;
        cpu.evaluate(OpCode::new(0x61));
        assert_eq!(cpu.acc, 5);
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_sbc() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 2]);
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
        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 176]);
        reset_cpu(&mut cpu);
        cpu.acc = 80;
        cpu.evaluate(OpCode::new(0xE1));
        assert_eq!(cpu.acc, 160);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::OVERFLOW | Flags:: NEGATIVE)
    }

    #[test]
    fn test_overflow_add() {
        let mut cpu = create_test_cpu(vec![0x61, 0x02, 0x04, 0x00, 80]);
        reset_cpu(&mut cpu);
        cpu.acc = 80;
        cpu.evaluate(OpCode::new(0x61));
        assert_eq!(cpu.acc, 160);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::OVERFLOW | Flags::NEGATIVE)
    }

    #[test]
    fn test_carry() {
        let mut cpu = create_test_cpu(vec![0x61, 0x02, 0x04, 0x00, 80]);
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
        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 2]);
        reset_cpu(&mut cpu);
        cpu.acc = 1;
        cpu.evaluate(OpCode::new(0xE1));
        assert_eq!(cpu.acc, 255);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE)
    }

    #[test]
    fn test_compare() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 10]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xC1));
        assert_eq!(cpu.acc, 10);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::ZERO | Flags::CARRY);

        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 9]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xC1));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);

        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 11]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xC1));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);

        // cpy
        let mut cpu = create_test_cpu(vec![0xCC, 0x03, 0x00, 11]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 10;
        cpu.evaluate(OpCode::new(0xCC));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);

        //cpx
        let mut cpu = create_test_cpu(vec![0xCC, 0x03, 0x00, 11]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 10;
        cpu.evaluate(OpCode::new(0xCC));
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);
    }

    #[test]
    fn test_load_accumulator() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 180]);
        reset_cpu(&mut cpu);
        cpu.acc = 0;
        cpu.evaluate(OpCode::new(0xA1));
        assert_eq!(cpu.acc, 180);
    }

    #[test]
    fn test_store_accumulator() {
        let mut cpu = create_test_cpu(vec![0xE1, 0x02, 0x04, 0x00, 180]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0x81));
        assert_eq!(cpu.fetch(4), 10);
    }

    #[test]
    fn test_shift_left() {
        let mut cpu = create_test_cpu(vec![0x0E, 0x03, 0x00, 20, 5, 6, 7]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x0E));
        assert_eq!(cpu.fetch(3), 40);

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
        let mut cpu = create_test_cpu(vec![0x2E, 0x03, 0x00, 0b1000_0000]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::CARRY;
        cpu.evaluate(OpCode::new(0x2E));
        assert_eq!(cpu.fetch(3), 0b0000_0001);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);
    }

    #[test]
    fn test_logical_shift_right() {
        let mut cpu = create_test_cpu(vec![0x4E, 0x03, 0x00, 0b1000_0001]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x4E));
        assert_eq!(cpu.fetch(3), 0b0100_0000);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY);
    }

    #[test]
    fn test_rotate_right() {
        let mut cpu = create_test_cpu(vec![0x6E, 0x03, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::CARRY;
        cpu.evaluate(OpCode::new(0x6E));
        assert_eq!(cpu.fetch(3), 0b1000_0000);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::NEGATIVE);
    }

    #[test]
    fn test_store_register_x() {
        let mut cpu = create_test_cpu(vec![0x96, 0x03, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 10;
        cpu.evaluate(OpCode::new(0x96));
        assert_eq!(cpu.fetch(3), 10);
    }

    #[test]
    fn test_load_register_x() {
        let mut cpu = create_test_cpu(vec![0xB6, 0x03, 0x00, 150]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 10;
        cpu.evaluate(OpCode::new(0xB6));
        assert_eq!(cpu.reg_x, 150);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    //TODO: Add test for ZP, X
    #[test]
    fn test_store_register_y() {
        let mut cpu = create_test_cpu(vec![0x8C, 0x03, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 10;
        cpu.evaluate(OpCode::new(0x8C));
        assert_eq!(cpu.fetch(3), 10);
    }

    //TODO: Add test for ZP, X
    #[test]
    fn test_load_register_y() {
        let mut cpu = create_test_cpu(vec![0xAC, 0x03, 0x00, 150]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 10;
        cpu.evaluate(OpCode::new(0xAC));
        assert_eq!(cpu.reg_y, 150);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_increment() {
        let mut cpu = create_test_cpu(vec![0xEE, 0x03, 0x00, 0b0000_0000]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0xFE));
        assert_eq!(cpu.fetch(3), 1);
    }

    #[test]
    fn test_decrement() {
        let mut cpu = create_test_cpu(vec![0xCE, 0x03, 0x00, 1]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0xCE));
        assert_eq!(cpu.fetch(3), 0);
    }

    #[test]
    fn test_increment_reg_x() {
        let mut cpu = create_test_cpu(vec![0xE8]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 0;
        cpu.status = Flags::ZERO | Flags::PLACEHOLDER | Flags::NEGATIVE;
        cpu.evaluate(OpCode::new(0xE8));
        assert_eq!(cpu.reg_x, 1);
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_decrement_reg_y() {
        let mut cpu = create_test_cpu(vec![0x88]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 1;
        cpu.evaluate(OpCode::new(0x88));
        assert_eq!(cpu.reg_y, 0);
        assert_eq!(cpu.status, Flags::ZERO | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_bit_test() {
        let mut cpu = create_test_cpu(vec![0x2C, 0x03, 0x00, 0b1100_0000]);
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
        let mut cpu = create_test_cpu(vec![0x6C, 0x03, 0x00, 20, 0]);
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
        assert_eq!(cpu.program_counter, 5)
    }

    #[test]
    fn test_break() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        // TODO: Fix off by one in tests
        memory[0xFFFE] = 0x44;
        memory[0xFFFF] = 0x66;
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
        cpu.program_counter = 2;
        cpu.evaluate(OpCode::new(0x20));
        assert_eq!(cpu.program_counter, 0x03);

        let lsb_stored_program_counter =  cpu.fetch((cpu.stack_pointer + 1) as u16);
        let msb_stored_program_counter =  cpu.fetch((cpu.stack_pointer + 2) as u16);

        assert_eq!(combine_u8(lsb_stored_program_counter, msb_stored_program_counter), 1)
    }

    #[test]
    fn test_rti() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        let flags_on_stack = Flags::NEGATIVE | Flags::PLACEHOLDER | Flags::OVERFLOW;
        memory[0] = 0x40;
        memory[0x00FF] = 0x44;
        memory[0x00FE] = 0x66;
        memory[0x00FD] = flags_on_stack.bits();

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
        memory[0x00FF] = 0x44;
        memory[0x00FE] = 0x66;

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
        memory[0x00FF] = stored_flags.bits();

        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.stack_pointer -= 1;

        cpu.evaluate(OpCode::new(0x28));

        assert_eq!(cpu.status, stored_flags)
    }

    #[test]
    fn test_push_acc() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.acc = 20;
        cpu.evaluate(OpCode::new(0x48));

        let stored_acc = cpu.fetch(((cpu.stack_pointer + 1) as u16));
        assert_eq!(cpu.acc, stored_acc)
    }

    #[test]
    fn test_pull_acc() {
        let len = 0x10000;
        let mut memory = vec![0; len];
        let acc = 20;
        memory[0x00FF] = acc;
        let mut cpu = create_test_cpu(memory);
        reset_cpu(&mut cpu);
        cpu.stack_pointer -= 1;
        cpu.evaluate(OpCode::new(0x68));

        assert_eq!(cpu.acc, acc)
    }

    #[test]
    fn test_transfer() {
        let mut cpu = create_test_cpu(vec![0xAA]);
        reset_cpu(&mut cpu);
        cpu.acc = 10;
        cpu.evaluate(OpCode::new(0xAA));
        assert_eq!(cpu.reg_x, 10);

        reset_cpu(&mut cpu);
        cpu.reg_x = 0b1111_1111;
        cpu.evaluate(OpCode::new(0x9A));
        assert_eq!(cpu.stack_pointer, 0b1111_1111);
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_set_flag() {
        let mut cpu = create_test_cpu(vec![0x38]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER;
        cpu.evaluate(OpCode::new(0x38));
        println!("{:?}", cpu);
        assert_eq!(cpu.status, Flags::PLACEHOLDER | Flags::CARRY)
    }

    #[test]
    fn test_clear_carry() {
        let mut cpu = create_test_cpu(vec![0x18]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::CARRY;
        cpu.evaluate(OpCode::new(0x18));
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_clear_decimal() {
        let mut cpu = create_test_cpu(vec![0xD8]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::DECIMAL;
        cpu.evaluate(OpCode::new(0xD8));
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_clear_irq_dis() {
        let mut cpu = create_test_cpu(vec![0x58]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::IRQ_DIS;
        cpu.evaluate(OpCode::new(0x58));
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_clear_overflow() {
        let mut cpu = create_test_cpu(vec![0xB8]);
        reset_cpu(&mut cpu);
        cpu.status = Flags::PLACEHOLDER | Flags::OVERFLOW;
        cpu.evaluate(OpCode::new(0xB8));
        assert_eq!(cpu.status, Flags::PLACEHOLDER)
    }

    #[test]
    fn test_indexed_indirect() {
        let mut cpu = create_test_cpu(vec![0x01, 0x02, 0x04, 0x00, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x01));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_zero_page() {
        let mut cpu = create_test_cpu(vec![0x05, 0x02, 0b1111_1111]);
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
        let mut cpu = create_test_cpu(vec![0x05, 0x3, 0x0, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x0D));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_indirect_indexed() {
        let mut cpu = create_test_cpu(vec![0x05, 0x0, 0b1111_1111, 0x0]);
        reset_cpu(&mut cpu);
        cpu.reg_y = 2;
        cpu.evaluate(OpCode::new(0x11));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_zero_page_indexed() {
        let mut cpu = create_test_cpu(vec![0x05, 0x0, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.reg_x = 2;
        cpu.evaluate(OpCode::new(0x15));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }

    #[test]
    fn test_absolute_indexed() {
        let mut cpu = create_test_cpu(vec![0x05, 0x3, 0x0, 0b1111_1111]);
        reset_cpu(&mut cpu);
        cpu.evaluate(OpCode::new(0x19));
        assert_eq!(cpu.acc, 0b1111_1111);
        assert_eq!(cpu.status, Flags::NEGATIVE | Flags::PLACEHOLDER)
    }
}
