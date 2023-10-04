use super::field::Field;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

macro_rules! flag_register {
    ($e:expr,bool) => {
        paste::item! {
            pub fn [<check_ $e >](&self) -> bool {
                self.$e
            }

            pub fn [<set_ $e >](&mut self, u: bool) {
                self.$e = u;
            }
        }
    };
    ($e:expr,$b:ty) => {
        paste::item! {
            #[allow(dead_code)]
            pub fn [<check_ $e >](&self) -> &$b {
                &self.$e
            }

            #[allow(dead_code)]
            pub fn [<set_ $e >](&mut self, u: $b) {
                self.$e = u;
            }
        }
    };
}

#[derive(Debug)]
pub struct RegisterWithOffset {
    pub register: Register,
    pub offsets: Vec<RegisterOffset>,
}

impl RegisterWithOffset {
    pub fn new(register: Register, offsets: Vec<RegisterOffset>) -> Self {
        Self { register, offsets }
    }
}

#[derive(Debug)]
pub struct RegisterOffset {
    pub offset: Field,
    pub operand: RegisterOffsetOperandType,
}

impl Display for RegisterOffsetOperandType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegisterOffsetOperandType::None => write!(f, ""),
            RegisterOffsetOperandType::Add => write!(f, "+"),
            RegisterOffsetOperandType::Sub => write!(f, "-"),
            RegisterOffsetOperandType::Mul => write!(f, "*"),
            RegisterOffsetOperandType::Div => write!(f, "/"),
            RegisterOffsetOperandType::Rem => write!(f, "%"),
        }
    }
}

impl Clone for RegisterWithOffset {
    fn clone(&self) -> Self {
        Self {
            register: self.register,
            offsets: self.offsets.clone(),
        }
    }
}

impl Clone for RegisterOffset {
    fn clone(&self) -> Self {
        Self {
            offset: self.offset.underlying_data_clone(),
            operand: self.operand.clone(),
        }
    }
}

impl PartialEq for RegisterOffset {
    fn eq(&self, other: &Self) -> bool {
        self.offset == other.offset && self.operand == other.operand
    }
}

#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum RegisterOffsetOperandType {
    None,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}
impl RegisterOffsetOperandType {
    pub(crate) fn apply(&self, final_value: &mut Field, i: Field) {
        match self {
            RegisterOffsetOperandType::None => {
                *final_value = i;
            }
            RegisterOffsetOperandType::Add => {
                *final_value = final_value.underlying_data_clone() + i;
            }
            RegisterOffsetOperandType::Sub => {
                *final_value = final_value.underlying_data_clone() - i;
            }
            RegisterOffsetOperandType::Mul => {
                *final_value = final_value.underlying_data_clone() * i;
            }
            RegisterOffsetOperandType::Div => {
                *final_value = final_value.underlying_data_clone() / i;
            }
            RegisterOffsetOperandType::Rem => {
                *final_value = final_value.underlying_data_clone() % i;
            }
        }
    }
}

impl From<char> for RegisterOffsetOperandType {
    fn from(c: char) -> Self {
        match c {
            '+' => RegisterOffsetOperandType::Add,
            '-' => RegisterOffsetOperandType::Sub,
            '*' => RegisterOffsetOperandType::Mul,
            '/' => RegisterOffsetOperandType::Div,
            '%' => RegisterOffsetOperandType::Rem,
            _ => RegisterOffsetOperandType::None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Register {
    Ra,
    Rb,
    Rc,
    Rd,
    Re,
    Rf,
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    Unknown,
}

impl FromStr for Register {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Register::match_register(s))
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Register::Ra => write!(f, "ra"),
            Register::Rb => write!(f, "rb"),
            Register::Rc => write!(f, "rc"),
            Register::Rd => write!(f, "rd"),
            Register::Re => write!(f, "re"),
            Register::Rf => write!(f, "rf"),
            Register::R0 => write!(f, "r0"),
            Register::R1 => write!(f, "r1"),
            Register::R2 => write!(f, "r2"),
            Register::R3 => write!(f, "r3"),
            Register::R4 => write!(f, "r4"),
            Register::R5 => write!(f, "r5"),
            Register::R6 => write!(f, "r6"),
            Register::R7 => write!(f, "r7"),
            Register::R8 => write!(f, "r8"),
            Register::R9 => write!(f, "r9"),
            Register::Unknown => write!(f, "unknown"),
        }
    }
}

impl Register {
    pub fn from(value: &str) -> Self {
        Register::match_register(value)
    }

    pub fn match_register(str: &str) -> Register {
        match str {
            "ra" => Register::Ra,
            "rb" => Register::Rb,
            "rc" => Register::Rc,
            "rd" => Register::Rd,
            "re" => Register::Re,
            "rf" => Register::Rf,
            "r0" => Register::R0,
            "r1" => Register::R1,
            "r2" => Register::R2,
            "r3" => Register::R3,
            "r4" => Register::R4,
            "r5" => Register::R5,
            "r6" => Register::R6,
            "r7" => Register::R7,
            "r8" => Register::R8,
            "r9" => Register::R9,
            _ => Register::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    pub ra: Field,
    pub rb: Field,
    pub rc: Field,
    pub rd: Field,
    pub re: Field,
    pub rf: Field,
    pub r0: Field,
    pub r1: Field,
    pub r2: Field,
    pub r3: Field,
    pub r4: Field,
    pub r5: Field,
    pub r6: Field,
    pub r7: Field,
    pub r8: Field,
    pub r9: Field,
    equals_flag: bool,
    greater_than_flag: bool,
    less_than_flag: bool,
    stack_len: Field,
    call_stack_len: Field,
    pc: Field,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            ra: Field::default(),
            rb: Field::default(),
            rc: Field::default(),
            rd: Field::default(),
            re: Field::default(),
            rf: Field::default(),
            r0: Field::default(),
            r1: Field::default(),
            r2: Field::default(),
            r3: Field::default(),
            r4: Field::default(),
            r5: Field::default(),
            r6: Field::default(),
            r7: Field::default(),
            r8: Field::default(),
            r9: Field::default(),
            equals_flag: false,
            greater_than_flag: false,
            less_than_flag: false,
            stack_len: Field::default(),
            call_stack_len: Field::default(),
            pc: Field::default(),
        }
    }

    pub fn set(&mut self, r: Register, f: Field) {
        match r {
            Register::Ra => self.ra = f,
            Register::Rb => self.rb = f,
            Register::Rc => self.rc = f,
            Register::Rd => self.rd = f,
            Register::Re => self.re = f,
            Register::Rf => self.rf = f,
            Register::R0 => self.r0 = f,
            Register::R1 => self.r1 = f,
            Register::R2 => self.r2 = f,
            Register::R3 => self.r3 = f,
            Register::R4 => self.r4 = f,
            Register::R5 => self.r5 = f,
            Register::R6 => self.r6 = f,
            Register::R7 => self.r7 = f,
            Register::R8 => self.r8 = f,
            Register::R9 => self.r9 = f,
            _ => {}
        }
    }

    pub fn get(&self, p0: Register) -> &Field {
        match p0 {
            Register::Ra => &self.ra,
            Register::Rb => &self.rb,
            Register::Rc => &self.rc,
            Register::Rd => &self.rd,
            Register::Re => &self.re,
            Register::Rf => &self.rf,
            Register::R0 => &self.r0,
            Register::R1 => &self.r1,
            Register::R2 => &self.r2,
            Register::R3 => &self.r3,
            Register::R4 => &self.r4,
            Register::R5 => &self.r5,
            Register::R6 => &self.r6,
            Register::R7 => &self.r7,
            Register::R8 => &self.r8,
            Register::R9 => &self.r9,
            _ => panic!("Register does not exist!"),
        }
    }

    pub fn reset_flags(&mut self) {
        self.equals_flag = false;
        self.less_than_flag = false;
        self.greater_than_flag = false;
    }

    flag_register!(equals_flag, bool);
    flag_register!(less_than_flag, bool);
    flag_register!(greater_than_flag, bool);

    flag_register!(stack_len, Field);
    flag_register!(call_stack_len, Field);
    flag_register!(pc, Field);
}
