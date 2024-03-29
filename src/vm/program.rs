use std::collections::HashMap;
use crate::vm::field::Field;
use crate::vm::instruction::Instruction;

#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
    pub data: HashMap<String, Field>
}

impl Program {
    pub fn new() -> Self {
        Program{ instructions: vec![], labels: Default::default(), data: Default::default() }
    }
}