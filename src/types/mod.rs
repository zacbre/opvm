use core::fmt::Debug;
use std::{
    fmt::Display,
    ops::{Add, BitXor, Div, Mul, Rem, Sub},
    ptr::NonNull,
};

use crate::vm::register::{Register, RegisterWithOffset};

pub mod date;
pub mod duration;

#[derive(Debug, Clone)]
pub struct Allocation {
    pub ptr: NonNull<u8>,
    pub size: usize,
    pub align: usize,
}

impl Allocation {
    pub fn new(ptr: NonNull<u8>, size: usize, align: usize) -> Self {
        Self { ptr, size, align }
    }
}

impl PartialEq for Allocation {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl PartialOrd for Allocation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.ptr.partial_cmp(&other.ptr)
    }
}

pub trait Object: Display + Debug {
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
    Pointer(Allocation),
    Register(Register),
    RegisterWithOffsets(RegisterWithOffset),
    Object(Box<dyn Object>),
}

macro_rules! add_types {
    ($left:expr, $right:expr, $($pat:pat => $result:expr),*) => {{
        match ($left, $right) {
            $($pat => $result,)*
            _ => panic!("Invalid combination for type..."),
        }
    }};
}

impl Add for Type {
    type Output = Type;

    fn add(self, rhs: Self) -> Self::Output {
        add_types!(self, rhs,
            (Type::Byte(b1), Type::Byte(b2)) => Type::Byte(b1.wrapping_add(b2)),
            (Type::Short(s1), Type::Short(s2)) => Type::Short(s1.wrapping_add(s2)),
            (Type::Int(i1), Type::Int(i2)) => Type::Int(i1.wrapping_add(i2)),
            (Type::UInt(u1), Type::UInt(u2)) => Type::UInt(u1.wrapping_add(u2)),
            (Type::Float(f1), Type::Float(f2)) => Type::Float(f1 + f2),
            (Type::Char(c1), Type::Char(c2)) => Type::Int((c1 as i32 + c2 as i32).into()),
            (Type::UInt(u), Type::Int(i)) => Type::Int(u as i64 + i),
            (Type::Int(i), Type::UInt(u)) => Type::Int(i + u as i64)
            // todo: add more combinations later
            //(Type::String(s1), Type::String(s2)) => Type::String(format!("{}{}", s1, s2)),
            //todo: (Type::Pointer(p1), Type::Pointer(p2)) => todo!(),
        )
    }
}

impl Sub for Type {
    type Output = Type;

    fn sub(self, rhs: Self) -> Self::Output {
        add_types!(self, rhs,
            (Type::Byte(b1), Type::Byte(b2)) => Type::Byte(b1.wrapping_sub(b2)),
            (Type::Short(s1), Type::Short(s2)) => Type::Short(s1.wrapping_sub(s2)),
            (Type::Int(i1), Type::Int(i2)) => Type::Int(i1.wrapping_sub(i2)),
            (Type::UInt(u1), Type::UInt(u2)) => Type::UInt(u1.wrapping_sub(u2)),
            (Type::Float(f1), Type::Float(f2)) => Type::Float(f1 - f2),
            (Type::Char(c1), Type::Char(c2)) => Type::Int((c1 as i32 - c2 as i32).into()),
            (Type::UInt(u), Type::Int(i)) => Type::Int(u as i64 - i),
            (Type::Int(i), Type::UInt(u)) => Type::Int(i - u as i64)
            // todo: add more combinations later
            //(Type::String(s1), Type::String(s2)) => Type::String(format!("{}{}", s1, s2)),
            //todo: (Type::Pointer(p1), Type::Pointer(p2)) => todo!(),
        )
    }
}

impl Mul for Type {
    type Output = Type;

    fn mul(self, rhs: Self) -> Self::Output {
        add_types!(self, rhs,
            (Type::Byte(b1), Type::Byte(b2)) => Type::Byte(b1.wrapping_mul(b2)),
            (Type::Short(s1), Type::Short(s2)) => Type::Short(s1.wrapping_mul(s2)),
            (Type::Int(i1), Type::Int(i2)) => Type::Int(i1.wrapping_mul(i2)),
            (Type::UInt(u1), Type::UInt(u2)) => Type::UInt(u1.wrapping_mul(u2)),
            (Type::Float(f1), Type::Float(f2)) => Type::Float(f1 * f2),
            (Type::Char(c1), Type::Char(c2)) => Type::Int((c1 as i32 * c2 as i32).into()),
            (Type::UInt(u), Type::Int(i)) => Type::Int(u as i64 * i),
            (Type::Int(i), Type::UInt(u)) => Type::Int(i * u as i64),
            (Type::UInt(u), Type::Float(f1)) => Type::Float(u as f64 * f1),
            (Type::Float(f1), Type::UInt(u)) => Type::Float(u as f64 * f1),
            (Type::Int(u), Type::Float(f1)) => Type::Float(u as f64 * f1),
            (Type::Float(f1), Type::Int(u)) => Type::Float(u as f64 * f1)
            // todo: add more combinations later
            //(Type::String(s1), Type::String(s2)) => Type::String(format!("{}{}", s1, s2)),
            //todo: (Type::Pointer(p1), Type::Pointer(p2)) => todo!(),
        )
    }
}

impl Div for Type {
    type Output = Type;

    fn div(self, rhs: Self) -> Self::Output {
        add_types!(self, rhs,
            (Type::Byte(b1), Type::Byte(b2)) => Type::Byte(b1.wrapping_div(b2)),
            (Type::Short(s1), Type::Short(s2)) => Type::Short(s1.wrapping_div(s2)),
            (Type::Int(i1), Type::Int(i2)) => Type::Int(i1.wrapping_div(i2)),
            (Type::UInt(u1), Type::UInt(u2)) => Type::UInt(u1.wrapping_div(u2)),
            (Type::Float(f1), Type::Float(f2)) => Type::Float(f1 / f2),
            (Type::Char(c1), Type::Char(c2)) => Type::Int((c1 as i32 / c2 as i32).into()),
            (Type::UInt(u), Type::Int(i)) => Type::Int(u as i64 / i),
            (Type::Int(i), Type::UInt(u)) => Type::Int(i / u as i64)
            // todo: add more combinations later
            //(Type::String(s1), Type::String(s2)) => Type::String(format!("{}{}", s1, s2)),
            //todo: (Type::Pointer(p1), Type::Pointer(p2)) => todo!(),
        )
    }
}

impl BitXor for Type {
    type Output = Type;

    fn bitxor(self, rhs: Self) -> Self::Output {
        add_types!(self, rhs,
            (Type::Byte(b1), Type::Byte(b2)) => Type::Byte(b1 ^ b2),
            (Type::Short(s1), Type::Short(s2)) => Type::Short(s1 ^ s2),
            (Type::Int(i1), Type::Int(i2)) => Type::Int(i1 ^ i2),
            (Type::UInt(u1), Type::UInt(u2)) => Type::UInt(u1 ^ u2),
            (Type::Char(c1), Type::Char(c2)) => Type::Int((c1 as i32 ^ c2 as i32).into())
            // todo: add more combinations later
            //(Type::String(s1), Type::String(s2)) => Type::String(format!("{}{}", s1, s2)),
            //todo: (Type::Pointer(p1), Type::Pointer(p2)) => todo!(),
        )
    }
}

impl Rem for Type {
    type Output = Type;

    fn rem(self, rhs: Self) -> Self::Output {
        add_types!(self, rhs,
            (Type::Byte(b1), Type::Byte(b2)) => Type::Byte(b1 % b2),
            (Type::Short(s1), Type::Short(s2)) => Type::Short(s1 % s2),
            (Type::Int(i1), Type::Int(i2)) => Type::Int(i1 % i2),
            (Type::UInt(u1), Type::UInt(u2)) => Type::UInt(u1 % u2),
            (Type::Char(c1), Type::Char(c2)) => Type::Int((c1 as i32 % c2 as i32).into())
            // todo: add more combinations later
            //(Type::String(s1), Type::String(s2)) => Type::String(format!("{}{}", s1, s2)),
            //todo: (Type::Pointer(p1), Type::Pointer(p2)) => todo!(),
        )
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Byte(l0), Self::Byte(r0)) => l0 == r0,
            (Self::Short(l0), Self::Short(r0)) => l0 == r0,
            (Self::Int(l0), Self::Int(r0)) => l0 == r0,
            (Self::UInt(l0), Self::UInt(r0)) => l0 == r0,
            (Self::Float(l0), Self::Float(r0)) => l0 == r0,
            (Self::Char(l0), Self::Char(r0)) => l0 == r0,
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Bool(l0), Self::Bool(r0)) => l0 == r0,
            (Self::Pointer(l0), Self::Pointer(r0)) => l0 == r0,
            (Self::Register(l0), Self::Register(r0)) => l0 == r0,
            (Self::RegisterWithOffsets(l0), Self::RegisterWithOffsets(r0)) => {
                for (l, r) in l0.offsets.iter().zip(r0.offsets.iter()) {
                    if l != r {
                        return false;
                    }
                }
                l0.register.eq(&r0.register)
            }
            _ => false,
        }
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Byte(l0), Self::Byte(r0)) => l0.partial_cmp(r0),
            (Self::Short(l0), Self::Short(r0)) => l0.partial_cmp(r0),
            (Self::Int(l0), Self::Int(r0)) => l0.partial_cmp(r0),
            (Self::UInt(l0), Self::UInt(r0)) => l0.partial_cmp(r0),
            (Self::Float(l0), Self::Float(r0)) => l0.partial_cmp(r0),
            (Self::Char(l0), Self::Char(r0)) => l0.partial_cmp(r0),
            (Self::String(l0), Self::String(r0)) => l0.partial_cmp(r0),
            (Self::Bool(l0), Self::Bool(r0)) => l0.partial_cmp(r0),
            (Self::Pointer(l0), Self::Pointer(r0)) => l0.partial_cmp(r0),
            (Self::Register(l0), Self::Register(r0)) => l0.partial_cmp(r0),
            (Self::RegisterWithOffsets(l0), Self::RegisterWithOffsets(r0)) => {
                // compare the offsets.
                for (l, r) in l0.offsets.iter().zip(r0.offsets.iter()) {
                    if l != r {
                        return None;
                    }
                }
                // if they are equal, then compare the registers
                l0.register.partial_cmp(&r0.register)
            }
            // Add additional cases as needed
            _ => None, // Handle cases where comparison is not possible
        }
    }
}
