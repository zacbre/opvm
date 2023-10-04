use std::{fmt::Display, ops::Sub};

use super::{duration::Duration, Object};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Date(chrono::DateTime<chrono::Utc>);
impl Object for Date {
    fn clone(&self) -> Box<dyn Object> {
        Box::new(Date(self.0.clone()))
    }
}
impl Date {
    #[allow(dead_code)]
    pub fn format(&self, fmt: &str) -> String {
        self.0.format(fmt).to_string()
    }
    pub fn new() -> Box<dyn Object> {
        Box::new(Date(chrono::Utc::now()))
    }
}
impl Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", chrono::Utc::now().to_rfc3339())
    }
}

impl Sub for Date {
    type Output = Box<dyn Object>;

    fn sub(self, rhs: Self) -> Box<dyn Object> {
        Duration::from(self.0 - rhs.0)
    }
}
