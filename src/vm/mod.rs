use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::field::Field;

pub mod instruction;
pub mod opcode;
pub mod stack;
pub mod field;
pub mod builder;

pub struct Vm {
    instructions: Vec<Instruction>,
    stack: stack::Stack<Field>,
    pc: i32,
}

impl Vm {
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Vm{
            instructions,
            stack: stack::Stack::new(),
            pc: 0
        }
    }

    pub fn execute(&mut self) {
        while (self.pc as usize) < self.instructions.len() {
            let mut instruction = &mut self.instructions[self.pc as usize];
            match instruction.opcode {
                OpCode::Push => {
                    self.stack.push(instruction.operand.pop());
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Add => {
                    let a1 = self.stack.pop().to_i32().unwrap();
                    let a2 = self.stack.pop().to_i32().unwrap();
                    self.stack.push(Field::Number(a1 + a2));
                }
                OpCode::Mul => {
                    let a1 = self.stack.pop().to_i32().unwrap();
                    let a2 = self.stack.pop().to_i32().unwrap();
                    self.stack.push(Field::Number(a1 * a2));
                }
                OpCode::Sub => {
                    let a1 = self.stack.pop().to_i32().unwrap();
                    let a2 = self.stack.pop().to_i32().unwrap();
                    self.stack.push(Field::Number(a1 - a2));
                }
                OpCode::Div => {
                    let a1 = self.stack.pop().to_i32().unwrap();
                    let a2 = self.stack.pop().to_i32().unwrap();
                    self.stack.push(Field::Number(a1 / a2));
                }
                OpCode::Mod => {
                    let a1 = self.stack.pop().to_i32().unwrap();
                    let a2 = self.stack.pop().to_i32().unwrap();
                    self.stack.push(Field::Number(a1 % a2));
                }
                OpCode::Print => {
                    println!("{}", self.stack.pop());
                }
                OpCode::Call => {
                    self.stack.push(Field::from(self.pc))
                }
                OpCode::Ret => {
                    let new_pc = self.stack.pop();
                    self.pc = new_pc.to_i32().unwrap();
                }
            }
            self.pc += 1;
        }
    }
}
