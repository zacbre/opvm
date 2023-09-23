use std::fmt::Display;
use core::fmt::Debug;

use crate::vm::register::Register;

pub trait Object: Display + Debug{
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

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Date(chrono::DateTime<chrono::Utc>);
impl Object for Date {}
impl Date {
    pub fn format(&self, fmt: &str) -> String {
        self.0.format(fmt).to_string()
    }
}
impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", chrono::Utc::now().to_rfc3339())
    }
}