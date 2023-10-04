use crate::lexer::token::{Token, TokenType};
use crate::vm::field::Field;
use crate::vm::instruction::Instruction;
use crate::vm::program::Program;
use crate::vm::register::{
    self, Register, RegisterOffset, RegisterOffsetOperandType, RegisterWithOffset,
};
use nom::branch::alt;
use nom::bytes::complete::*;
use nom::character::complete::one_of;
use nom::combinator::{eof, opt, peek, value};
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;

pub struct Lexer;

impl Lexer {
    pub fn new() -> Self {
        Lexer {}
    }

    pub fn process(&self, input: String) -> Option<Program> {
        let matched = handle_lines(input.as_str());
        match matched {
            Ok((_, v)) => {
                return Some(self.build(v));
            }
            Err(e) => println!("{:?}", e),
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
                                program.data.insert(
                                    "_".to_owned() + v[0],
                                    Instruction::construct_field(v[1]),
                                );
                            }
                            Err(e) => println!("Error: {:?}", e),
                        }
                    } else {
                        program
                            .labels
                            .insert("_".to_owned() + &token.content.unwrap(), pc);
                    }
                }
                TokenType::Instruction => {
                    let to_parse = token.content.unwrap();
                    let parsed = parse_words(&to_parse);
                    match parsed {
                        Ok((_, v)) => {
                            let mut offsets: Vec<Field> = Vec::new();
                            for item in &v {
                                if let Ok((left, prefix)) = match_operand_prefix(item) {
                                    let output = match_operands(left);
                                    match output {
                                        Ok((_, v)) => {
                                            offsets.push(Field(
                                                crate::types::Type::RegisterWithOffsets(
                                                    RegisterWithOffset::new(
                                                        Register::from(prefix),
                                                        v.iter()
                                                            .map(|(a, b)| RegisterOffset {
                                                                offset:
                                                                    Instruction::construct_field(a),
                                                                operand:
                                                                    RegisterOffsetOperandType::from(
                                                                        *b,
                                                                    ),
                                                            })
                                                            .collect(),
                                                    ),
                                                ),
                                            ));
                                        }
                                        Err(_) => panic!("Error parsing operands!"),
                                    }
                                } else if item != &v[0] {
                                    offsets.push(Instruction::construct_field(item));
                                }
                            }

                            program
                                .instructions
                                .push(Instruction::new_from_fields(v[0], offsets));
                        }
                        Err(e) => println!("Error: {:?}", e),
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

fn match_operand_prefix(i: &str) -> IResult<&str, &str> {
    terminated(take_till(|c| c == '['), tag("["))(i)
}

fn match_operands(i: &str) -> IResult<&str, Vec<(&str, char)>> {
    many0(pair(
        preceded(opt(match_whitespace), take_till(|c| "+-/*%]".contains(c))),
        alt((one_of("+-/*%"), value(char::default(), one_of("]")))),
    ))(i)
}

fn match_newline(i: &str) -> IResult<&str, &str> {
    take_till(|c| c == '\n' || c == ';')(i)
}

fn get_before_colon(i: &str) -> IResult<&str, &str> {
    take_till(|c| c == ':' || c == '\n')(i)
}

fn match_whitespace(i: &str) -> IResult<&str, &str> {
    take_till(|c| c != ' ' && c != '\t')(i)
}

fn match_empty_line(i: &str) -> IResult<&str, Token> {
    build_token(peek(tag("\n"))(i), TokenType::Empty)
}

fn match_blank_line(i: &str) -> IResult<&str, Token> {
    build_token(
        preceded(match_whitespace, alt((peek(tag("\n")), eof)))(i),
        TokenType::Empty,
    )
}

fn match_comments(i: &str) -> IResult<&str, Token> {
    build_token(
        preceded(
            opt(match_whitespace),
            preceded(alt((tag("; "), tag(";"))), take_till(|c| c == '\n')),
        )(i),
        TokenType::Comment,
    )
}

fn match_directive(i: &str) -> IResult<&str, Token> {
    build_token(
        terminated(
            preceded(match_whitespace, preceded(tag("section ."), match_newline)),
            opt(match_comments),
        )(i),
        TokenType::Directive,
    )
}

fn match_label(i: &str) -> IResult<&str, Token> {
    build_token(
        terminated(
            preceded(match_whitespace, preceded(tag("_"), get_before_colon)),
            preceded(tag(":"), alt((match_comments, match_blank_line))),
        )(i),
        TokenType::Label,
    )
}

fn match_label_with_value(i: &str) -> IResult<&str, Token> {
    build_token_vec(
        terminated(
            preceded(
                match_whitespace,
                preceded(tag("_"), parse_words_label_values),
            ),
            alt((match_comments, get_quoted_label, match_blank_line)),
        )(i),
        TokenType::Label,
    )
}

fn match_opcode(i: &str) -> IResult<&str, Token> {
    build_token(
        terminated(match_newline, opt(match_comments))(i),
        TokenType::Instruction,
    )
}

fn handle_lines(i: &str) -> IResult<&str, Vec<Token>> {
    separated_list0(
        tag("\n"),
        alt((
            match_comments,
            match_empty_line,
            match_blank_line,
            match_directive,
            match_label,
            match_label_with_value,
            match_opcode,
        )),
    )(i)
}

fn get_quoted_label(i: &str) -> IResult<&str, Token> {
    build_token(match_words_or_quotes(i), TokenType::Label)
}

fn build_token<'a>(item: IResult<&'a str, &str>, token_type: TokenType) -> IResult<&'a str, Token> {
    match item {
        Ok((i, v)) => Ok((
            i,
            Token {
                content: Some(v.trim().to_string()),
                token_type,
            },
        )),
        Err(e) => Err(e),
    }
}

fn build_token_vec<'a>(
    item: IResult<&'a str, Vec<&str>>,
    token_type: TokenType,
) -> IResult<&'a str, Token> {
    match item {
        Ok((i, v)) => Ok((
            i,
            Token {
                content: Some(v.join(" ")),
                token_type,
            },
        )),
        Err(e) => Err(e),
    }
}

fn get_quoted(i: &str) -> IResult<&str, &str> {
    delimited(
        alt((tag("'"), tag("\""))),
        take_till(|c| c == '\'' || c == '"'),
        alt((tag("'"), tag("\""))),
    )(i)
}

fn match_words_or_quotes(i: &str) -> IResult<&str, &str> {
    if i.starts_with('\'') || i.starts_with('"') {
        get_quoted(i)
    } else {
        take_till(|c| c == ',' || c == ' ' || c == ':' || c == '\n')(i)
    }
}

fn match_quotes(i: &str) -> IResult<&str, &str> {
    if i.starts_with('\'') || i.starts_with('"') {
        preceded(
            alt((peek(tag("'")), peek(tag("\"")))),
            take_till(|c| c == ';' || c == '\n'),
        )(i)
    } else {
        take_till(|c| c == ',' || c == ' ' || c == ':' || c == '\n')(i)
    }
}

fn parse_words(i: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(alt((tag(", "), tag(","), tag(" "))), match_words_or_quotes)(i)
}

fn parse_words_label_values(i: &str) -> IResult<&str, Vec<&str>> {
    separated_list0(
        alt((tag(":"), tag(" "), peek(tag("\"")), peek(tag("'")))),
        terminated(
            preceded(opt(match_whitespace), match_quotes),
            opt(match_comments),
        ),
    )(i)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{types::Type, vm::field::Field};

    #[test]
    fn can_parse_directives() {
        let assm = r#"
        section .data
            _label: 1
        section .code
            _main: 
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.data.len(), 1);
        //assert_eq!(unwrapped.data.get("_label").unwrap(), &Field(Type::Int(1)));
    }

    #[test]
    fn can_parse_labels() {
        let assm = r#"
        section .data
            _label: 1
        section .code
            _main:
                push @label
                print
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.labels.len(), 1);
        assert_eq!(*unwrapped.labels.get("_main").unwrap(), 0 as usize);
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
            section .data; my comment
                _hi: "ayy";comment
                _xdd: 2 ; comment
            section .code        ; comment
                _main: ;comment
                    push 1; comment
                    pop                             ; comment
                    push 2;comment
                    pop;        comment
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.instructions.len(), 4);
        assert_eq!(unwrapped.labels.len(), 1);
        assert_eq!(
            *unwrapped.data.get("_hi").unwrap().to_string(),
            "ayy".to_string()
        );
        if let Type::Int(int) = unwrapped.data.get("_xdd").unwrap().0 {
            assert_eq!(int, 2);
        } else {
            panic!("Expected int!");
        }

        assert_eq!(*unwrapped.labels.get("_main").unwrap(), 0);
    }

    #[test]
    fn can_parse_commas() {
        let assm = r#"
        section .data
            _label: 1
        section .code
            _main:
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
        section .data
            _label: 1
        section .code
            _main:
                mov ra[2],0
        "#;
        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        assert_eq!(unwrapped.labels.len(), 1);
        println!("{:?}", unwrapped.instructions[0].operand);
        //assert_eq!(unwrapped.instructions[0].operand, 1);
    }

    #[test]
    fn parse_words_can_parse_spaces_or_commas() {
        let assm = r#"
        section .code
            _main:
                mov ra,1
                mov rb,1 a a a a
        "#;

        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let unwrapped = instructions.unwrap();
        println!("{:?}", unwrapped.instructions);
        assert_eq!(2, unwrapped.instructions.len());
        assert_eq!(2, unwrapped.instructions[0].operand.len());
    }

    #[test]
    fn parse_words_can_parse_spaces_after_commas() {
        let assm = r#"
        section .code
            _main:
                mov ra, 1
                mov rb, 1
        "#;

        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let mut unwrapped = instructions.unwrap();
        println!("{:?}", unwrapped.instructions);
        assert_eq!(2, unwrapped.instructions.len());
        assert_eq!(2, unwrapped.instructions[0].operand.len());
        assert_eq!(
            Field(Type::Int(1)),
            unwrapped.instructions[0].operand.pop().unwrap()
        );
        assert_eq!(
            Field(Type::Register(Register::Ra)),
            unwrapped.instructions[0].operand.pop().unwrap()
        );
    }

    #[test]
    fn can_parse_offsets() {
        let assm = r#"
        section .code
            _main:
                mov ra[ra-rb+rc], rb[r0]
                mov ra[ra], rb[ra+rb]
                mov ra[ra-2], rb[1]
                mov r2, rb[1]
        "#;

        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let mut unwrapped = instructions.unwrap();
        println!("{:?}", unwrapped.instructions);
        assert_eq!(4, unwrapped.instructions.len());
        assert_eq!(2, unwrapped.instructions[0].operand.len());
        assert_eq!(2, unwrapped.instructions[1].operand.len());
        assert_eq!(2, unwrapped.instructions[2].operand.len());
        assert_eq!(2, unwrapped.instructions[3].operand.len());

        //mov ra[ra-rb+rc], rb[r0]
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Rb,
                vec![RegisterOffset {
                    offset: Field(Type::Register(Register::R0)),
                    operand: RegisterOffsetOperandType::None
                }]
            ))),
            unwrapped.instructions[0].operand.pop().unwrap()
        );
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Ra,
                vec![
                    RegisterOffset {
                        offset: Field(Type::Register(Register::Ra)),
                        operand: RegisterOffsetOperandType::Sub
                    },
                    RegisterOffset {
                        offset: Field(Type::Register(Register::Rb)),
                        operand: RegisterOffsetOperandType::Add
                    },
                    RegisterOffset {
                        offset: Field(Type::Register(Register::Rc)),
                        operand: RegisterOffsetOperandType::None
                    }
                ]
            ))),
            unwrapped.instructions[0].operand.pop().unwrap()
        );

        // mov ra[ra], rb[ra+rb]
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Rb,
                vec![
                    RegisterOffset {
                        offset: Field(Type::Register(Register::Ra)),
                        operand: RegisterOffsetOperandType::Add
                    },
                    RegisterOffset {
                        offset: Field(Type::Register(Register::Rb)),
                        operand: RegisterOffsetOperandType::None
                    }
                ]
            ))),
            unwrapped.instructions[1].operand.pop().unwrap()
        );
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Ra,
                vec![RegisterOffset {
                    offset: Field(Type::Register(Register::Ra)),
                    operand: RegisterOffsetOperandType::None
                }]
            ))),
            unwrapped.instructions[1].operand.pop().unwrap()
        );

        // mov ra[ra-2], rb[1]
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Rb,
                vec![RegisterOffset {
                    offset: Field(Type::Int(1)),
                    operand: RegisterOffsetOperandType::None
                }]
            ))),
            unwrapped.instructions[2].operand.pop().unwrap()
        );
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Ra,
                vec![
                    RegisterOffset {
                        offset: Field(Type::Register(Register::Ra)),
                        operand: RegisterOffsetOperandType::Sub
                    },
                    RegisterOffset {
                        offset: Field(Type::Int(2)),
                        operand: RegisterOffsetOperandType::None
                    }
                ]
            ))),
            unwrapped.instructions[2].operand.pop().unwrap()
        );

        // mov r2, rb[1]
        assert_eq!(
            Field(Type::RegisterWithOffsets(RegisterWithOffset::new(
                Register::Rb,
                vec![RegisterOffset {
                    offset: Field(Type::Int(1)),
                    operand: RegisterOffsetOperandType::None
                }]
            ))),
            unwrapped.instructions[3].operand.pop().unwrap()
        );
        assert_eq!(
            Field(Type::Register(Register::R2)),
            unwrapped.instructions[3].operand.pop().unwrap()
        );
    }

    #[test]
    fn can_assert_chars() {
        let assm = r#"
        section .code
            _main:
                mov ra[0], 'a'
        "#;

        let instructions = Lexer::new().process(assm.to_string());
        assert!(instructions.is_some());
        let mut unwrapped = instructions.unwrap();
        println!("{:?}", unwrapped.instructions);
        assert_eq!(1, unwrapped.instructions.len());
        assert_eq!(2, unwrapped.instructions[0].operand.len());
        assert_eq!(Field(Type::Char('a')), unwrapped.instructions[0].operand.pop().unwrap());
    }
}
