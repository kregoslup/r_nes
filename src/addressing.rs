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
    Accumulator
}

#[derive(PartialEq, Copy, Clone)]
pub enum AddressingRegistry {
    X,
    Y
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

    pub fn immediate() -> Addressing {
        Addressing {
            register: None,
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

    pub fn to_register_specific_addressing(&self) -> Addressing {
        let fixed_addressing_register = match self.register {
            Some(register) => {
                if register == AddressingRegistry::X {
                    Some(AddressingRegistry::Y)
                } else {
                    Some(AddressingRegistry::X)
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
}
