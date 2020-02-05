pub struct OpCode {
    pub value: u8,
}

impl OpCode {
    pub fn msb(&self) -> u8 {
        (self.value & 0xF0) >> 4
    }

    pub fn lsb(&self) -> u8 {
        self.value & 0x0F
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msb() {
        let x = OpCode { value: 0xF0 };
        assert_eq!(x.msb(), 0xF)
    }

    #[test]
    fn test_lsb() {
        let x = OpCode { value: 0x0F };
        assert_eq!(x.lsb(), 0xF)
    }
}