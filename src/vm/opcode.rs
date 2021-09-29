#[derive(Copy, Clone)]
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
    Label
}