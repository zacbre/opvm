use nom::branch::alt;
use nom::IResult;
use nom::bytes::complete::*;
use nom::multi::{separated_list0};
use nom::sequence::{delimited, preceded, terminated};
use nom::combinator::{eof, opt, peek};
use crate::lexer::token::{Token, TokenType};
use crate::vm::instruction::Instruction;
use crate::vm::program::Program;

pub struct Lexer {

}

impl Lexer {
    pub fn new() -> Self {
        Lexer{}
    }

    pub fn process(&self, input: String) -> Option<Program> {
        let matched = handle_lines(input.as_str());
        match matched {
            Ok((_, v)) => {
                return Some(self.build(v));
            }
            Err(e) => println!("{:?}", e)
        }

        None
    }

    fn build(&self, tokens: Vec<Token>) -> Program {
        let mut pc: usize = 0;
        let mut program = Program::new();
        let mut current_directive = String::default();
        for token in tokens {
            match token.token_type {
                TokenType::Directive => {
                    current_directive = token.content.unwrap();
                }
                TokenType::Label => {
                    if current_directive == "data" {
                        let to_parse = token.content.unwrap();
                        let parsed = parse_words(&to_parse);
                        match parsed {
                            Ok((_, v)) => {
                                program.data.insert("@".to_owned()+v[0], Instruction::construct_field(v[1]));
                            }
                            Err(e) => println!("Error: {:?}", e)
                        }
                    } else {
                        program.labels.insert("@".to_owned() + &token.content.unwrap(), pc);
                    }
                }
                TokenType::Instruction => {
                    let to_parse = token.content.unwrap();
                    let parsed = parse_words(&to_parse);
                    match parsed {
                        Ok((_, v)) => {
                            program.instructions.push(Instruction::new_from_words(v))
                        }
                        Err(e) => println!("Error: {:?}", e)
                    }
                    pc += 1;
                }
                TokenType::Empty => {}
                TokenType::Comment => {}
            }
        }

        program
    }
}

fn match_newline(i: &str) -> IResult<&str,&str> {
    take_till(|c| c == '\n' || c == ';')(i)
}

fn match_whitespace(i: &str) -> IResult<&str,&str> {
    take_till(|c| c != ' ' && c != '\t')(i)
}

fn match_empty_line(i: &str) -> IResult<&str, Token> {
    build_token(peek(tag("\n"))(i), TokenType::Empty)
}

fn match_blank_line(i: &str) -> IResult<&str, Token> {
    build_token(preceded(match_whitespace, alt((peek(tag("\n")), eof)))(i), TokenType::Empty)
}

fn match_comments(i: &str) -> IResult<&str, Token> {
    build_token(preceded(opt(match_whitespace), preceded(alt((tag("; "), tag(";"))), take_till(|c| c == '\n')))(i), TokenType::Comment)
}

fn match_directive(i: &str) -> IResult<&str, Token> {
    build_token(terminated(preceded(match_whitespace, preceded(tag("#"), match_newline)), opt(match_comments))(i), TokenType::Directive)
}

fn match_label(i: &str) -> IResult<&str, Token> {
    build_token(terminated(preceded(match_whitespace, preceded(tag("."), match_newline)), opt(match_comments))(i), TokenType::Label)
}

fn match_opcode(i: &str) -> IResult<&str, Token> {
    build_token(terminated(match_newline, opt(match_comments))(i), TokenType::Instruction)
}

fn handle_lines(i: &str) -> IResult<&str, Vec<Token>> {
    separated_list0(tag("\n"), alt((match_comments, match_empty_line, match_blank_line, match_directive, match_label, match_opcode)))(i)
}

fn build_token<'a>(item: IResult<&'a str,&str>, token_type: TokenType) -> IResult<&'a str, Token> {
    match item {
        Ok((i, v)) => {
            Ok((i, Token{
                content: Some(v.trim().to_string()),
                token_type
            }))
        }
        Err(e) => Err(e)
    }
}

fn match_word(i: &str) -> IResult<&str, &str> {
    take_till(|c| c == ' ' || c == ',' || c == '\n')(i)
}

fn get_quoted(i: &str) -> IResult<&str, &str> {
    delimited(
        alt((tag("'"), tag("\""))),
        take_till(|c| c == '\'' || c == '"'),
        alt((tag("'"), tag("\""))),
    )(i)
}

fn match_quote(i: &str) -> IResult<&str, &str> {
    if i.starts_with('\'') || i.starts_with('"') {
        get_quoted(i)
    } else {
        is_not(" ")(i)
    }
}

fn parse_words(i: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(tag(" "), alt((match_quote, match_word)))(i)
}

#[cfg(test)]
mod test {
    use crate::vm::field::Field;
    use super::*;

    #[test]
    fn can_parse_directives() {
        let assm = r#"
        #data
            .label 1
        #code
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.data.len(), 1);
        assert_eq!(unwrapped.data.get("@label").unwrap(), &Field::I(1));
    }

    #[test]
    fn can_parse_labels() {
        let assm = r#"
        #data
            .label 1
        #code
            .main
            push @label
            print
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.labels.len(), 1);
        assert_eq!(*unwrapped.labels.get("@main").unwrap(), 0 as usize);
    }

    #[test]
    fn can_ignore_comments() {
        let assm = r#"
        ; this is a test comment!
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.instructions.len(), 0);
    }

    #[test]
    fn can_ignore_empty_lines() {
        let assm = r#"

        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.instructions.len(), 0);
    }

    #[test]
    fn can_have_comments_on_lines() {
        let assm = r#"
            #data; my comment
                .hi "ayy";comment
                .xdd 2 ; comment
            #code        ; comment
                .main ;comment
                    push 1; comment
                    pop; comment
                    push 2;comment
                    pop;        comment
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.instructions.len(), 4);
        assert_eq!(unwrapped.labels.len(), 1);
        assert_eq!(*unwrapped.data.get("@hi").unwrap(), Field::from("ayy"));
        assert_eq!(*unwrapped.data.get("@xdd").unwrap(), Field::from(2));
        assert_eq!(*unwrapped.labels.get("@main").unwrap(), 0);
    }

    #[test]
    fn can_parse_commas() {
        let assm = r#"
        #data
            .label 1
        #code
            .main
            mov ra,0
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.labels.len(), 1);
        println!("{:?}", unwrapped.instructions[0].operand);
        //assert_eq!(unwrapped.instructions[0].operand, 1);
    }

    #[test]
    fn can_parse_commas_with_offsets() {
        let assm = r#"
        #data
            .label 1
        #code
            .main
            mov ra[2],0
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.labels.len(), 1);
        println!("{:?}", unwrapped.instructions[0].operand);
        //assert_eq!(unwrapped.instructions[0].operand, 1);
    }
}