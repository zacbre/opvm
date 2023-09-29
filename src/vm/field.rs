use crate::types::{Allocation, Object, Type};
use std::{
    fmt::{Display, Formatter},
    ops::{Add, BitXor, Div, Mul, Rem, Sub},
};

use super::register::{Register, RegisterWithOffset};

#[derive(Debug)]
pub struct Field(pub Type);
impl Field {
    pub fn underlying_data_clone(&self) -> Field {
        match &self.0 {
            Type::Byte(b) => Field(Type::Byte(*b)),
            Type::Short(s) => Field(Type::Short(*s)),
            Type::Int(i) => Field(Type::Int(*i)),
            Type::UInt(u) => Field(Type::UInt(*u)),
            Type::Float(f) => Field(Type::Float(*f)),
            Type::Char(c) => Field(Type::Char(*c)),
            Type::String(s) => Field(Type::String(s.clone())),
            Type::Bool(b) => Field(Type::Bool(*b)),
            Type::Pointer(p) => Field(Type::Pointer(p.clone())),
            Type::Register(r) => Field(Type::Register(*r)),
            Type::RegisterWithOffsets(r, o) => {
                let mut offsets: Vec<RegisterWithOffset> = Vec::new();
                for offset in o {
                    offsets.push(offset.clone());
                }
                Field(Type::RegisterWithOffsets(*r, offsets))
            }
            Type::Object(o) => Field(Type::Object((*o).clone())),
            //_ => Field::default(),
        }
    }
    pub fn to_r(&self, arg: &&mut super::vm::Vm) -> Result<Register, super::error::Error> {
        match self.0 {
            Type::Register(r) => Ok(r),
            _ => {
                let err = arg.error(
                    "Value is not a register!".to_string(),
                    Some(vec![self.underlying_data_clone()]),
                );
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_u(&self, arg: &super::vm::Vm) -> Result<usize, super::error::Error> {
        match self.0 {
            Type::UInt(u) => Ok(u),
            Type::Int(i) => Ok(i as usize),
            _ => {
                let err = arg.error(
                    format!("Value '{:?}' is not a number!", self.0),
                    Some(vec![self.underlying_data_clone()]),
                );
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_p(&self, arg: &&mut super::vm::Vm) -> Result<&Allocation, super::error::Error> {
        match &self.0 {
            Type::Pointer(p) => Ok(p),
            _ => {
                let err = arg.error(
                    "Value is not a pointer!".to_string(),
                    Some(vec![self.underlying_data_clone()]),
                );
                Err(err.unwrap_err())
            }
        }
    }
}

impl Default for Field {
    fn default() -> Self {
        Field(Type::Int(0))
    }
}

impl From<u8> for Field {
    fn from(u: u8) -> Self {
        Field(Type::Byte(u))
    }
}

impl From<u16> for Field {
    fn from(u: u16) -> Self {
        Field(Type::Short(u))
    }
}

impl From<usize> for Field {
    fn from(u: usize) -> Self {
        Field(Type::UInt(u))
    }
}

impl From<i64> for Field {
    fn from(i: i64) -> Self {
        Field(Type::Int(i))
    }
}

impl From<i32> for Field {
    fn from(i: i32) -> Self {
        Field(Type::Int(i as i64))
    }
}

impl From<f64> for Field {
    fn from(f: f64) -> Self {
        Field(Type::Float(f))
    }
}

impl From<char> for Field {
    fn from(c: char) -> Self {
        Field(Type::Char(c))
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Field(Type::String(s))
    }
}

impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Field(Type::String(s.to_string()))
    }
}

impl From<bool> for Field {
    fn from(b: bool) -> Self {
        Field(Type::Bool(b))
    }
}

impl From<Allocation> for Field {
    fn from(field: Allocation) -> Self {
        Field(Type::Pointer(field))
    }
}

impl From<Register> for Field {
    fn from(r: Register) -> Self {
        Field(Type::Register(r))
    }
}

impl From<Box<dyn Object>> for Field {
    fn from(o: Box<dyn Object>) -> Self {
        Field(Type::Object(o))
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Field(Type::Byte(b)) => write!(f, "{}", b),
            Field(Type::Short(s)) => write!(f, "{}", s),
            Field(Type::Float(fl)) => write!(f, "{}", fl),
            Field(Type::Bool(b)) => write!(f, "{}", b),
            Field(Type::Int(i)) => write!(f, "{}", i),
            Field(Type::UInt(u)) => write!(f, "{}", u),
            Field(Type::Pointer(p)) => write!(f, "{:?}", p),
            Field(Type::Char(c)) => write!(f, "{}", c),
            Field(Type::String(ref s)) => write!(f, "{}", s),
            Field(Type::Register(r)) => write!(f, "{:?}", r),
            Field(Type::RegisterWithOffsets(r, o)) => {
                write!(f, "{:?} {:?}", r, o)
            }
            Field(Type::Object(ref o)) => write!(f, "{}", (*o).to_string()),
            //_ => write!(f, "{:?}", self),
        }
    }
}

impl Add for Field {
    type Output = Field;

    fn add(self, rhs: Self) -> Self::Output {
        Field(self.0 + rhs.0)
    }
}

impl Sub for Field {
    type Output = Field;

    fn sub(self, rhs: Self) -> Self::Output {
        Field(self.0 - rhs.0)
    }
}

impl Mul for Field {
    type Output = Field;

    fn mul(self, rhs: Self) -> Self::Output {
        Field(self.0 * rhs.0)
    }
}

impl Div for Field {
    type Output = Field;

    fn div(self, rhs: Self) -> Self::Output {
        Field(self.0 / rhs.0)
    }
}

impl BitXor for Field {
    type Output = Field;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Field(self.0 ^ rhs.0)
    }
}

impl Rem for Field {
    type Output = Field;

    fn rem(self, rhs: Self) -> Self::Output {
        Field(self.0 % rhs.0)
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd for Field {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
