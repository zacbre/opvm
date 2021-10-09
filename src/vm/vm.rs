use std::cmp;
use crate::vm::instruction::Instruction;
use crate::vm::opcode::OpCode;
use crate::vm::field::Field;
use std::collections::HashMap;
use crate::vm::error::Error;
use crate::vm::program::Program;
use crate::vm::stack;
use crate::vm::stack::Stack;
use crate::vm::heap::Heap;

pub struct Vm {
    instructions: Vec<Instruction>,
    labels: HashMap<String,usize>,
    data: HashMap<String, Field>,
    stack: stack::Stack<Field>,
    call_stack: stack::Stack<usize>,
    pc: usize,
    heap: HashMap<String,Heap>,
    reflection: bool
}

impl Vm {
    pub fn new(reflection: bool) -> Self {
        Vm{
            instructions: vec![],
            labels: HashMap::new(),
            data: HashMap::new(),
            stack: stack::Stack::new(),
            call_stack: stack::Stack::new(),
            pc: 0,
            heap: HashMap::new(),
            reflection
        }
    }

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

        let stack_size_var = Field::from("$__stack_size");
        let callstack_size_var = Field::from("$__callstack_size");
        let pc_var = Field::from("$__pc");

        if self.reflection {
            self.allocate_heap(&stack_size_var)?;
            self.allocate_heap(&callstack_size_var)?;
            self.allocate_heap(&pc_var)?;
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
                            return self.error(format!("Cannot increment non-int type at {}!", self.pc), Some(vec![v1]));
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
                            return self.error(format!("Cannot decrement non-int type at {}!", self.pc), Some(vec![v1]));
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
                OpCode::Alloc => {
                    let address = self.pop_operand(&mut instruction.operand)?;

                    self.allocate_heap(&address)?;
                }
                OpCode::Free => {
                    let address = self.pop_operand(&mut instruction.operand)?;

                    self.free_heap(&address)?;
                }
                OpCode::Load => {
                    let address = self.pop_operand(&mut instruction.operand)?;

                    let heap_copy = self.load_heap(&address)?;
                    self.stack.push(heap_copy);

                }
                OpCode::Store => {
                    let address = self.pop_operand(&mut instruction.operand)?;
                    let v1 = self.pop_stack()?;

                    self.store_heap(&address, v1)?;
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
                self.store_heap(&stack_size_var, Field::from(self.stack.len()))?;
                self.store_heap(&callstack_size_var, Field::from(self.call_stack.len()))?;
                self.store_heap(&pc_var, Field::from(self.pc))?;
            }
        }
        Ok(())
    }

    fn error(&self, msg: String, field: Option<Vec<Field>>) -> Result<(),Error> {
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
                            assembled.push_str(format!("{} ", item.to_string()).as_str());
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

    fn allocate_heap(&mut self, var: &Field) -> Result<(), Error> {
        let cloned_field = var.clone();
        if self.heap.contains_key(self.check_str(cloned_field)?.as_str()) {
            return self.error("That variable was already allocated!".to_string(), Some(vec![var.clone()]));
        }
        self.heap.insert(var.to_string(), Heap::new());

        Ok(())
    }

    fn free_heap(&mut self, var: &Field) -> Result<(), Error> {
        let cloned_var = var.clone();
        let field = cloned_var.to_str().unwrap();
        if !self.heap.contains_key(field) {
            return self.error("The variable wasn't allocated!".to_string(), Some(vec![var.clone()]));
        }
        self.heap.remove(field);
        Ok(())
    }

    fn load_heap(&mut self, var: &Field) -> Result<Field, Error> {
        let key = var.to_str().unwrap();
        if !self.heap.contains_key(key) {
            let err = self.error("The variable doesn't exist!".to_string(), Some(vec![var.clone()]));
            return Err(err.err().unwrap());
        }

        let value = self.heap.get_mut(key);
        return match value {
            Some(v) => {
                let cloned_item = v.item.clone();
                return match cloned_item {
                    Some(i) => Ok(*i),
                    None => {
                        let err = self.error("Unable to load from heap!".to_string(), Some(vec![var.clone()]));
                        Err(err.err().unwrap())
                    }
                }

            }
            None => {
                let err = self.error("Unable to load from heap!".to_string(), Some(vec![var.clone()]));
                Err(err.err().unwrap())
            }
        }

    }

    fn store_heap(&mut self, var: &Field, item: Field) -> Result<(), Error> {
        let key = var.to_str().unwrap();
        if !self.heap.contains_key(key) {
            return self.error("The variable does not exist!".to_string(), Some(vec![var.clone()]));
        }
        let item = Box::new(item);

        let heap = self.heap.get_mut(key);
        let mut heapitem = heap.unwrap();
        heapitem.item = Some(item);

        return Ok(());
    }

    fn check_int(&self, operand: Field) -> Result<i64, Error> {
        let item = operand.to_i();
        match item {
            Some(i) => Ok(i),
            None => {
                let err = self.error("Cannot parse as integer!".to_string(), Some(vec![operand]));
                Err(err.err().unwrap())
            }
        }
    }

    fn check_usize(&self, operand: Field) -> Result<usize, Error> {
        let item = operand.to_u();
        match item {
            Some(u) => Ok(u),
            None => {
                let err = self.error("Cannot parse as usize!".to_string(), Some(vec![operand]));
                Err(err.err().unwrap())
            }
        }
    }

    fn check_str(&self, operand: Field) -> Result<String, Error> {
        let item = operand.to_s();
        match item {
            Some(s) => Ok(s),
            None => {
                let err = self.error("Cannot parse as string!".to_string(), Some(vec![operand]));
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
            ins(OpCode::Push, 4)
        ], None)?;

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 4 as i64);
        Ok(())
    }

    #[test]
    fn test_pop() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins_e(OpCode::Pop)
        ], None)?;

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_add() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins(OpCode::Push, 5),
            ins_e(OpCode::Add)
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 9);
        Ok(())
    }

    #[test]
    fn test_mul() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins(OpCode::Push, 5),
            ins_e(OpCode::Mul)
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 20);
        Ok(())
    }

    #[test]
    fn test_sub() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 10),
            ins(OpCode::Push, 3),
            ins_e(OpCode::Sub)
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 7);
        Ok(())
    }

    #[test]
    fn test_div() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 12),
            ins(OpCode::Push, 3),
            ins_e(OpCode::Div)
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 4);
        Ok(())
    }

    #[test]
    fn test_mod() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 13),
            ins(OpCode::Push, 3),
            ins_e(OpCode::Mod)
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 1);
        Ok(())
    }

    #[test]
    fn test_print() -> Result<(),Error>  {
        let vm = create_vm(vec![
            ins(OpCode::Push, 3),
            ins_e(OpCode::Print)
        ], None)?;

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 1);
        let mut vm = create_vm(vec![
            ins(OpCode::Call, "@func"),
            ins(OpCode::Push, "should be on stack"),
        ], Some(hashmap))?;

        assert_eq!(vm.pop_stack()?.to_str().unwrap(), "should be on stack");
        Ok(())
    }

    #[test]
    fn test_ret() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@func".to_string(), 2);
        hashmap.insert("@end".to_string(), 5);
        let mut vm = create_vm(vec![
            ins(OpCode::Call, "@func"),
            ins(OpCode::Jmp, "@end"),
            ins(OpCode::Push, "test"),
            ins_e(OpCode::Pop),
            ins_e(OpCode::Ret),
            ins(OpCode::Push, "should be on stack"),
        ], Some(hashmap))?;

        assert_eq!(vm.pop_stack()?.to_str().unwrap(), "should be on stack");
        assert_eq!(vm.stack.len(), 0);
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
            ins(OpCode::Push, 1),
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);
        Ok(())
    }

    #[test]
    fn test_je() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 1),
            ins(OpCode::Push, 1),
            ins(OpCode::Je, "@equal"),
            ins(OpCode::Push, 5),
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 1),
            ins(OpCode::Push, 2),
            ins(OpCode::Je, "@equal"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jne() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 2),
            ins(OpCode::Push, 1),
            ins(OpCode::Jne, "@notequal"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@notequal".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 1),
            ins(OpCode::Push, 1),
            ins(OpCode::Jne, "@notequal"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jle() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins(OpCode::Push, 7),
            ins(OpCode::Jle, "@less"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 7),
            ins(OpCode::Push, 7),
            ins(OpCode::Jle, "@equal"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 7),
            ins(OpCode::Push, 4),
            ins(OpCode::Jle, "@less"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jge() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 7),
            ins(OpCode::Push, 4),
            ins(OpCode::Jge, "@greater"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@equal".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 7),
            ins(OpCode::Push, 7),
            ins(OpCode::Jge, "@equal"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins(OpCode::Push, 7),
            ins(OpCode::Jge, "@greater"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jl() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins(OpCode::Push, 7),
            ins(OpCode::Jl, "@less"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@less".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 7),
            ins(OpCode::Push, 4),
            ins(OpCode::Jl, "@less"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_jg() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let vm = create_vm(vec![
            ins(OpCode::Push, 7),
            ins(OpCode::Push, 4),
            ins(OpCode::Jg, "@greater"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;

        assert_eq!(vm.stack.len(), 0);

        let mut hashmap = HashMap::new();
        hashmap.insert("@greater".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 4),
            ins(OpCode::Push, 7),
            ins(OpCode::Jg, "@greater"),
            ins(OpCode::Push, 5)
        ], Some(hashmap))?;


        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 5);
        Ok(())
    }

    #[test]
    fn test_dup() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 10),
            ins_e(OpCode::Dup),
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 10);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 10);
        Ok(())
    }

    #[test]
    fn test_inc() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 10),
            ins_e(OpCode::Inc),
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 11);
        Ok(())
    }

    #[test]
    fn test_dec() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 10),
            ins_e(OpCode::Dec),
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 9);
        Ok(())
    }

    #[test]
    fn test_swap() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 10),
            ins(OpCode::Push, 20),
            ins_e(OpCode::Swap),
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 10);
        assert_eq!(vm.pop_stack()?.to_i().unwrap(), 20);
        Ok(())
    }

    #[test]
    fn test_concat() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Push, 10),
            ins(OpCode::Push, 20),
            ins_e(OpCode::Concat),
        ], None)?;

        assert_eq!(vm.pop_stack()?.to_str().unwrap(), "1020");
        Ok(())
    }

    #[test]
    fn test_alloc_store() -> Result<(),Error>  {
        let mut vm = create_vm(vec![
            ins(OpCode::Alloc, "$myvar"),
            ins(OpCode::Push, 20 as usize),
            ins(OpCode::Store, "$myvar"),
            ins(OpCode::Alloc, "$myvar1"),
            ins(OpCode::Push, "hey this is my lame string"),
            ins(OpCode::Store, "$myvar1"),
        ], None)?;

        assert_eq!(vm.heap.len(), 5);
        let mutmap = vm.heap.get_mut("$myvar");
        let unwrapped = mutmap.unwrap().item.clone().unwrap();
        assert_eq!(vm.check_usize(*unwrapped)?, 20);

        let mutmap1 = vm.heap.get_mut("$myvar1");
        let unwrapped1 = mutmap1.unwrap().item.clone().unwrap();
        assert_eq!(vm.check_str(*unwrapped1)?, "hey this is my lame string");
        Ok(())
    }

    #[test]
    fn test_alloc_load() -> Result<(),Error>  {
        let mut hashmap = HashMap::new();
        hashmap.insert("@match".to_string(), 4);
        let mut vm = create_vm(vec![
            ins(OpCode::Alloc, "$myvar"),
            ins(OpCode::Push, 20 as usize),
            ins(OpCode::Store, "$myvar"),
            ins(OpCode::Load, "$__stack_size"),
            ins(OpCode::Push, 1),
            ins(OpCode::Jl, "@end"),
            ins(OpCode::Push, 45),
            ins_e(OpCode::Nop)
        ], None)?;

        assert_eq!(vm.heap.len(), 4);
        let popped_stack = vm.pop_stack()?;
        assert_eq!(45, vm.check_int(popped_stack)?);
        Ok(())
    }

    fn ins<T>(opcode: OpCode, item: T) -> Instruction where Field: From<T> {
        Instruction::new(opcode, vec![Field::from(item)])
    }

    fn ins_e(opcode: OpCode) -> Instruction {
        Instruction::new(opcode, vec![])
    }

    fn create_vm(instructions: Vec<Instruction>, labels: Option<HashMap<String, usize>>) -> Result<Vm,Error> {
        let mut vm = Vm::new(true);
        execute(&mut vm, instructions, labels)?;
        Ok(vm)
    }

    fn execute(vm: &mut Vm, instructions: Vec<Instruction>, labels: Option<HashMap<String, usize>>) -> Result<(),Error> {
        let mut program = Program::new();
        program.instructions = instructions;

        if labels.is_some() {
            program.labels = labels.unwrap();
        }

        vm.execute(program)?;

        Ok(())
    }
}