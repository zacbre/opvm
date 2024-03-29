use crate::vm::opcode;
use crate::vm::opcode::OpCode;
use crate::vm::field::Field;
use crate::vm::stack::Stack;

#[derive(Clone, Debug)]
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

    pub fn new_from_words(str: Vec<&str>) -> Self {
        let pre_opcode = *str.get(0).unwrap();
        let opcode = OpCode::from(pre_opcode);
        if opcode == OpCode::Igl {
            println!("Error: Unknown opcode: {:?}", str);
        }
        let mut stack: Stack<Field> = Stack::new();
        for i in 1..str.len() {
            stack.push(Instruction::construct_field(str[i]));
        }

        Instruction {
            opcode,
            operand: stack
        }
    }

    pub fn construct_field(str: &str) -> Field {
        match str.parse::<i64>() {
            Ok(i) => { return Field::from(i); }
            Err(_) => (),
        }

        match str.parse::<i32>() {
            Ok(i) => { return Field::from(i); }
            Err(_) => (),
        }

        Field::from(str)
    }

    pub fn assemble(&self) -> String {
        let str: &str = self.opcode.into();

        let mut final_string = String::default();
        final_string.push_str(str);
        let cloned_operands = self.operand.clone();
        let operands = cloned_operands.to_vec();
        for i in 0..operands.len() {
            let item = &operands[i];
            final_string.push_str(" ");
            final_string.push_str(item.to_string().as_str());
        }
        final_string
    }
}
