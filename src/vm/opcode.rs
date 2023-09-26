#[derive(Copy, Clone, Debug, PartialEq)]
pub enum OpCode {
    Move,
    Push,
    Pop,
    Add,
    Mul,
    Sub,
    Div,
    Mod,
    Input,
    Call,
    Ret,
    Test,
    Jmp,
    Je,
    Jne,
    Jle,
    Jge,
    Jl,
    Jg,
    Xor,
    Nop,
    Hlt,
    Dup,
    Igl,
    Alloc,
    Free,
    Cast,
    //Load,
    //Store
}

impl From<&str> for OpCode {
    fn from(str: &str) -> Self {
        match str {
            "mov" => OpCode::Move,
            "push" => OpCode::Push,
            "pop" => OpCode::Pop,
            "add" => OpCode::Add,
            "mul" => OpCode::Mul,
            "sub" => OpCode::Sub,
            "div" => OpCode::Div,
            "mod" => OpCode::Mod,
            "input" => OpCode::Input,
            "call" => OpCode::Call,
            "ret" => OpCode::Ret,
            "test" => OpCode::Test,
            "jmp" => OpCode::Jmp,
            "je" => OpCode::Je,
            "jne" => OpCode::Jne,
            "jle" => OpCode::Jle,
            "jge" => OpCode::Jge,
            "jl" => OpCode::Jl,
            "jg" => OpCode::Jg,
            "xor" => OpCode::Xor,
            "nop" => OpCode::Nop,
            "hlt" => OpCode::Hlt,
            "dup" => OpCode::Dup,
            "alloc" => OpCode::Alloc,
            "free" => OpCode::Free,
            "cast" => OpCode::Cast,
            //"load" => OpCode::Load,
            //"store" => OpCode::Store,
            _ => OpCode::Igl
        }
    }
}

impl From<OpCode> for &str {
    fn from(opcode: OpCode) -> Self {
        match opcode {
            OpCode::Move => "mov",
            OpCode::Push => "push",
            OpCode::Pop => "pop",
            OpCode::Add => "add",
            OpCode::Mul => "mul",
            OpCode::Sub => "sub",
            OpCode::Div => "div",
            OpCode::Mod => "mod",
            OpCode::Input => "input",
            OpCode::Call => "call",
            OpCode::Ret => "ret",
            OpCode::Test => "test",
            OpCode::Jmp => "jmp",
            OpCode::Je => "je",
            OpCode::Jne => "jne",
            OpCode::Jle => "jle",
            OpCode::Jge => "jge",
            OpCode::Jl => "jl",
            OpCode::Jg => "jg",
            OpCode::Xor => "xor",
            OpCode::Nop => "nop",
            OpCode::Hlt => "hlt",
            OpCode::Dup => "dup",
            OpCode::Igl => "igl",
            OpCode::Alloc => "alloc",
            OpCode::Free => "free",
            OpCode::Cast => "cast",
            //OpCode::Load => "load",
            //OpCode::Store => "store"
        }
    }
}