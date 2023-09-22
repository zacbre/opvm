use std::{fmt::Debug, time::{SystemTime, UNIX_EPOCH}};
use chrono::Utc;

use super::{register::Registers, field::Field, stack::Stack};


pub trait BuiltIn: Debug {
    fn call(&self, registers: &mut Registers, args: &mut Stack<Field>) -> Field;
    fn get_name(&self) -> &str;
}

#[derive(Debug)]
pub struct Println;
impl BuiltIn for Println {
    fn call(&self, registers: &mut Registers, _: &mut Stack<Field>) -> Field {
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
    fn call(&self, registers: &mut Registers, _: &mut Stack<Field>) -> Field {
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
    fn call(&self, registers: &mut Registers, _: &mut Stack<Field>) -> Field {
        Field::from(format!("{}{}", registers.rd, registers.re).as_str())        
    }

    fn get_name(&self) -> &str {
        "__concat"
    }
}

#[derive(Debug)]
pub struct DateNowUnix;
impl BuiltIn for DateNowUnix {
    fn call(&self, registers: &mut Registers, _: &mut Stack<Field>) -> Field {
        Field::from(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros())
    }

    fn get_name(&self) -> &str {
        "__date_now_unix"
    }
}

#[derive(Debug)]
pub struct DateNow;
impl BuiltIn for DateNow {
    fn call(&self, registers: &mut Registers, _: &mut Stack<Field>) -> Field {
        Field::from(Utc::now())
    }

    fn get_name(&self) -> &str {
        "__date_now"
    }
}