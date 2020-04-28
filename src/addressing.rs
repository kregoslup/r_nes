use std::borrow::{BorrowMut, Borrow};

pub struct Addressing {
    pub register: Option<AddressingRegistry>,
    pub add_cycles: bool,
    pub mode: AddressingMode
}

// Add wrapping around
#[derive(PartialEq, Copy, Clone)]
pub enum AddressingMode {
    IndexedIndirect,
    IndirectIndexed,
    ZeroPage,
    Immediate,
    Absolute,
    AbsoluteIndexed,
    ZeroPageIndexed,
    Accumulator,
    Indirect,
    Relative,
    Implied,
}

#[derive(PartialEq, Copy, Clone)]
pub enum AddressingRegistry {
    X,
    Y,
    Acc,
    StackPtr
}

impl Addressing {
    pub fn indexed_indirect() -> Addressing {
        Addressing {
            register: None,
            add_cycles: false,
            mode: AddressingMode::IndexedIndirect
        }
    }

    pub fn indirect_indexed() -> Addressing {
        Addressing {
            register: None,
            add_cycles: true,
            mode: AddressingMode::IndirectIndexed
        }
    }

    pub fn zero_page() -> Addressing {
        Addressing {
            register: None,
            add_cycles: true,
            mode: AddressingMode::ZeroPage
        }
    }

    pub fn immediate(addressing_registry: Option<AddressingRegistry>) -> Addressing {
        Addressing {
            register: addressing_registry,
            add_cycles: true,
            mode: AddressingMode::Immediate
        }
    }

    pub fn absolute() -> Addressing {
        Addressing {
            register: None,
            add_cycles: false,
            mode: AddressingMode::Absolute
        }
    }

    pub fn absolute_indexed(reg: Option<AddressingRegistry>, add_cycles: bool) -> Addressing {
        Addressing {
            register: reg,
            add_cycles,
            mode: AddressingMode::AbsoluteIndexed
        }
    }

    pub fn zero_page_indexed(reg: Option<AddressingRegistry>, add_cycles: bool) -> Addressing {
        Addressing {
            register: reg,
            add_cycles,
            mode: AddressingMode::ZeroPageIndexed
        }
    }

    pub fn accumulator() -> Addressing {
        Addressing {
            register: None,
            add_cycles: false,
            mode: AddressingMode::Accumulator
        }
    }

    pub fn indirect() -> Addressing {
        Addressing {
            register: None,
            add_cycles: false,
            mode: AddressingMode::Indirect
        }
    }

    pub fn relative() -> Addressing {
        Addressing {
            register: None,
            add_cycles: false,
            mode: AddressingMode::Relative
        }
    }

    pub fn to_register_specific_addressing(&self) -> Addressing {
        let fixed_addressing_register = match self.register {
            Some(register) => {
                if register == AddressingRegistry::X {
                    Some(AddressingRegistry::Y)
                } else {
                    // TODO: Confirm
                    Some(register)
                }
            },
            None => None
        };
        Addressing {
            register: fixed_addressing_register,
            add_cycles: self.add_cycles,
            mode: self.mode
        }
    }

    pub fn from_op_code(mid_op_code: u8, lower_op_code: u8) -> Addressing {
        println!("Extracting addressing from {:#010b}", mid_op_code);
        match (mid_op_code, lower_op_code) {
            // c == 0b10
            (0b0, 0b01) => Addressing::indexed_indirect(),
            (0b001, 0b01) => Addressing::zero_page(),
            (0b010, 0b01) => Addressing::immediate(None),
            (0b011, 0b01) => Addressing::absolute(),
            (0b100, 0b01) => Addressing::indirect_indexed(),
            (0b101, 0b01) => Addressing::zero_page_indexed(Some(AddressingRegistry::X), false),
            (0b110, 0b01) => Addressing::absolute_indexed(Some(AddressingRegistry::Y), true),
            (0b111, 0b01) => Addressing::absolute_indexed(Some(AddressingRegistry::X), false),
            // c == 0b10
            (0b000, 0b10) => Addressing::immediate(None),
            (0b001, 0b10) => Addressing::zero_page(),
            (0b010, 0b10) => Addressing::accumulator(),
            (0b011, 0b10) => Addressing::absolute(),
            (0b101, 0b10) => Addressing::zero_page_indexed(Some(AddressingRegistry::X), false),
            (0b111, 0b10) => Addressing::absolute_indexed(Some(AddressingRegistry::X), false),
            // c == 00
            (0b000, 0b00) => Addressing::immediate(None),
            (0b001, 0b00) => Addressing::zero_page(),
            (0b011, 0b00) => Addressing::absolute(),
            (0b101, 0b00) => Addressing::zero_page_indexed(Some(AddressingRegistry::X), false),
            (0b111, 0b00) => Addressing::absolute_indexed(Some(AddressingRegistry::X), false),
            (_, 0b0) => Addressing::relative(),
            _ => panic!("Unknown addressing type")
        }
    }
}
