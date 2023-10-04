use rand::Rng;
use std::{
    fmt::Debug,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{types, types::date::Date, vm::heap::HEAP_MEM};

use super::{field::Field, instruction::Instruction, register::Registers, stack::Stack};

pub trait BuiltIn: Debug {
    fn call(
        &self,
        registers: &mut Registers,
        args: &mut Stack<Field>,
        instructions: &mut Vec<Instruction>,
    ) -> Field;
    fn get_name(&self) -> &str;
}

#[derive(Debug)]
pub struct Println;
impl BuiltIn for Println {
    fn call(
        &self,
        registers: &mut Registers,
        _: &mut Stack<Field>,
        _instructions: &mut Vec<Instruction>,
    ) -> Field {
        println!("{}", registers.rd);
        Field::default()
    }

    fn get_name(&self) -> &str {
        "__println"
    }
}

#[derive(Debug)]
pub struct Print;
impl BuiltIn for Print {
    fn call(
        &self,
        registers: &mut Registers,
        _: &mut Stack<Field>,
        _instructions: &mut Vec<Instruction>,
    ) -> Field {
        print!("{}", registers.rd);
        Field::default()
    }

    fn get_name(&self) -> &str {
        "__print"
    }
}

#[derive(Debug)]
pub struct Concat;
impl BuiltIn for Concat {
    fn call(
        &self,
        registers: &mut Registers,
        _: &mut Stack<Field>,
        _instructions: &mut Vec<Instruction>,
    ) -> Field {
        // todo: there's probably a faster way than creating a new String
        Field::from(format!("{}{}", registers.rd, registers.re).as_str())
    }

    fn get_name(&self) -> &str {
        "__concat"
    }
}

#[derive(Debug)]
pub struct DateNowUnix;
impl BuiltIn for DateNowUnix {
    fn call(
        &self,
        _: &mut Registers,
        _: &mut Stack<Field>,
        _instructions: &mut Vec<Instruction>,
    ) -> Field {
        Field::from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        )
    }

    fn get_name(&self) -> &str {
        "__date_now_unix"
    }
}

#[derive(Debug)]
pub struct DateNow;
impl BuiltIn for DateNow {
    fn call(
        &self,
        _: &mut Registers,
        _: &mut Stack<Field>,
        _instructions: &mut Vec<Instruction>,
    ) -> Field {
        Field::from(Date::new())
    }

    fn get_name(&self) -> &str {
        "__date_now"
    }
}

#[derive(Debug)]
pub struct Dbg;
impl BuiltIn for Dbg {
    fn call(
        &self,
        registers: &mut Registers,
        stack: &mut Stack<Field>,
        instructions: &mut Vec<Instruction>,
    ) -> Field {
        println!("{:?}", registers);
        println!("{:?}", stack);
        println!("{:?}", instructions);
        Field::default()
    }

    fn get_name(&self) -> &str {
        "__dbg_print"
    }
}

#[derive(Debug)]
pub struct DbgPtr;
impl BuiltIn for DbgPtr {
    fn call(&self, _: &mut Registers, _: &mut Stack<Field>, _: &mut Vec<Instruction>) -> Field {
        unsafe {
            println!("{:?}", HEAP_MEM);
        }
        Field::default()
    }

    fn get_name(&self) -> &str {
        "__dbg_heap"
    }
}

#[derive(Debug)]
pub struct Random;
impl BuiltIn for Random {
    fn call(&self, _: &mut Registers, _: &mut Stack<Field>, _: &mut Vec<Instruction>) -> Field {
        let mut rng = rand::thread_rng();
        let number: f64 = rng.gen();
        Field::from(number)
    }

    fn get_name(&self) -> &str {
        "__random"
    }
}

#[derive(Debug)]
pub struct MathFloor;
impl BuiltIn for MathFloor {
    fn call(&self, r: &mut Registers, _: &mut Stack<Field>, _: &mut Vec<Instruction>) -> Field {
        match &r.r0 {
            Field(types::Type::Float(f)) => Field::from(f.floor()),
            _ => r.r0.underlying_data_clone(),
        }
    }

    fn get_name(&self) -> &str {
        "__floor"
    }
}
