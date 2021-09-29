use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Field {
    Number(i32),
    String(String)
}

impl Field {
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            &Field::Number(num) => Some(num),
            _ => None
        }
    }

    pub fn to_str(&self) -> Option<&str> {
        match self {
            &Field::String(ref s) => Some(s),
            _ => None
        }
    }
}

impl From<i32> for Field {
    fn from(i: i32) -> Self {
        Field::Number(i)
    }
}

impl From<String> for Field {
    fn from(s: String) -> Self {
        Field::String(s)
    }
}

impl From<&str> for Field {
    fn from(s: &str) -> Self {
        Field::String(s.to_string())
    }
}

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &Field::Number(i) => write!(f, "{}", i),
            &Field::String(ref s) => write!(f, "{}", s),
        }
    }
}