use std::char::ParseCharError;
use std::num::ParseIntError;

#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub stacktrace: Vec<String>,
    pub app_stack: Vec<String>,
}

impl Error {
    pub fn new(message: String, stack: Vec<String>, app_stack: Vec<String>) -> Self {
        Error {
            message,
            stacktrace: stack,
            app_stack,
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Error::new(value.to_string(), vec![], vec![])
    }
}

impl From<ParseCharError> for Error {
    fn from(value: ParseCharError) -> Self {
        Error::new(value.to_string(), vec![], vec![])
    }
}
