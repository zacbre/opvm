use crate::types::{Allocation, Type};
use crate::vm::error::Error;
use crate::vm::field::Field;
use crate::vm::heap::Heap;
use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::program::Program;
use crate::vm::register::Registers;
use crate::vm::stack::Stack;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{cmp, io};

use super::builtin::{self, BuiltIn};
use super::register::{RegisterOffsetOperandType, RegisterWithOffset};

#[derive(Debug)]
pub struct Vm {
    builtins: Vec<Box<dyn BuiltIn>>,
    instructions: Vec<Instruction>,
    labels: HashMap<String, usize>,
    data: HashMap<String, Field>,
    pub registers: Registers,
    stack: Stack<Field>,
    call_stack: Stack<usize>,
    pc: usize,
    pub heap: Arc<Mutex<Heap>>,
    reflection: bool,
}

impl Vm {
    pub fn new(reflection: bool) -> Self {
        Vm {
            builtins: vec![
                Box::new(builtin::Println),
                Box::new(builtin::Print),
                Box::new(builtin::Concat),
                Box::new(builtin::DateNowUnix),
                Box::new(builtin::DateNow),
                Box::new(builtin::Dbg),
                Box::new(builtin::DbgPtr),
                Box::new(builtin::Random),
                Box::new(builtin::MathFloor),
            ],
            instructions: vec![],
            labels: HashMap::new(),
            data: HashMap::new(),
            registers: Registers::new(),
            stack: Stack::new(),
            call_stack: Stack::new(),
            pc: 0,
            heap: Heap::get(),
            reflection,
        }
    }

    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.heap.lock().unwrap().reset();

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
            // clone operand
            let mut operands: Vec<Field> = vec![];
            for operand in tmp_ins.operand.to_vec() {
                operands.push(Field::from(operand.underlying_data_clone()));
            }
            let mut instruction: Instruction =
                Instruction::new_from_fields(tmp_ins.opcode.into(), operands);
            match instruction.opcode {
                OpCode::Move => {
                    let data = self.pop_operand(&mut instruction.operand)?;
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let r_result = register.to_r(&self);
                    if r_result.is_ok() {
                        let r = r_result.unwrap();
                        match &data {
                            Field(Type::String(s)) => {
                                if self.data.contains_key(s.as_str()) {
                                    self.registers.set(
                                        r,
                                        self.data.get(s.as_str()).unwrap().underlying_data_clone(),
                                    );
                                } else {
                                    if s.len() == 1 {
                                        let char = s.chars().nth(0).unwrap();
                                        self.registers.set(r, Field::from(char));
                                    } else {
                                        return self.error(
                                            format!("Cannot find symbol '{}' at {}!", s, self.pc),
                                            Some(vec![data]),
                                        );
                                    }
                                }
                            }
                            Field(Type::Register(r2)) => {
                                self.registers
                                    .set(r, self.registers.get(*r2).underlying_data_clone());
                            }
                            Field(Type::RegisterWithOffsets(r2)) => {
                                let source_data = self.get_source_data(r2)?;
                                self.registers.set(r, source_data);
                            }
                            _ => self.registers.set(r, data),
                        }
                    } else {
                        // get register with offset.
                        let rwo = register.to_rwo(&self)?;
                        match &data {
                            Field(Type::String(s)) => {
                                if self.data.contains_key(s.as_str()) {
                                    self.set_dest_data(
                                        &rwo,
                                        self.data.get(s.as_str()).unwrap().underlying_data_clone(),
                                    )?;
                                } else {
                                    if s.len() == 1 {
                                        let char = s.chars().nth(0).unwrap();
                                        self.set_dest_data(&rwo, Field::from(char))?;
                                    } else {
                                        return self.error(
                                            format!("Cannot find symbol '{}' at {}!", s, self.pc),
                                            Some(vec![data]),
                                        );
                                    }
                                }
                            }
                            Field(Type::Register(r2)) => {
                                self.set_dest_data(
                                    &rwo,
                                    self.registers.get(*r2).underlying_data_clone(),
                                )?;
                            }
                            Field(Type::RegisterWithOffsets(r2)) => {
                                let source_data = self.get_source_data(r2)?;
                                self.set_dest_data(&rwo, source_data)?;
                            }
                            _ => {
                                self.set_dest_data(&rwo, data)?;
                            }
                        }
                    }
                }
                OpCode::Push => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    match register.0 {
                        Type::Register(r) => self
                            .stack
                            .push(self.registers.get(r).underlying_data_clone()),
                        Type::String(s) => {
                            if self.data.contains_key(s.as_str()) {
                                self.stack.push(
                                    self.data.get(s.as_str()).unwrap().underlying_data_clone(),
                                );
                            }
                        }
                        _ => {
                            return self.error(
                                format!("Cannot push datatype to stack at {}!", self.pc),
                                Some(vec![register]),
                            );
                        }
                    }
                }
                OpCode::Pop => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let register = register.to_r(&self)?;
                    let data = self.pop_stack()?;
                    self.registers.set(register, data).clone()
                }
                OpCode::Add => {
                    self.add(&mut instruction)?;
                }
                OpCode::Mul => {
                    self.mul(&mut instruction)?;
                }
                OpCode::Sub => {
                    self.sub(&mut instruction)?;
                }
                OpCode::Div => {
                    self.div(&mut instruction)?;
                }
                OpCode::Mod => {
                    self.rem(&mut instruction)?;
                }
                OpCode::Input => {
                    let input = self.get_input();
                    self.stack.push(Field::from(input));
                }
                OpCode::Call => {
                    let label = self.pop_operand(&mut instruction.operand)?;
                    if self.labels.contains_key(&label.to_string()) {
                        self.call_stack.push(self.pc + 1);
                        let result = self.jump_to_label(label, &self.labels)?;
                        self.pc = result;
                        continue;
                    } else if self
                        .builtins
                        .iter()
                        .any(|b| b.get_name() == label.to_string())
                    {
                        for func in &self.builtins {
                            if func.get_name() == label.to_string() {
                                let result = func.call(
                                    &mut self.registers,
                                    &mut self.stack,
                                    &mut self.instructions,
                                );
                                self.registers.r0 = result;
                                break;
                            }
                        }
                    } else {
                        self.error(
                            format!("Cannot find label '{}' at {}!", label, self.pc),
                            Some(vec![label]),
                        )?;
                    }
                }
                OpCode::Ret => {
                    self.pc = self.pop_call_stack()?;
                    continue;
                }
                OpCode::Test => {
                    self.test(&mut instruction)?;
                }
                OpCode::Jmp => {
                    let operand = self.pop_operand(&mut instruction.operand)?;
                    let result = self.jump_to_label(operand, &self.labels)?;
                    self.pc = result;
                    continue;
                }
                OpCode::Je => {
                    if self.registers.check_equals_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand, &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jne => {
                    if !self.registers.check_equals_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand, &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jl => {
                    if self.registers.check_less_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand, &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jg => {
                    if self.registers.check_greater_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand, &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jle => {
                    if self.registers.check_equals_flag() || self.registers.check_less_than_flag() {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand, &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Jge => {
                    if self.registers.check_equals_flag()
                        || self.registers.check_greater_than_flag()
                    {
                        let operand = self.pop_operand(&mut instruction.operand)?;
                        let result = self.jump_to_label(operand, &self.labels)?;
                        self.pc = result;
                        continue;
                    }
                }
                OpCode::Xor => {
                    self.xor(&mut instruction)?;
                }
                OpCode::Dup => {
                    let v1 = self.pop_stack()?;
                    // push to the stack twice.
                    self.stack.push(v1.underlying_data_clone());
                    self.stack.push(v1);
                }
                OpCode::Alloc => {
                    let to_alloc = self.pop_operand(&mut instruction.operand)?;
                    let allocation_size = match &to_alloc.0 {
                        Type::Register(r) => {
                            let value = self.registers.get(r.clone());
                            value.to_u(&self)?
                        }
                        Type::UInt(u) => *u,
                        Type::Int(i) => *i as usize,
                        Type::String(s) => {
                            let key = s.as_str();
                            if self.data.contains_key(key) {
                                self.data.get(key).unwrap().to_u(&self)?
                            } else {
                                return self.error(
                                    format!("Cannot parse '{}' as size for allocation!", key),
                                    Some(vec![to_alloc]),
                                );
                            }
                        }
                        _ => {
                            return self.error(
                                format!("Cannot use for allocation!",),
                                Some(vec![to_alloc]),
                            );
                        }
                    };

                    let register = self.pop_operand(&mut instruction.operand)?;
                    let register = register.to_r(&self)?;

                    let allocated = self.allocate_heap(allocation_size)?;
                    self.registers.set(register, allocated);
                }
                OpCode::Free => {
                    let register = self.pop_operand(&mut instruction.operand)?;
                    let register = register.to_r(&self)?;
                    let field = self.registers.get(register).underlying_data_clone();
                    let p = field.to_p(&self)?;
                    self.free_heap(&p)?;
                }
                OpCode::Load => {}
                OpCode::Store => {}
                OpCode::Nop => (),
                OpCode::Hlt => {
                    return Ok(());
                }
                OpCode::Igl => {
                    return self.error(
                        format!("ILLEGAL instruction encountered at {}.", self.pc),
                        None,
                    );
                }
                OpCode::Assert => {
                    self.test(&mut instruction)?;
                    if !self.registers.check_equals_flag() {
                        return self.error(
                            format!("Assertion failed at {}.", self.pc),
                            None
                        );
                    }
                    self.registers.reset_flags();
                }
            }
            self.pc += 1;
            if self.reflection {
                self.registers.set_stack_len(Field::from(self.stack.len()));
                self.registers
                    .set_call_stack_len(Field::from(self.call_stack.len()));
                self.registers.set_pc(Field::from(self.pc));
            }
        }
        Ok(())
    }

    pub fn error(&self, msg: String, field: Option<Vec<Field>>) -> Result<(), Error> {
        let first_instruction = cmp::max(self.pc as i32 - 4, 0) as usize;
        let last_instruction = cmp::min(self.pc + 4, self.instructions.len());
        let mut stack: Vec<String> = Vec::new();
        for i in first_instruction..last_instruction {
            let mut assembled = self.instructions[i].assemble();
            if i == self.pc {
                match &field {
                    Some(f) => {
                        assembled
                            .push_str(format!(" <-- error occurred here, operand(s): ").as_str());
                        for item in f {
                            match &item.0 {
                                Type::Char(c) => {
                                    assembled.push_str(format!("{} ", c).as_str());
                                }
                                Type::Int(i) => {
                                    assembled.push_str(format!("{} ", i).as_str());
                                }
                                Type::UInt(u) => {
                                    assembled.push_str(format!("{:#04x} ", u).as_str());
                                }
                                Type::String(s) => {
                                    if s.len() == 0 {
                                        continue;
                                    }
                                    assembled.push_str(format!("{} ", s).as_str());
                                }
                                Type::Register(r) => {
                                    assembled.push_str(format!("{},", r.to_string()).as_str());
                                }
                                _ => {
                                    assembled.push_str(format!("{} ", item.to_string()).as_str());
                                }
                            }
                        }
                    }
                    None => {
                        assembled.push_str(" <-- error occurred here");
                    }
                }
            }
            stack.push(format!("{}\t | {}", i, assembled));
        }
        let app_stack = self.stack.to_vec();
        let mut new_app_stack: Vec<String> = Vec::new();
        for i in 0..app_stack.len() {
            new_app_stack.push(format!("{}\t: {}", i, app_stack[i]))
        }
        Err(Error::new(msg, stack, new_app_stack))
    }

    fn jump_to_label(
        &self,
        operand: Field,
        labels: &HashMap<String, usize>,
    ) -> Result<usize, Error> {
        let label = operand.to_string();
        let new_pc = labels.get(&label);
        return match new_pc {
            Some(n) => Ok(*n),
            None => Err(Error::new(
                format!("Cannot find label '{}'.", label),
                vec![],
                vec![],
            )),
        };
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
        let mut heap = Heap::recover_poison(&self.heap);
        let ptr = heap.allocate(size).map_err(|_| {
            self.error(
                format!("Cannot allocate heap at {}!", self.pc),
                Some(vec![Field::from(size)]),
            )
            .unwrap_err()
        })?;
        let allocation = Allocation::new(ptr, size, 64);
        Ok(Field(Type::Pointer(allocation)))
    }

    fn free_heap(&mut self, allocation: &Allocation) -> Result<(), Error> {
        let mut heap = Heap::recover_poison(&self.heap);
        heap.deallocate(allocation.ptr, allocation.size)
            .map_err(|_| {
                self.error(
                    format!("Cannot free heap at {}!", self.pc),
                    Some(vec![Field::from(allocation.ptr.as_ptr() as usize)]),
                )
                .unwrap_err()
            })
    }

    fn get_input(&self) -> String {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        input.trim().to_string()
    }

    fn test(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let i1 = self.registers.get(r).underlying_data_clone();

        let i2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

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
        Ok(())
    }

    fn add(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let r1_data = self.registers.get(r).underlying_data_clone();

        let data2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

        self.registers.set(r, r1_data + data2);

        Ok(())
    }

    fn sub(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let r1_data = self.registers.get(r).underlying_data_clone();

        let data2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

        self.registers.set(r, r1_data - data2);

        Ok(())
    }

    fn mul(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let r1_data = self.registers.get(r).underlying_data_clone();

        let data2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

        self.registers.set(r, r1_data * data2);

        Ok(())
    }

    fn div(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let r1_data = self.registers.get(r).underlying_data_clone();

        let data2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

        self.registers.set(r, r1_data / data2);

        Ok(())
    }

    fn rem(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let r1_data = self.registers.get(r).underlying_data_clone();

        let data2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

        self.registers.set(r, r1_data % data2);

        Ok(())
    }

    fn xor(&mut self, instruction: &mut Instruction) -> Result<(), Error> {
        let register2 = self.pop_operand(&mut instruction.operand)?;
        let register1 = self.pop_operand(&mut instruction.operand)?;

        let r = register1.to_r(&self)?;
        let r1_data = self.registers.get(r).underlying_data_clone();

        let data2 = match register2 {
            Field(Type::Register(r)) => self.registers.get(r).underlying_data_clone(),
            _ => register2,
        };

        self.registers.set(r, r1_data ^ data2);

        Ok(())
    }

    fn get_source(&mut self, source: &RegisterWithOffset) -> Result<Field, Error> {
        let mut field = Field::default();
        let mut previous_operand = RegisterOffsetOperandType::None;
        for item in &source.offsets {
            match item.offset {
                Field(Type::Int(_)) => {
                    // get the offset at specified index.
                    previous_operand.apply(&mut field, item.offset.underlying_data_clone());
                }
                Field(Type::Register(rv)) => {
                    let register_value = self.registers.get(rv).underlying_data_clone();
                    previous_operand.apply(&mut field, register_value);
                }
                _ => {
                    return Err(self
                        .error(
                            format!("Cannot use '{}' as offset at {}!", item.offset, self.pc),
                            Some(vec![item.offset.underlying_data_clone()]),
                        )
                        .unwrap_err());
                }
            }
            previous_operand = item.operand.clone();
        }

        Ok(field)
    }

    fn get_source_data(&mut self, source: &RegisterWithOffset) -> Result<Field, Error> {
        let field = self.get_source(source)?;
        // once the offsets have been calculated, let's pull the original item and do something with it
        let register_for_data = self.registers.get(source.register);
        let result = match register_for_data {
            Field(Type::Pointer(p)) => {
                let value = unsafe { p.ptr.as_ptr().offset(field.to_u(&self)? as isize) };
                Field::from(unsafe { value.read() })
            }
            Field(Type::String(s)) => {
                let offset = field.to_u(&self)?;
                Field::from(s[offset..offset + 1].to_string())
            }
            _ => {
                return Err(self
                    .error(
                        format!(
                            "Cannot use '{}' as offset at {}!",
                            register_for_data, self.pc
                        ),
                        Some(vec![register_for_data.underlying_data_clone()]),
                    )
                    .unwrap_err());
            }
        };
        Ok(result)
    }

    fn set_dest_data(&mut self, dest: &RegisterWithOffset, data: Field) -> Result<(), Error> {
        let offset = self.get_source(dest)?;
        let register_for_data = self.registers.get(dest.register);
        match register_for_data {
            Field(Type::Pointer(p)) => {
                let value = unsafe { p.ptr.as_ptr().offset(offset.to_u(&self)? as isize) };
                let mut bytes = data.to_b(&self)?;
                unsafe {
                    let bytes_ptr = bytes.as_mut_ptr();
                    bytes_ptr.copy_to_nonoverlapping(value, bytes.len());
                }
                //self.registers.set(dest.register, Field(Type::Byte(unsafe { value.read() })));
            }
            Field(Type::String(s)) => {
                let offset = offset.to_u(&self)?;
                let new_string = data.to_string();
                let new_value = if s.len() < offset + new_string.len() {
                    format!("{}{}", &s[offset..], new_string)
                } else {
                    format!(
                        "{}{}{}",
                        &s[..offset],
                        new_string,
                        &s[offset + new_string.len()..]
                    )
                };
                self.registers.set(dest.register, Field::from(new_value));
            }
            _ => {
                return self.error(
                    format!(
                        "Cannot use '{}' as offset at {}!",
                        register_for_data, self.pc
                    ),
                    Some(vec![register_for_data.underlying_data_clone()]),
                );
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::vm::register::Register;

    use super::*;

    #[test]
    fn test_mov() -> Result<(), Error> {
        let mut hm = HashMap::new();
        hm.insert("uhoh".to_string(), Field::from("Uh OH!"));
        let vm = create_vm_with_data(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Register::Ra.into()]),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Register::Rb.into()]),
                ins_vec(
                    OpCode::Move,
                    vec![Register::Rd.into(), Field(Type::String("uhoh".to_string()))],
                ),
            ],
            None,
            hm,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 4);
        assert_eq!(vm.registers.rb.to_u(&vm)?, 4);
        assert_eq!(vm.registers.rc.to_u(&vm)?, 4);
        assert_eq!(vm.registers.rd.to_string(), "Uh OH!".to_string());
        Ok(())
    }

    #[test]
    fn test_push() -> Result<(), Error> {
        let mut vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins(OpCode::Push, Register::Ra),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 4);
        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_pop() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins(OpCode::Push, Register::Ra),
                ins(OpCode::Pop, Register::Rb),
            ],
            None,
        )?;

        assert_eq!(vm.stack.len(), 0);
        assert_eq!(vm.registers.rb.to_u(&vm)?, 4);
        assert_eq!(vm.registers.ra.to_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_add() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(5)]),
                ins_vec(OpCode::Add, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 9);

        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Add, vec![Register::Ra.into(), Field::from(12)]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 16);
        Ok(())
    }

    #[test]
    fn test_mul() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(5)]),
                ins_vec(OpCode::Mul, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 20);
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(10)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
                ins_vec(OpCode::Sub, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 7);
        Ok(())
    }

    #[test]
    fn test_div() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(12)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
                ins_vec(OpCode::Div, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_mod() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
                ins_vec(OpCode::Mod, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 1);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 2);
        let mut vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins(OpCode::Call, "@func"),
                ins(OpCode::Push, Register::Ra),
            ],
            Some(hashmap),
        )?;

        assert_eq!(vm.pop_stack()?.to_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_ret() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 3);
        hashmap.insert("@end".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins(OpCode::Call, "@func"),
                ins(OpCode::Jmp, "@end"),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(9)]),
                ins_e(OpCode::Ret),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(8)]),
            ],
            Some(hashmap),
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 4);
        assert_eq!(vm.registers.rb.to_u(&vm)?, 9);
        assert_eq!(vm.registers.rc.to_u(&vm)?, 8);
        Ok(())
    }

    #[test]
    fn test_label() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@end".to_string(), 2);
        let vm = create_vm(
            vec![ins(OpCode::Jmp, "@end"), ins(OpCode::Push, 1)],
            Some(hashmap),
        )?;

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_jmp() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@end".to_string(), 2);
        let vm = create_vm(
            vec![
                ins(OpCode::Jmp, "@end"),
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_je() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 6);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(13)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Je, "@equal"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(13)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Je, "@equal"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_eq!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jne() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 6);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(13)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Je, "@notequal"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(13)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(3)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Je, "@notequal"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_eq!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jle() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 6);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jle, "@less"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jle, "@equal"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;
        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jle, "@less"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;
        assert_eq!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jge() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 6);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jge, "@greater"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jge, "@equal"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;
        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jge, "@greater"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;
        assert_eq!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jl() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("less".to_string(), 6);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jl, "less"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("less".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jl, "less"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;
        assert_eq!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_jg() -> Result<(), Error> {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 6);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(4)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jge, "@greater"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;

        assert_ne!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 5);
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(7)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
                ins(OpCode::Jge, "@greater"),
                ins_vec(OpCode::Move, vec![Register::Rc.into(), Field::from(5)]),
            ],
            Some(hashmap),
        )?;
        assert_eq!(vm.registers.get(Register::Rc).to_u(&vm)?, 5);
        Ok(())
    }

    #[test]
    fn test_dup() -> Result<(), Error> {
        let mut vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(4)]),
                ins(OpCode::Push, Register::Ra),
                ins_e(OpCode::Dup),
            ],
            None,
        )?;

        assert_eq!(vm.pop_stack()?.to_u(&vm)?, 4);
        assert_eq!(vm.pop_stack()?.to_u(&vm)?, 4);
        Ok(())
    }

    #[test]
    fn test_xor() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(10)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(10)]),
                ins_vec(OpCode::Xor, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 0);

        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(100)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(10)]),
                ins_vec(OpCode::Xor, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.ra.to_u(&vm)?, 110);
        Ok(())
    }

    #[test]
    fn test_test() -> Result<(), Error> {
        let vm = create_vm(
            vec![
                ins_vec(OpCode::Move, vec![Register::Ra.into(), Field::from(7)]),
                ins_vec(OpCode::Move, vec![Register::Rb.into(), Field::from(5)]),
                ins_vec(OpCode::Test, vec![Register::Ra.into(), Register::Rb.into()]),
            ],
            None,
        )?;

        assert_eq!(vm.registers.check_greater_than_flag(), true);
        assert_eq!(vm.registers.check_equals_flag(), false);
        assert_eq!(vm.registers.check_less_than_flag(), false);
        Ok(())
    }

    // #[test]
    // fn test_alloc() -> Result<(),Error> {
    //     let vm = create_vm(vec![
    //         ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(5)]),
    //         ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from(6)]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from(12)]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from(18)]),
    //     ], None)?;

    //     let ptr = vm.registers.rd.to_p(&vm)?;
    //     let boxed = unsafe {Box::from_raw(ptr)};
    //     assert_eq!(boxed.len(), 5);
    //     assert_eq!(boxed[0], 6);
    //     assert_eq!(boxed[1], 12);
    //     assert_eq!(boxed[2], 18);

    //     Ok(())
    // }

    // #[test]
    // fn test_free() -> Result<(),Error> {
    //     let mut vm = create_vm(vec![
    //         ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(3)]),
    //         ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from('h')]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from('e')]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from('y')]),
    //         ins(OpCode::Println, Register::Rd),
    //     ], None)?;

    //     assert_eq!(vm.heap.len(), 1);

    //     let mut prog = Program::new();
    //     prog.instructions = vec![ins(OpCode::Free, Register::Rd)];
    //     vm.pc = 0;
    //     let _ = vm.execute(prog);
    //     assert_eq!(vm.heap.len(), 0);

    //     Ok(())
    // }

    // #[test]
    // fn test_freed_error() -> Result<(),Error> {
    //     let vm = create_vm(vec![
    //         ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(3)]),
    //         ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from('h')]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from('e')]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from('y')]),
    //         ins(OpCode::Println, Register::Rd),
    //         ins(OpCode::Free, Register::Rd),
    //         ins(OpCode::Println, Register::Rd),
    //     ], None);

    //     assert!(vm.is_err());
    //     let err = vm.unwrap_err();
    //     assert_eq!(err.message, "Cannot print allocation because memory has already been freed!");

    //     Ok(())
    // }

    // #[test]
    // fn test_clear() -> Result<(),Error> {
    //     let mut vm = create_vm(vec![
    //         ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(3)]),
    //         ins_vec(OpCode::Alloc, vec![Register::Rd.into(), Register::Rf.into()]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(0)), Field::from('h')]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(1)), Field::from('e')]),
    //         ins_vec(OpCode::Move, vec![Field::RO(Register::Rd, OffsetOperand::Number(2)), Field::from('y')]),
    //     ], None)?;

    //     assert_eq!(1, vm.heap.len());
    //     let allocations = vm.heap.get_allocations();
    //     let real_allocation = {
    //         let (first_allocation, _) = allocations.iter().nth(0).unwrap();
    //         first_allocation.clone()
    //     };

    //     let derefed = unsafe {Box::from_raw(real_allocation)};
    //     assert_eq!(vec!['h' as usize,'e' as usize,'y' as usize], derefed.to_vec());
    //     let _ = Box::into_raw(derefed);
    //     vm.heap.clear();
    //     assert_eq!(0, vm.heap.len());

    //     Ok(())
    // }

    // #[test]
    // fn test_cast() -> Result<(),Error> {
    //     let vm = create_vm(vec![
    //         ins_vec(OpCode::Move, vec![Register::Rf.into(), Field::from(100)]),
    //         ins_vec(OpCode::Cast, vec![Register::Rf.into(), Field::from("char")]),
    //     ], None)?;

    //     let field = vm.registers.rf;
    //     assert_eq!(field, Field::C('d'));

    //     Ok(())
    // }

    fn ins<T>(opcode: OpCode, item: T) -> Instruction
    where
        Field: From<T>,
    {
        Instruction::new_from_fields(opcode.into(), vec![Field::from(item)])
    }

    fn ins_vec(opcode: OpCode, items: Vec<Field>) -> Instruction {
        Instruction::new_from_fields(opcode.into(), items)
    }

    fn ins_e(opcode: OpCode) -> Instruction {
        Instruction::new_from_fields(opcode.into(), vec![])
    }

    fn create_vm_with_data(
        instructions: Vec<Instruction>,
        labels: Option<HashMap<String, usize>>,
        data: HashMap<String, Field>,
    ) -> Result<Vm, Error> {
        let mut vm = Vm::new(true);
        execute(&mut vm, instructions, labels, Some(data))?;
        Ok(vm)
    }

    fn create_vm(
        instructions: Vec<Instruction>,
        labels: Option<HashMap<String, usize>>,
    ) -> Result<Vm, Error> {
        let mut vm = Vm::new(true);
        execute(&mut vm, instructions, labels, None)?;
        Ok(vm)
    }

    fn execute(
        vm: &mut Vm,
        instructions: Vec<Instruction>,
        labels: Option<HashMap<String, usize>>,
        data: Option<HashMap<String, Field>>,
    ) -> Result<(), Error> {
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
