use std::cmp;
use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::field::Field;
use std::collections::HashMap;
use crate::vm::error::Error;
use crate::vm::program::Program;
use crate::vm::stack;
use crate::vm::stack::Stack;

pub struct Vm {
    instructions: Vec<Instruction>,
    labels: HashMap<String,usize>,
    data: HashMap<String, Field>,
    stack: stack::Stack<Field>,
    call_stack: stack::Stack<usize>,
    pc: usize,
}

impl Vm {
    pub fn new() -> Self {
        Vm{
            instructions: vec![],
            labels: HashMap::new(),
            data: HashMap::new(),
            stack: stack::Stack::new(),
            call_stack: stack::Stack::new(),
            pc: 0,
        }
    }

    pub fn execute(&mut self, program: Program) -> Result<(), Error> {
        self.instructions = program.instructions;
        self.labels = program.labels;
        self.data = program.data;
        while !self.stack.is_empty() {
            self.stack.pop();
        }

        while (self.pc as usize) < self.instructions.len() {
            let tmp_ins = &self.instructions[self.pc as usize];
            let mut instruction = tmp_ins.clone();
            match instruction.opcode {
                OpCode::Push => {
                    let operand = self.pop_operand(&mut instruction.operand)?;
                    let operand_as_str = operand.to_str();
                    match operand_as_str {
                        Some(s) => {
                            if self.data.contains_key(s) {
                                self.stack.push(self.data.get(s).unwrap().clone());
                            } else {
                                self.stack.push(operand);
                            }
                        }
                        None => self.stack.push(operand)
                    }
                }
                OpCode::Pop => {
                    self.pop_stack()?;
                }
                OpCode::Add => {
                    let a2 = self.pop_stack()?;
                    let a1 = self.pop_stack()?;
                    let i2 = self.check_int(a2)?;
                    let i1 = self.check_int(a1)?;
                    self.stack.push(Field::I(i1 + i2));
                }
                OpCode::Mul => {
                    let a2 = self.pop_stack()?;
                    let a1 = self.pop_stack()?;
                    let i2 = self.check_int(a2)?;
                    let i1 = self.check_int(a1)?;
                    self.stack.push(Field::I(i1 * i2));
                }
                OpCode::Sub => {
                    let a2 = self.pop_stack()?;
                    let a1 = self.pop_stack()?;
                    let i2 = self.check_int(a2)?;
                    let i1 = self.check_int(a1)?;
                    self.stack.push(Field::I(i1 - i2));
                }
                OpCode::Div => {
                    let a2 = self.pop_stack()?;
                    let a1 = self.pop_stack()?;
                    let i2 = self.check_int(a2)?;
                    let i1 = self.check_int(a1)?;
                    self.stack.push(Field::I(i1 / i2));
                }
                OpCode::Mod => {
                    let a2 = self.pop_stack()?;
                    let a1 = self.pop_stack()?;
                    let i2 = self.check_int(a2)?;
                    let i1 = self.check_int(a1)?;
                    self.stack.push(Field::I(i1 % i2));
                }
                OpCode::Print => {
                    println!("{}", self.pop_stack()?);
                }
                OpCode::Call => {
                    self.call_stack.push(self.pc + 1);
                    let label = self.pop_operand(&mut instruction.operand)?;
                    let result = self.jump_to_label(label, &self.labels)?;
                    self.pc = result;
                    continue;
                }
                OpCode::Ret => {
                    self.pc = self.pop_call_stack()?;
                    continue;
                }
                OpCode::Jmp => {
                    let operand = self.pop_operand(&mut instruction.operand)?;
                    let result = self.jump_to_label(operand.clone(), &self.labels)?;
                    self.pc = result;
                    continue;
                }
                OpCode::Je => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;
                    if v1 == v2 {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jne => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;
                    if v1 != v2 {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jl => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;
                    if v1 < v2 {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jg => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;
                    if v1 > v2 {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jle => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;
                    if v1 <= v2 {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jge => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;
                    if v1 >= v2 {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Inc => {
                    let v1 = self.pop_stack()?;
                    match v1 {
                        Field::I(mut i) => {
                            i += 1;
                            self.stack.push(Field::from(i));
                        }
                        Field::U(mut u) => {
                            u += 1;
                            self.stack.push(Field::from(u));
                        }
                        _ => {
                            return self.error(format!("Cannot increment non-int type at {}!", self.pc), Some(v1));
                        }
                    }
                }
                OpCode::Dec => {
                    let v1 = self.pop_stack()?;
                    match v1 {
                        Field::I(mut i) => {
                            i -= 1;
                            self.stack.push(Field::from(i));
                        }
                        Field::U(mut u) => {
                            u -= 1;
                            self.stack.push(Field::from(u));
                        }
                        _ => {
                            return self.error(format!("Cannot decrement non-int type at {}!", self.pc), Some(v1));
                        }
                    }
                }
                OpCode::Dup => {
                    let v1 = self.pop_stack()?;
                    // push to the stack twice.
                    self.stack.push(v1.clone());
                    self.stack.push(v1);
                }
                OpCode::Concat => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;

                    self.stack.push(Field::from(format!("{}{}", v1.to_string(),v2.to_string())));
                }
                OpCode::Swap => {
                    let v2 = self.pop_stack()?;
                    let v1 = self.pop_stack()?;

                    self.stack.push(v2);
                    self.stack.push(v1);
                }
                OpCode::Nop => (),
                OpCode::Hlt => {
                    return Ok(());
                }
                OpCode::Igl => {
                    return self.error(format!("ILLEGAL instruction encountered at {}.", self.pc), None);
                }
            }
            self.pc += 1;
        }
        Ok(())
    }

    fn error(&self, msg: String, field: Option<Field>) -> Result<(),Error> {
        let first_instruction = cmp::max(self.pc as i32 - 4, 0) as usize;
        let last_instruction = cmp::min(self.pc + 4, self.instructions.len());
        let mut stack: Vec<String> = Vec::new();
        for i in first_instruction..last_instruction {
            let mut assembled = self.instructions[i].assemble();
            if i == self.pc {
                match &field {
                    Some(f) => {
                        assembled.push_str(format!(" <-- error occurred here, operand: {}", f.to_string()).as_str());
                    },
                    None => {
                        assembled.push_str(" <-- error occurred here");
                    }
                }

            }
            stack.push(format!("{}\t | {}", i, assembled));
        }
        let app_stack = self.stack.to_vec().clone();
        let mut new_app_stack: Vec<String> = Vec::new();
        for i in 0..app_stack.len() {
            new_app_stack.push(format!("{}\t: {}", i, app_stack[i]))
        }
        Err(Error::new(msg, stack, new_app_stack))
    }

    fn jump_to_label(&self, operand: Field, labels: &HashMap<String,usize>) -> Result<usize, Error> {
        let label = self.check_str(operand)?;
        let new_pc = labels.get(&label);
        return match new_pc {
            Some(n) => {
                Ok(*n)
            },
            None => {
                Err(Error::new("Cannot find label.".to_string(), vec![], vec![]))
            }
        }
    }

    fn pop_operand(&mut self, operand: &mut Stack<Field>) -> Result<Field, Error> {
        let item = operand.pop();
        match item {
            Some(i) => Ok(i),
            None => {
                let err = self.error("Cannot pop empty operand stack.".to_string(), None);
                Err(err.err().unwrap())
            }
        }
    }

    fn pop_stack(&mut self) -> Result<Field, Error> {
        let item = self.stack.pop();
        match item {
            Some(i) => Ok(i),
            None => {
                let err = self.error("Cannot pop empty stack.".to_string(), None);
                Err(err.err().unwrap())
            }
        }
    }

    fn pop_call_stack(&mut self) -> Result<usize, Error> {
        let item = self.call_stack.pop();
        match item {
            Some(u) => Ok(u),
            None => {
                let err = self.error("Cannot pop empty call stack.".to_string(), None);
                Err(err.err().unwrap())
            }
        }
    }

    fn check_int(&self, operand: Field) -> Result<i64, Error> {
        let item = operand.to_i();
        match item {
            Some(i) => Ok(i),
            None => {
                let err = self.error("Cannot parse as integer!".to_string(), Some(operand));
                Err(err.err().unwrap())
            }
        }
    }

    fn check_usize(&self, operand: Field) -> Result<usize, Error> {
        let item = operand.to_u();
        match item {
            Some(u) => Ok(u),
            None => {
                let err = self.error("Cannot parse as usize!".to_string(), Some(operand));
                Err(err.err().unwrap())
            }
        }
    }

    fn check_str(&self, operand: Field) -> Result<String, Error> {
        let item = operand.to_s();
        match item {
            Some(s) => Ok(s),
            None => {
                let err = self.error("Cannot parse as string!".to_string(), Some(operand));
                Err(err.err().unwrap())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_push() -> Result<(),Error> {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)])
        ], HashMap::new());

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 4 as i64);
        Ok(())
    }

    #[test]
    fn test_pop() -> Result<(),Error>  {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Pop, vec![])
        ], HashMap::new());

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_add() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Add, vec![])
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 9);
        Ok(())
    }

    #[test]
    fn test_mul() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
            Instruction::new(OpCode::Mul, vec![])
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 20);
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Sub, vec![])
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 7);
        Ok(())
    }

    #[test]
    fn test_div() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(12)]),
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Div, vec![])
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 4);
        Ok(())
    }

    #[test]
    fn test_mod() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(13)]),
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Mod, vec![])
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 1);
        Ok(())
    }

    #[test]
    fn test_print() -> Result<(),Error>  {
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(3)]),
            Instruction::new(OpCode::Print, vec![])
        ], HashMap::new());

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 1);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Call, vec![Field::from("@func")]),
            Instruction::new(OpCode::Push, vec![Field::from("should be on stack")]),
        ], hashmap);

        assert_eq!(vm.pop_stack()?.to_str().unwrap(), "should be on stack");
        Ok(())
    }

    #[test]
    fn test_ret() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 2);
        hashmap.insert("@end".to_string(), 5);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Call, vec![Field::from("@func")]),
            Instruction::new(OpCode::Jmp, vec![Field::from("@end")]),
            Instruction::new(OpCode::Push, vec![Field::from("test")]),
            Instruction::new(OpCode::Pop, vec![]),
            Instruction::new(OpCode::Ret, vec![]),
            Instruction::new(OpCode::Push, vec![Field::from("should be on stack")]),
        ], hashmap);

        assert_eq!(vm.pop_stack()?.to_str().unwrap(), "should be on stack");
        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_label() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@end".to_string(), 2);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Jmp, vec![Field::from("@end")]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_jmp() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@end".to_string(), 2);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Jmp, vec![Field::from("@end")]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_je() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Je, vec![Field::from("@equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)]),
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Push, vec![Field::from(2)]),
            Instruction::new(OpCode::Je, vec![Field::from("@equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jne() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(2)]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Jne, vec![Field::from("@notequal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 4);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Push, vec![Field::from(1)]),
            Instruction::new(OpCode::Jne, vec![Field::from("@notequal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jle() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jle, vec![Field::from("@less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jle, vec![Field::from("@equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jle, vec![Field::from("@less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jge() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jge, vec![Field::from("@greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jge, vec![Field::from("@equal")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jge, vec![Field::from("@greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jl() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jl, vec![Field::from("@less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jl, vec![Field::from("@less")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jg() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Jg, vec![Field::from("@greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(4)]),
            Instruction::new(OpCode::Push, vec![Field::from(7)]),
            Instruction::new(OpCode::Jg, vec![Field::from("@greater")]),
            Instruction::new(OpCode::Push, vec![Field::from(5)])
        ], hashmap);


        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_dup() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Dup, vec![]),
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 10);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 10);
        Ok(())
    }

    #[test]
    fn test_inc() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Inc, vec![]),
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 11);
        Ok(())
    }

    #[test]
    fn test_dec() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Dec, vec![]),
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 9);
        Ok(())
    }

    #[test]
    fn test_swap() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Push, vec![Field::from(20)]),
            Instruction::new(OpCode::Swap, vec![]),
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 10);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 20);
        Ok(())
    }

    #[test]
    fn test_concat() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            Instruction::new(OpCode::Push, vec![Field::from(10)]),
            Instruction::new(OpCode::Push, vec![Field::from(20)]),
            Instruction::new(OpCode::Concat, vec![]),
        ], HashMap::new());

        assert_eq!(vm.pop_stack()?.to_str().unwrap(), "1020");
        Ok(())
    }

    fn create_vm(instructions: Vec<Instruction>, labels: HashMap<String, usize>) -> Vm {
        let mut program = Program::new();
        program.instructions = instructions;
        program.labels = labels;
        let mut vm = Vm::new();
        vm.execute(program);
        vm
    }
}