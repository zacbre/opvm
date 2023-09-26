use std::{fmt::Display, ops::Sub};

use super::Object;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Duration(chrono::Duration);
impl Object for Duration {
    fn clone(&self) -> Box<dyn Object> {
        Box::new(Duration(self.0.clone()))
    }
}
impl Duration {
    pub fn new() -> Box<dyn Object> {
        Box::new(Duration(chrono::Duration::zero()))
    }
    pub fn from(d: chrono::Duration) -> Box<dyn Object> {
        Box::new(Duration(d))
    }
}
impl Display for Duration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", chrono::Utc::now().to_rfc3339())
    }
}

impl Sub for Duration {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Duration(self.0 - rhs.0)
    }
}
