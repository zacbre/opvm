use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::field::Field;
use std::collections::HashMap;
use crate::vm::program::Program;
use crate::vm::stack;

pub struct Vm {
    instructions: Vec<Instruction>,
    labels: HashMap<String,usize>,
    data: HashMap<String, Field>,
    stack: stack::Stack<Field>,
    pc: usize,
}

impl Vm {
    pub fn new() -> Self {
        Vm{
            instructions: vec![],
            labels: HashMap::new(),
            data: HashMap::new(),
            stack: stack::Stack::new(),
            pc: 0,
        }
    }

    pub fn execute(&mut self, program: Program) {
        self.instructions = program.instructions;
        self.labels = program.labels;
        self.data = program.data;

        while (self.pc as usize) < self.instructions.len() {
            let instruction = &mut self.instructions[self.pc as usize];
            match instruction.opcode {
                OpCode::Push => {
                    let operand = instruction.operand.pop();
                    let operand_as_str = operand.to_str();
                    match operand_as_str {
                        Some(s) => {
                            if self.data.contains_key(s) {
                                self.stack.push(self.data.get(s).unwrap().clone());
                            }
                            else {
                                self.stack.push(operand);
                            }
                        }
                        None => self.stack.push(operand)
                    }
                }
                OpCode::Pop => {
                    self.stack.pop();
                }
                OpCode::Add => {
                    let a2 = self.stack.pop().to_i().unwrap();
                    let a1 = self.stack.pop().to_i().unwrap();
                    self.stack.push(Field::I(a1 + a2));
                }
                OpCode::Mul => {
                    let a2 = self.stack.pop().to_i().unwrap();
                    let a1 = self.stack.pop().to_i().unwrap();
                    self.stack.push(Field::I(a1 * a2));
                }
                OpCode::Sub => {
                    let a2 = self.stack.pop().to_i().unwrap();
                    let a1 = self.stack.pop().to_i().unwrap();
                    self.stack.push(Field::I(a1 - a2));
                }
                OpCode::Div => {
                    let a2 = self.stack.pop().to_i().unwrap();
                    let a1 = self.stack.pop().to_i().unwrap();
                    self.stack.push(Field::I(a1 / a2));
                }
                OpCode::Mod => {
                    let a2 = self.stack.pop().to_i().unwrap();
                    let a1 = self.stack.pop().to_i().unwrap();
                    self.stack.push(Field::I(a1 % a2));
                }
                OpCode::Print => {
                    println!("{}", self.stack.pop());
                }
                OpCode::Call => {
                    self.stack.push(Field::from(self.pc+1));
                    let label = instruction.operand.pop().to_s().unwrap();
                    self.pc = *self.labels.get(&label).unwrap();
                    continue;
                }
                OpCode::Ret => {
                    let new_pc = self.stack.pop();
                    self.pc = new_pc.to_u().unwrap();
                    continue;
                }
                OpCode::Jmp => {
                    let label = instruction.operand.pop().to_s().unwrap();
                    self.pc = *self.labels.get(&label).unwrap();
                    continue;
                }
                OpCode::Je => {
                    let v2 = self.stack.pop();
                    let v1 = self.stack.pop();
                    if v1 == v2 {
                        let label = instruction.operand.pop().to_s().unwrap();
                        self.pc = *self.labels.get(&label).unwrap();
                        continue;
                    }
                }
                OpCode::Jne => {
                    let v2 = self.stack.pop();
                    let v1 = self.stack.pop();
                    if v1 != v2 {
                        let label = instruction.operand.pop().to_s().unwrap();
                        self.pc = *self.labels.get(&label).unwrap();
                        continue;
                    }
                }
                OpCode::Jl => {
                    let v2 = self.stack.pop();
                    let v1 = self.stack.pop();
                    if v1 < v2 {
                        let label = instruction.operand.pop().to_s().unwrap();
                        self.pc = *self.labels.get(&label).unwrap();
                        continue;
                    }
                }
                OpCode::Jg => {
                    let v2 = self.stack.pop();
                    let v1 = self.stack.pop();
                    if v1 > v2 {
                        let label = instruction.operand.pop().to_s().unwrap();
                        self.pc = *self.labels.get(&label).unwrap();
                        continue;
                    }
                }
                OpCode::Jle => {
                    let v2 = self.stack.pop();
                    let v1 = self.stack.pop();
                    if v1 <= v2 {
                        let label = instruction.operand.pop().to_s().unwrap();
                        self.pc = *self.labels.get(&label).unwrap();
                        continue;
                    }
                }
                OpCode::Jge => {
                    let v2 = self.stack.pop();
                    let v1 = self.stack.pop();
                    if v1 >= v2 {
                        let label = instruction.operand.pop().to_s().unwrap();
                        self.pc = *self.labels.get(&label).unwrap();
                        continue;
                    }
                }
                _ => ()
            }
            self.pc += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)])
        ]);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 4 as i64);
    }

    #[test]
    fn test_pop() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Pop, vec![])
        ]);

        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_add() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Add, vec![])
        ]);

        assert_eq!(vm.stack.pop().to_i().unwrap(), 9);
    }

    #[test]
    fn test_mul() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Mul, vec![])
        ]);

        assert_eq!(vm.stack.pop().to_i().unwrap(), 20);
    }

    #[test]
    fn test_sub() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Sub, vec![])
        ]);

        assert_eq!(vm.stack.pop().to_i().unwrap(), 7);
    }

    #[test]
    fn test_div() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(12)]),
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Div, vec![])
        ]);

        assert_eq!(vm.stack.pop().to_i().unwrap(), 4);
    }

    #[test]
    fn test_mod() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(13)]),
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Mod, vec![])
        ]);

        assert_eq!(vm.stack.pop().to_i().unwrap(), 1);
    }

    #[test]
    fn test_print() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Print, vec![])
        ]);

        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_call() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Call, vec![Field::from("func")]),
            Instruction::new(OpCode::Label, vec![Field::from("func")]),
            Instruction::new(OpCode::Push, vec![Field::from("should be on stack")]),
        ]);

        assert_eq!(vm.stack.pop().to_str().unwrap(), "should be on stack");
    }

    #[test]
    fn test_ret() {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Call, vec![Field::from("func")]),
            Instruction::new(OpCode::Jmp, vec![Field::from("end")]),
            Instruction::new(OpCode::Label, vec![Field::from("func")]),
            Instruction::new(OpCode::Push, vec![Field::from("test")]),
            Instruction::new(OpCode::Pop, vec![]),
            Instruction::new(OpCode::Ret, vec![]),
            Instruction::new(OpCode::Label, vec![Field::from("end")]),
            Instruction::new(OpCode::Push, vec![Field::from("should be on stack")]),
        ]);

        assert_eq!(vm.stack.pop().to_str().unwrap(), "should be on stack");
        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_label() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Jmp, vec![Field::from("end")]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Label, vec![Field::from("end")]),
        ]);

        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_jmp() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Jmp, vec![Field::from("end")]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Label, vec![Field::from("end")]),
        ]);

        assert_eq!(vm.stack.len(), 0);
    }

    #[test]
    fn test_je() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Je, vec![Field::from("equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("equal")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Push, vec![Field::from(2)]),
            Instruction::new(OpCode::Je, vec![Field::from("equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("equal")]),
        ]);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 5);
    }

    #[test]
    fn test_jne() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(2)]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Jne, vec![Field::from("notequal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("notequal")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Jne, vec![Field::from("notequal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("notequal")]),
        ]);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 5);
    }

    #[test]
    fn test_jle() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jle, vec![Field::from("less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("less")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jle, vec![Field::from("equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("equal")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jle, vec![Field::from("less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("less")]),
        ]);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 5);
    }

    #[test]
    fn test_jge() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jge, vec![Field::from("greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("greater")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jge, vec![Field::from("equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("equal")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jge, vec![Field::from("greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("greater")]),
        ]);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 5);
    }

    #[test]
    fn test_jl() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jl, vec![Field::from("less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("less")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jl, vec![Field::from("less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("less")]),
        ]);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 5);
    }

    #[test]
    fn test_jg() {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jg, vec![Field::from("greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("greater")]),
        ]);

        assert_eq!(vm.stack.len(), 0);

        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jg, vec![Field::from("greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Label, vec![Field::from("greater")]),
        ]);


        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack.pop().to_i().unwrap(), 5);
    }

    fn create_vm(instructions: Vec<Instruction>) -> Vm {
        let mut vm = Vm::new(instructions);
        vm.execute();
        vm
    }
}