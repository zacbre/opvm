use crate::vm::instruction::Instruction;
use crate::vm::field::Field;
use crate::vm::opcode::OpCode;

pub struct builder {
    pub instructions: Vec<Instruction>
}

impl builder {
    pub fn new() -> Self {
        builder{
            instructions: vec![]
        }
    }

    pub fn push(&mut self, opcode: OpCode, operand: Vec<Field>) {
        self.instructions.push(Instruction::new(opcode, operand));
    }

    pub fn label(&mut self, name: &str) {
        self.push(OpCode::Label, vec![Field::from(name)]);
    }
}
