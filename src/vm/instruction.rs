use crate::vm::opcode;
use crate::vm::opcode::OpCode;
use crate::vm::field::Field;
use crate::vm::stack::Stack;
use crate::vm::builder::builder;

pub struct Instruction {
    pub opcode: opcode::OpCode,
    pub operand: Stack<Field>
}

impl Instruction {
    pub fn new(opcode: OpCode, operand: Vec<Field>) -> Self {
        let mut stack: Stack<Field> = Stack::new();
        for field in operand {
            stack.push(field);
        }
        Instruction {
            opcode,
            operand: stack
        }
    }
}

impl From<builder> for Vec<Instruction> {
    fn from(builder: builder) -> Vec<Instruction> {
        builder.instructions
    }
}