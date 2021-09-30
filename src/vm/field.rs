use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Field {
    I(i64),
    U(usize),
    S(String)
}

impl Field {
    pub fn to_i(&self) -> Option<i64> {
        match self {
            &Field::I(num) => Some(num),
            _ => None
        }
    }

    pub fn to_str(&self) -> Option<&str> {
        match self {
            &Field::S(ref s) => Some(s),
            _ => None
        }
    }

    pub fn to_s(&self) -> Option<String> {
        match self {
            &Field::S(ref s) => Some(s.to_string()),
            _ => None
        }
    }

    pub fn to_u(&self) -> Option<usize> {
        match self {
            &Field::U(u) => Some(u),
            _ => None
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

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Field::I(i) => write!(f, "{}", i),
            &Field::U(u) => write!(f, "{}", u),
            &Field::S(ref s) => {
                write!(f, "{}", s)
            },
        }
    }
}