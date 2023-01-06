use std::{cmp, io};
use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::field::{CastType, Field};
use std::collections::HashMap;
use crate::vm::error::Error;
use crate::vm::program::Program;
use crate::vm::heap::Heap;
use std::io::Write;
use crate::vm::register::{OffsetOperand, Register, Registers};
use crate::vm::stack::Stack;

#[derive(Debug)]
pub struct Vm {
    instructions: Vec<Instruction>,
    labels: HashMap<String,usize>,
    data: HashMap<String, Field>,
    pub registers: Registers,
    stack: Stack<Field>,
    call_stack: Stack<usize>,
    pc: usize,
    pub heap: Heap,
    reflection: bool
}

impl Vm {
    pub fn new(reflection: bool) -> Self {
        Vm{
            instructions: vec![],
            labels: HashMap::new(),
            data: HashMap::new(),
            registers: Registers::new(),
            stack: Stack::new(),
            call_stack: Stack::new(),
            pc: 0,
            heap: Heap::new(),
            reflection
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.heap.clear();

        while self.stack.len() > 0 {
            self.stack.pop();
        }

        while self.call_stack.len() > 0 {
            self.call_stack.pop();
        }
    }

    pub fn execute(&mut self, program: Program) -> Result<(), Error> {
        self.instructions = program.instructions;
        self.labels = program.labels;
        self.data = program.data;

        while (self.pc as usize) < self.instructions.len() {
            let tmp_ins = &self.instructions[self.pc as usize];
            let mut instruction = tmp_ins.clone();
            match instruction.opcode {
                OpCode::Move => {
                    let data = self.pop_operand(&mut instruction.operand)?;
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (r, operand) = register.to_r(&self)?;
                    match &data {
                        &Field::C(c) if operand != OffsetOperand::Default => {
                            Registers::set_offset_for_p(self, r, operand, Field::from(c))?;
                        }
                        Field::S(s) if operand != OffsetOperand::Default => {
                            if self.data.contains_key(s.as_str()) {
                                self.registers.set(r, self.data.get(s.as_str()).unwrap().clone());
                            } else {
                                if s.len() == 1 {
                                    Registers::set_offset_for_p(self, r, operand, Field::from(s.as_str()))?;
                                } else {
                                    return self.error(format!("Cannot find symbol '{}' at {}!", s, self.pc), Some(vec![data]));
                                }
                            }
                        }
                        Field::S(s) => {
                            if self.data.contains_key(s.as_str()) {
                                self.registers.set(r, self.data.get(s.as_str()).unwrap().clone());
                            } else {
                                if s.len() == 1 {
                                    let char = s.chars().nth(0).unwrap();
                                    self.registers.set(r, Field::from(char));
                                } else {
                                    return self.error(format!("Cannot find symbol '{}' at {}!", s, self.pc), Some(vec![data]));
                                }
                            }
                        }
                        &Field::I(i) if operand != OffsetOperand::Default => {
                            Registers::set_offset_for_p(self, r, operand, Field::from(i))?;
                        }
                        &Field::U(i) if operand != OffsetOperand::Default => {
                            Registers::set_offset_for_p(self, r, operand, Field::from(i))?;
                        }
                        Field::R(r2) => {
                            self.registers.set(r, self.registers.get(r2.clone()).clone());
                        }
                        _ => self.registers.set(r, data)
                    }
                }
                OpCode::Push => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    match register {
                        Field::R(r) => {
                            self.stack.push(self.registers.get(r).clone())
                        }
                        Field::S(s) => {
                            if self.data.contains_key(s.as_str()) {
                                self.stack.push(self.data.get(s.as_str()).unwrap().clone());
                            }
                        }
                        _ => {
                            return self.error(format!("Cannot push datatype to stack at {}!", self.pc), Some(vec![register]));
                        }
                    }
                }
                OpCode::Pop => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (register, _) = register.to_r(&self)?;
                    let data = self.pop_stack()?;
                    self.registers.set(register, data).clone()
                }
                OpCode::Add => {
                    let (register, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.set(register, Field::from(i1 + i2));
                }
                OpCode::Mul => {
                    let (register, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.set(register, Field::from(i1 * i2));
                }
                OpCode::Sub => {
                    let (register, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.set(register, Field::from(i2 - i1));
                }
                OpCode::Div => {
                    let (register, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.set(register, Field::from(i2 / i1));
                }
                OpCode::Mod => {
                    let (register, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.set(register, Field::from(i2 % i1));
                }
                OpCode::Print | OpCode::Println => {
                    let field = self.pop_operand(&mut instruction.operand)?;
                    let output = if let Ok((r, offset_type)) = field.to_r(&self) {
                        // get the value from the register.
                        let p = self.registers.get(r);
                        // box field, unbox and get offset?
                        if let Ok(po) = p.to_p(&self) {
                            if !self.heap.contains(po) {
                                return self.error("Cannot set offset for allocation because memory has already been freed!".to_string(), Some(vec![p.clone()]));
                            }
                            let b = Heap::deref(po);
                            let field = if offset_type == OffsetOperand::Default {
                                let mut output = String::default();
                                for item in b.iter() {
                                    output.push(char::from_u32(*item as u32).unwrap());
                                }
                                Field::from(output)
                            } else {
                                let number = offset_type.resolve(self)?;
                                let b_offset = b[number].clone();
                                Field::from(b_offset)
                            };
                            Heap::reref(b);
                            field
                        } else {
                            match p {
                                Field::S(s) => {
                                    if offset_type != OffsetOperand::Default {
                                        let offset = offset_type.resolve(self)?;
                                        Field::from(s.chars().collect::<Vec<char>>()[offset])
                                    } else {
                                        p.clone()
                                    }
                                }
                                _ => p.clone()
                            }
                        }
                    } else {
                        field.clone()
                    };

                    if instruction.opcode == OpCode::Println {
                        println!("{}", output);
                    } else {
                        print!("{}", output);
                    }

                    let _ = io::stdout().flush();
                }
                OpCode::Input => {
                    let input = self.get_input();
                    self.stack.push(Field::from(input));
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
                OpCode::Test => {
                    let (_, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.reset_flags();
                    if i1 == i2 {
                        self.registers.set_equals_flag(true);
                    }
                    if i1 < i2 {
                        self.registers.set_less_than_flag(true);
                    }
                    if i1 > i2 {
                        self.registers.set_greater_than_flag(true);
                    }
                }
                OpCode::Jmp => {
                    let operand = self.pop_operand(&mut instruction.operand)?;
                    let result = self.jump_to_label(operand.clone(), &self.labels)?;
                    self.pc = result;
                    continue;
                }
                OpCode::Je => {
                    if self.registers.check_equals_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jne => {
                    if !self.registers.check_equals_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jl => {
                    if self.registers.check_less_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jg => {
                    if self.registers.check_greater_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jle => {
                    if self.registers.check_equals_flag() || self.registers.check_less_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jge => {
                    if self.registers.check_equals_flag() || self.registers.check_greater_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand.clone(), &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Inc => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (register, _) = register.to_r(&self)?;
                    let v1 = self.registers.get(register).clone();
                    match v1 {
                        Field::I(mut i) => {
                            i += 1;
                            self.registers.set(register, Field::from(i));
                        }
                        Field::U(mut u) => {
                            u += 1;
                            self.registers.set(register, Field::from(u));
                        }
                        Field::C(c) => {
                            let new_char: char = (c as u8 + 1) as char;
                            self.registers.set(register, Field::from(new_char));
                        }
                        _ => {
                            return self.error(format!("Cannot decrement non-int type at {}!", self.pc), Some(vec![v1]));
                        }
                    }
                }
                OpCode::Dec => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (register, _) = register.to_r(&self)?;
                    let v1 = self.registers.get(register).clone();
                    match v1 {
                        Field::I(mut i) => {
                            i -= 1;
                            self.registers.set(register, Field::from(i));
                        }
                        Field::U(mut u) => {
                            u -= 1;
                            self.registers.set(register, Field::from(u));
                        }
                        Field::C(c) => {
                            let new_char: char = (c as u8 - 1) as char;
                            self.registers.set(register, Field::from(new_char));
                        }
                        _ => {
                            return self.error(format!("Cannot decrement non-int type at {}!", self.pc), Some(vec![v1]));
                        }
                    }
                }
                OpCode::Xor => {
                    let (_, i1, i2) = self.get_fields_from_registers_or_data(&mut instruction)?;
                    self.registers.set(Register::Rd, Field::from(i1 ^ i2));
                }
                OpCode::Dup => {
                    let v1 = self.pop_stack()?;
                    // push to the stack twice.
                    self.stack.push(v1.clone());
                    self.stack.push(v1);
                }
                OpCode::Alloc => {
                    let to_alloc = self.pop_operand(&mut instruction.operand)?;
                    let allocation_size = match &to_alloc {
                        Field::R(r) => {
                            let value = self.registers.get(r.clone());
                            value.to_i_or_u(&self)?
                        },
                        Field::U(u) => *u,
                        Field::I(i) => *i as usize,
                        Field::S(s) => {
                            let key = s.as_str();
                            if self.data.contains_key(key) {
                                self.data.get(key).unwrap().to_u(&self)?
                            } else {
                                return self.error(format!("Cannot parse '{}' as size for allocation!", key), Some(vec![to_alloc]));
                            }
                        }
                        _ => {
                            return self.error(format!("Cannot use for allocation!", ), Some(vec![to_alloc]));
                        }
                    };

                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (register, _) = register.to_r(&self)?;

                    let allocated = self.allocate_heap(allocation_size)?;
                    self.registers.set(register, allocated);
                }
                OpCode::Free => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (r, _) = register.to_r(&self)?;
                    let field = self.registers.get(r);
                    self.free_heap(field.to_p(&self)?);
                }
                OpCode::Cast => {
                    let data = self.pop_operand(&mut instruction.operand)?;
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let (r, ro) = register.to_r(&self)?;
                    let field = self.registers.get(r);
                    let output = field.clone().cast(&self, CastType::from(data.to_s(&self)?.as_str()), ro)?;
                    self.registers.set(r, output);
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
            if self.reflection {
                self.registers.set_stack_len(Field::from(self.stack.len()));
                self.registers.set_call_stack_len(Field::from(self.call_stack.len()));
                self.registers.set_pc(Field::from(self.pc));
            }
        }
        Ok(())
    }

    pub fn error(&self, msg: String, field: Option<Vec<Field>>) -> Result<(),Error> {
        let first_instruction = cmp::max(self.pc as i32 - 4, 0) as usize;
        let last_instruction = cmp::min(self.pc + 4, self.instructions.len());
        let mut stack: Vec<String> = Vec::new();
        for i in first_instruction..last_instruction {
            let mut assembled = self.instructions[i].assemble();
            if i == self.pc {
                match &field {
                    Some(f) => {
                        assembled.push_str(format!(" <-- error occurred here, operand(s): ").as_str());
                        for item in f {
                            match item {
                                Field::C(c) => {
                                    assembled.push_str(format!("{}", c).as_str());
                                }
                                Field::I(i) => {
                                    assembled.push_str(format!("{} ", i).as_str());
                                }
                                Field::U(u) => {
                                    assembled.push_str(format!("{:#04x} ", u).as_str());
                                }
                                Field::S(s) => {
                                    if s.len() == 0 {
                                        continue;
                                    }
                                    assembled.push_str(format!("{} ", s).as_str());
                                },
                                _ => {
                                    assembled.push_str(format!("{} ", item.to_string()).as_str());
                                }
                            }
                        }
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
        let label = operand.to_s(&self)?;
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

    fn allocate_heap(&mut self, size: usize) -> Result<Field, Error> {
        Ok(Field::P(self.heap.allocate(size)))
    }

    fn free_heap(&mut self, var: *mut [usize]) {
        unsafe { self.heap.free(var) }
    }

    fn get_input(&self) -> String{
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {},
            Err(_no_updates_is_fine) => {},
        }
        input.trim().to_string()
    }

    fn get_usize_from_register(&mut self, register: &Field) -> Result<usize, Error> {
        let r1 = register.to_r(&self);
        match r1 {
            Ok((r, _)) => {
                let register1_value = self.registers.get(r).clone();
                Ok(register1_value.to_i_or_u(&self)?)
            },
            Err(_) => {
                let key = register.to_str(&self);
                match key {
                    Ok(k) => if self.data.contains_key(k) {
                        Ok(self.data.get(k).unwrap().to_i_or_u(&self)?)
                    } else {
                        return Err(self.error(format!("Operand '{}' is not valid here.", k), Some(vec![register.clone()])).unwrap_err());
                    }
                    Err(_) => Ok(register.to_i_or_u(&self)?)
                }
            }
        }
    }

    fn get_fields_from_registers_or_data(&mut self, instruction: &mut Instruction) -> Result<(Register, usize, usize), Error> {
        let register1 = self.pop_operand(&mut instruction.operand)?;
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let u1 = self.get_usize_from_register(&register1)?;
        let u2 = self.get_usize_from_register(&register2)?;
        return Ok((register2.to_r(&self)?.0, u1, u2));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_mov() -> Result<(),Error> {
        let mut hm = HashMap::new();
        hm.insert("uhoh".to_string(), Field::from("Uh OH!"));
        let vm = create_vm_with_data(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Register::Ra.into()]),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Register::Rb.into()]),
            ins_vec(OpCode::Move, vec![Register::Rd.into(), Field::S("uhoh".to_string())]),
        ], None, hm)?;

        assert_eq!(vm.registers.ra, Field::I(4));
        assert_eq!(vm.registers.rb, Field::I(4));
        assert_eq!(vm.registers.rc, Field::I(4));
        assert_eq!(vm.registers.rd, Field::S("Uh OH!".to_string()));
        Ok(())
    }

    #[test]
    fn test_push() -> Result<(),Error> {
        let mut vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins(OpCode::Push, Register::Ra)
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 4);
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i_or_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_pop() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins(OpCode::Push, Register::Ra),
            ins(OpCode::Pop, Register::Rb)
        ], None)?;

        assert_eq!(vm.stack.len(), 0);
        assert_eq!(vm.registers.rb.to_i_or_u(&vm)?, 4);
        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_add() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(5)]),
            ins_vec(OpCode::Add, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 9);

        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Add, vec![Register::Ra.into(), Field::from(12)])
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 16);
        Ok(())
    }

    #[test]
    fn test_mul() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(5)]),
            ins_vec(OpCode::Mul, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 20);
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(10)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
            ins_vec(OpCode::Sub, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 7);
        Ok(())
    }

    #[test]
    fn test_div() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(12)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
            ins_vec(OpCode::Div, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_mod() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
            ins_vec(OpCode::Mod, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 1);
        Ok(())
    }

    #[test]
    fn test_print() -> Result<(),Error>  {
        let _ = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(69)]),
            ins(OpCode::Print, Register::Ra)
        ], None)?;

        Ok(())
    }

    #[test]
    fn test_call() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 2);
        let mut vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins(OpCode::Call, "@func"),
            ins(OpCode::Push, Register::Ra),
        ], Some(hashmap))?;

        assert_eq!(vm.pop_stack()?.to_i_or_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_ret() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 3);
        hashmap.insert("@end".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins(OpCode::Call, "@func"),
            ins(OpCode::Jmp, "@end"),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(9)]),
            ins_e(OpCode::Ret),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(8)]),
        ], Some(hashmap))?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 4);
        assert_eq!(vm.registers.rb.to_i_or_u(&vm)?, 9);
        assert_eq!(vm.registers.rc.to_i_or_u(&vm)?, 8);
        Ok(())
    }

    #[test]
    fn test_label() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@end".to_string(), 2);
        let vm = create_vm(vec![
            ins(OpCode::Jmp, "@end"),
            ins(OpCode::Push, 1),
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_jmp() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@end".to_string(), 2);
        let vm = create_vm(vec![
            ins(OpCode::Jmp, "@end"),
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_je() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 6);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(13)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Je, "@equal"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(13)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Je, "@equal"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_eq!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jne() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 6);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(13)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Je, "@notequal"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Je, "@notequal"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_eq!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jle() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 6);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jle, "@less"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jle, "@equal"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;
        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jle, "@less"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;
        assert_eq!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jge() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 6);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jge, "@greater"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jge, "@equal"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;
        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jge, "@greater"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;
        assert_eq!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jl() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 6);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jl, "@less"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jl, "@less"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;
        assert_eq!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jg() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 6);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jge, "@greater"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;

        assert_ne!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 5);
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ins(OpCode::Jge, "@greater"),
            ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
        ], Some(hashmap))?;
        assert_eq!(vm.registers.get(Register::Rc).to_i_or_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_dup() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
            ins(OpCode::Push, Register::Ra),
            ins_e(OpCode::Dup),
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i_or_u(&vm)?, 4);
        assert_eq!(vm.pop_stack()?.to_i_or_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_inc() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(10)]),
            ins(OpCode::Inc, Register::Ra)
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 11);
        Ok(())
    }

    #[test]
    fn test_dec() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(10)]),
            ins(OpCode::Dec, Register::Ra)
        ], None)?;

        assert_eq!(vm.registers.ra.to_i_or_u(&vm)?, 9);
        Ok(())
    }

    #[test]
    fn test_xor() -> Result<(),Error> {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(10)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(10)]),
            ins_vec(OpCode::Xor, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.rd.to_i_or_u(&vm)?, 0);

        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(100)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(10)]),
            ins_vec(OpCode::Xor, vec![Register::Ra.into(), Register::Rb.into()])
        ], None)?;

        assert_eq!(vm.registers.rd.to_i_or_u(&vm)?, 110);
        Ok(())
    }

    #[test]
    fn test_test() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(5)]),
            ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
            ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
        ], None)?;

        assert_eq!(vm.registers.check_greater_than_flag(), true);
        assert_eq!(vm.registers.check_equals_flag(), false);
        assert_eq!(vm.registers.check_less_than_flag(), false);
        Ok(())
    }

    #[test]
    fn test_alloc() -> Result<(),Error> {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(5)]),
            ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from(6)]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from(12)]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from(18)]),
        ], None)?;

        let ptr = vm.registers.rd.to_p(&vm)?;
        let boxed = unsafe {Box::from_raw(ptr)};
        assert_eq!(boxed.len(), 5);
        assert_eq!(boxed[0], 6);
        assert_eq!(boxed[1], 12);
        assert_eq!(boxed[2], 18);

        Ok(())
    }

    #[test]
    fn test_free() -> Result<(),Error> {
        let mut vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(3)]),
            ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from('h')]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from('e')]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from('y')]),
            ins(OpCode::Println, Register::Rd),
        ], None)?;

        assert_eq!(vm.heap.len(), 1);

        let mut prog = Program::new();
        prog.instructions = vec![ins(OpCode::Free, Register::Rd)];
        vm.pc = 0;
        let _ = vm.execute(prog);
        assert_eq!(vm.heap.len(), 0);

        Ok(())
    }

    #[test]
    fn test_freed_error() -> Result<(),Error> {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(3)]),
            ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from('h')]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from('e')]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from('y')]),
            ins(OpCode::Println, Register::Rd),
            ins(OpCode::Free, Register::Rd),
            ins(OpCode::Println, Register::Rd),
        ], None);

        assert!(vm.is_err());
        let err = vm.unwrap_err();
        assert_eq!(err.message, "Cannot set offset for allocation because memory has already been freed!");

        Ok(())
    }

    #[test]
    fn test_clear() -> Result<(),Error> {
        let mut vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(3)]),
            ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from('h')]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from('e')]),
            ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from('y')]),
        ], None)?;

        assert_eq!(1, vm.heap.len());
        let allocations = vm.heap.get_allocations();
        let real_allocation = {
            let (first_allocation, _) = allocations.iter().nth(0).unwrap();
            first_allocation.clone()
        };

        let derefed = unsafe {Box::from_raw(real_allocation)};
        assert_eq!(vec!['h' as usize,'e' as usize,'y' as usize], derefed.to_vec());
        let _ = Box::into_raw(derefed);
        vm.heap.clear();
        assert_eq!(0, vm.heap.len());

        Ok(())
    }

    #[test]
    fn test_cast() -> Result<(),Error> {
        let vm = create_vm(vec![
            ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(100)]),
            ins_vec(OpCode::Cast, vec![Register::Rf.into(), Field::from("char")]),
        ], None)?;

        let field = vm.registers.rf;
        assert_eq!(field, Field::C('d'));

        Ok(())
    }

    fn ins<T>(opcode: OpCode, item: T) -> Instruction where Field: From<T> {
        Instruction::new(opcode, vec![Field::from(item)])
    }

    fn ins_vec(opcode: OpCode, items: Vec<Field>) -> Instruction {
        Instruction::new(opcode, items)
    }

    fn ins_e(opcode: OpCode) -> Instruction {
        Instruction::new(opcode, vec![])
    }

    fn create_vm_with_data(instructions: Vec<Instruction>, labels: Option<HashMap<String, usize>>, data: HashMap<String, Field>) -> Result<Vm,Error> {
        let mut vm = Vm::new(true);
        execute(&mut vm, instructions, labels, Some(data))?;
        Ok(vm)
    }

    fn create_vm(instructions: Vec<Instruction>, labels: Option<HashMap<String, usize>>) -> Result<Vm,Error> {
        let mut vm = Vm::new(true);
        execute(&mut vm, instructions, labels, None)?;
        Ok(vm)
    }

    fn execute(vm: &mut Vm, instructions: Vec<Instruction>, labels: Option<HashMap<String, usize>>, data: Option<HashMap<String, Field>>) -> Result<(),Error> {
        let mut program = Program::new();
        program.instructions = instructions;

        if labels.is_some() {
            program.labels = labels.unwrap();
        }

        if data.is_some() {
            program.data = data.unwrap();
        }

        vm.execute(program)?;

        Ok(())
    }
}