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
    Inc,
    Dec,
    Nop,
    Hlt,
    Dup,
    Concat,
    Igl,
    Swap,
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
            "inc" => OpCode::Inc,
            "dec" => OpCode::Dec,
            "nop" => OpCode::Nop,
            "hlt" => OpCode::Hlt,
            "dup" => OpCode::Dup,
            "concat" => OpCode::Concat,
            "swap" => OpCode::Swap,
            _ => OpCode::Igl
        }
    }
}

impl From<OpCode> for &str {
    fn from(opcode: OpCode) -> Self {
        match opcode {
            OpCode::Push => "push",
            OpCode::Pop => "pop",
            OpCode::Add => "add",
            OpCode::Mul => "mul",
            OpCode::Sub => "sub",
            OpCode::Div => "div",
            OpCode::Mod => "mod",
            OpCode::Print => "print",
            OpCode::Call => "call",
            OpCode::Ret => "ret",
            OpCode::Jmp => "jmp",
            OpCode::Je => "je",
            OpCode::Jne => "jne",
            OpCode::Jle => "jle",
            OpCode::Jge => "jge",
            OpCode::Jl => "jl",
            OpCode::Jg => "jg",
            OpCode::Inc => "inc",
            OpCode::Dec => "dec",
            OpCode::Nop => "nop",
            OpCode::Hlt => "hlt",
            OpCode::Dup => "dup",
            OpCode::Igl => "igl",
            OpCode::Concat => "concat",
            OpCode::Swap => "swap"
        }
    }
}