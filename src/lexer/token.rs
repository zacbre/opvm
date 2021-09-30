#[derive(Debug)]
pub struct Token {
    pub content: Option<String>,
    pub token_type: TokenType,
}

#[derive(Debug)]
pub enum TokenType {
    Directive,
    Label,
    Instruction,
    Empty,
    Comment
}