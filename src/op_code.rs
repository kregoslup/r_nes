use crate::addressing::Addressing;

pub struct OpCode {
    pub value: u8,
}

impl OpCode {

    pub fn new(value: u8) -> OpCode {
        OpCode { value }
    }

    pub fn upper_op_code(&self) -> u8 {
        (self.value & 0b1110_0000) >> 5
    }

    pub fn mid_op_code(&self) -> u8 {
        (self.value & 0b0001_1100) >> 2
    }

    pub fn lower_op_code(&self) -> u8 {
        self.value & 0b0000_0011
    }
}

trait BranchOperation {
    fn extract_branch_flag(&self) -> Self;

    fn extract_branch_equality(&self) -> Self;
}

impl BranchOperation for u8 {
    fn extract_branch_flag(&self) -> Self {
        unimplemented!()
    }

    fn extract_branch_equality(&self) -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

}