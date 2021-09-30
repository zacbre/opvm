#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    Push,
    Pop,
    Add,
    Mul,
    Sub,
    Div,
    Mod,
    Print,
    Call,
    Ret,
    Jmp,
    Je,
    Jne,
    Jle,
    Jge,
    Jl,
    Jg,
    Unknown
}

impl From<&str> for OpCode {
    fn from(str: &str) -> Self {
        match str {
            "push" => OpCode::Push,
            "pop" => OpCode::Pop,
            "add" => OpCode::Add,
            "mul" => OpCode::Mul,
            "sub" => OpCode::Sub,
            "div" => OpCode::Div,
            "mod" => OpCode::Mod,
            "print" => OpCode::Print,
            "call" => OpCode::Call,
            "ret" => OpCode::Ret,
            "jmp" => OpCode::Jmp,
            "je" => OpCode::Je,
            "jne" => OpCode::Jne,
            "jle" => OpCode::Jle,
            "jge" => OpCode::Jge,
            "jl" => OpCode::Jl,
            "jg" => OpCode::Jg,
            _ => OpCode::Unknown
        }
    }
}