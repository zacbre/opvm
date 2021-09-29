use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::Vm;
use crate::vm::field::Field;
use crate::vm::builder::builder;

mod lexer;
mod vm;

fn main() {
    let mut builder = builder::new();
    builder.push(OpCode::Push, vec![Field::from(4)]);
    builder.push(OpCode::Push, vec![Field::from(4)]);
    builder.push(OpCode::Add, vec![]);
    builder.push(OpCode::Print, vec![]);

    let instructions = Instruction::from(builder);

    let mut vm = Vm::new(instructions);
    vm.execute();
}
