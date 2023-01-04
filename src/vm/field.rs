use std::fmt::{Display, Formatter};
use crate::vm::error::Error;
use crate::vm::register::{OffsetOperand, Register};
use crate::vm::vm::Vm;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Field {
    I(i64),
    U(usize),
    S(String),
    P(*mut [usize]),
    R(Register),
    RO(Register, OffsetOperand),
}

impl Default for Field {
    fn default() -> Self {
        Field::I(0)
    }
}

impl Field {
    pub fn to_i(&self, vm: &Vm) -> Result<i64, Error> {
        match self {
            &Field::I(num) => Ok(num),
            _ => {
                let err = vm.error("Value is not an int!".to_string(), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_str(&self, vm: &Vm) -> Result<&str, Error> {
        match self {
            &Field::S(ref s) => Ok(s),
            _ => {
                let err = vm.error("Value is not a string!".to_string(), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_s(&self, vm: &Vm) -> Result<String, Error> {
        match self {
            &Field::S(ref s) => Ok(s.to_string()),
            _ => {
                let err = vm.error("Value is not a string!".to_string(), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_u(&self, vm: &Vm) -> Result<usize, Error> {
        match self {
            &Field::U(u) => Ok(u),
            _ => {
                let err = vm.error("Value is not a u64!".to_string(), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_r(&self, vm: &Vm) -> Result<(Register, OffsetOperand), Error> {
        match self {
            &Field::RO(r, offset_operand) => Ok((r, offset_operand)),
            &Field::R(r) => Ok((r, OffsetOperand::Default)),
            _ => {
                let err = vm.error(format!("Value is not a register: '{}'!", &self), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }

    pub fn to_p(&self, vm: &Vm) -> Result<*mut [usize], Error> {
        match self {
            &Field::P(p) => Ok(p),
            _ => {
                let err = vm.error(format!("Value is not a register: '{}'!", &self), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }
}

impl From<usize> for Field {
    fn from(u: usize) -> Self {
        Field::U(u)
    }
}

impl From<i64> for Field {
    fn from(i: i64) -> Self {
        Field::I(i)
    }
}

impl From<i32> for Field {
    fn from(i: i32) -> Self {
        Field::I(i as i64)
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Field::S(s)
    }
}

impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Field::S(s.to_string())
    }
}

impl From<*mut [usize]> for Field {
    fn from(field: *mut [usize]) -> Self {
        Field::P(field)
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Field::I(i) => write!(f, "{}", i),
            &Field::U(u) => write!(f, "{}", u),
            &Field::P(p) => write!(f, "{:?}", p),
            &Field::S(ref s) => {
                write!(f, "{}", s)
            },
            &Field::R(r) => {
                write!(f, "{:?}", r)
            }
            &Field::RO(r, operand) => {
                let operand = match operand {
                    OffsetOperand::Default => "".to_string(),
                    OffsetOperand::Number(n) => format!("[{}]", n),
                    OffsetOperand::Register(r) => format!("[{}]", r),
                };
                write!(f, "{:?}{}", r, operand)
            }
        }
    }
}