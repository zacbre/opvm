#[derive(Debug)]
pub struct Error {
    pub message: String,
    pub stacktrace: Vec<String>,
    pub app_stack: Vec<String>
}

impl Error {
    pub fn new(message: String, stack: Vec<String>, app_stack: Vec<String>) -> Self {
        Error {
            message,
            stacktrace: stack,
            app_stack
        }
    }
}