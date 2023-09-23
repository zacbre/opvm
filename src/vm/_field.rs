use std::fmt::{Display, Formatter};
use std::str::FromStr;
use chrono::{DateTime, Utc};

use crate::vm::error::Error;
use crate::vm::heap::Heap;
use crate::vm::register::{OffsetOperand, Register};
use crate::vm::vm::Vm;

#[derive(Debug, PartialEq)]
pub enum CastType {
    None,
    CastToI64,
    CastToUsize,
    CastToString,
    CastToChar,
}

impl CastType {
    pub fn from(s: &str) -> Self {
        match s {
            "i64" => CastType::CastToI64,
            "usize" => CastType::CastToUsize,
            "str" => CastType::CastToString,
            "char" => CastType::CastToChar,
            _ => CastType::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Field {
    I(i64),
    L(u128),
    D(DateTime<Utc>),
    U(usize),
    C(char),
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

    pub fn to_i_or_u(&self, vm: &Vm) -> Result<usize, Error> {
        match self {
            &Field::U(u) => Ok(u),
            &Field::I(i) => Ok(i as usize),
            &Field::C(c) => Ok(c as usize),
            _ => {
                let err = vm.error(format!("Value is not an i64 or a usize or a char, it's a '{:?}'!", self), Some(vec![self.clone()]));
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

    pub fn to_l(&self, vm: &Vm) -> Result<u128, Error> {
        match self {
            &Field::L(l) => Ok(l),
            _ => {
                let err = vm.error("Value is not a u128!".to_string(), Some(vec![self.clone()]));
                Err(err.unwrap_err())
            }
        }
    }

    pub fn cast(self, vm: &Vm, cast_type: CastType, ro: OffsetOperand) -> Result<Field, Error> {
        if cast_type == CastType::None {
            return Ok(self);
        }

        match self.clone() {
            Field::I(i) => {
                match cast_type {
                    CastType::CastToI64 => Ok(Field::from(i)),
                    CastType::CastToUsize => Ok(Field::from(i as usize)),
                    CastType::CastToString => Ok(Field::from(i.to_string())),
                    CastType::CastToChar => {
                        Ok(Field::from(char::from_u32(i as u32).unwrap()))
                    }
                    _ => {
                        Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", &self, cast_type), Some(vec![self.clone()])).unwrap_err())
                    }
                }
            }
            Field::U(u) => {
                match cast_type {
                    CastType::CastToI64 => Ok(Field::from(u as i64)),
                    CastType::CastToUsize => Ok(Field::from(u)),
                    CastType::CastToString => Ok(Field::from(u.to_string())),
                    _ => {
                        Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", &self, cast_type), Some(vec![self.clone()])).unwrap_err())
                    }
                }
            }
            Field::D(d) => {
                match cast_type {
                    CastType::CastToString => Ok(Field::from(d.to_string())),
                    _ => {
                        Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", &self, cast_type), Some(vec![self.clone()])).unwrap_err())
                    }
                }
            }
            Field::L(l) => {
                match cast_type {
                    CastType::CastToString => Ok(Field::from(l.to_string())),
                    _ => {
                        Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", &self, cast_type), Some(vec![self.clone()])).unwrap_err())
                    }
                }
            }
            Field::C(c) => {
                match cast_type {
                    CastType::CastToI64 => Ok(Field::from(c as i64)),
                    CastType::CastToUsize => Ok(Field::from(c as usize)),
                    CastType::CastToString => Ok(Field::from(c.to_string())),
                    CastType::CastToChar => Ok(Field::from(c)),
                    _ => {
                        Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", &self, cast_type), Some(vec![self.clone()])).unwrap_err())
                    }
                }
            }
            Field::S(s) => {
                match cast_type {
                    CastType::CastToI64 => {
                        match s.parse::<i64>() {
                            Ok(o) => Ok(Field::from(o)),
                            Err(e) => Err(e.into())
                        }
                    },
                    CastType::CastToUsize => {
                        match s.parse::<usize>() {
                            Ok(u) => Ok(Field::from(u)),
                            Err(e) => Err(e.into())
                        }
                    },
                    CastType::CastToString => {
                        Ok(Field::from(s))
                    },
                    CastType::CastToChar => {
                        match s.parse::<char>() {
                            Ok(c) => Ok(Field::from(c)),
                            Err(e) => Err(e.into())
                        }
                    },
                    _ => {
                        Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", self.clone(), cast_type), Some(vec![self.clone()])).unwrap_err())
                    }
                }
            }
            Field::P(p) => {
                let mut derefed = Heap::deref(p);
                let offset = ro.resolve(vm)?;
                match cast_type {
                    CastType::CastToI64 => derefed[offset] = derefed[offset] as i64 as usize,
                    CastType::CastToUsize => derefed[offset] = derefed[offset] as usize,
                    CastType::CastToString => {
                        derefed[offset] = usize::from_str(derefed[offset].to_string().as_str()).unwrap();
                    },
                    CastType::CastToChar => {
                        derefed[offset] = char::from_u32(derefed[offset] as u32).unwrap() as usize
                    },
                    _ => {}
                };
                let rerefed = Heap::reref(derefed);
                Ok(Field::from(rerefed))
            }
            _ => {
                Err(vm.error(format!("Cannot cast '{:?}' to: {:?}", &self, cast_type), Some(vec![self.clone()])).unwrap_err())
            }
        }
    }
}

impl From<usize> for Field {
    fn from(u: usize) -> Self {
        Field::U(u)
    }
}

impl From<char> for Field {
    fn from(value: char) -> Self {
        Field::C(value)
    }
}

impl From<u128> for Field {
    fn from(i: u128) -> Self {
        Field::L(i)
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

impl From<DateTime<Utc>> for Field {
    fn from(d: DateTime<Utc>) -> Self {
        Field::D(d)
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
            &Field::D(d) => write!(f, "{}", d),
            &Field::I(i) => write!(f, "{}", i),
            &Field::L(l) => write!(f, "{}", l),
            &Field::U(u) => write!(f, "{}", u),
            &Field::P(p) => write!(f, "{:?}", p),
            &Field::C(c) => write!(f, "{}", c),
            &Field::S(ref s) => write!(f, "{}", s),
            &Field::R(r) => write!(f, "{:?}", r),
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