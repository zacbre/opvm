use std::fmt::Display;
use core::fmt::Debug;

use crate::vm::register::Register;

pub mod date;
pub mod duration;

pub trait Object: Display + Debug{
    fn clone(&self) -> Box<dyn Object>;
}

#[derive(Debug)]
pub enum Type {
    Byte(u8),
    Short(u16),
    Int(i64),
    UInt(usize),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
    Pointer(*mut [usize]),
    Register(Register),
    Object(Box<dyn Object>),
}
