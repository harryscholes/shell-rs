use crate::{ast::Ast, grammar::Token, lex::is_operator};

pub struct Parser;

impl Parser {
    pub fn parse(tokens: &[Token]) -> Option<Ast> {
        let mut i = 0;
        let mut nodes = vec![];

        while i < tokens.len() {
            let token = &tokens[i];

            match token {
                Token::Pipe => {
                    let left = nodes.pop()?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::Pipe {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                Token::RedirectOut => {
                    let left = nodes.pop()?;
                    let right = tokens[i + 1].clone();
                    i += 1;
                    nodes.push(Ast::RedirectOut {
                        left: Box::new(left),
                        right,
                    });
                }
                Token::RedirectAppend => {
                    let left = nodes.pop()?;
                    let right = tokens[i + 1].clone();
                    i += 1;
                    nodes.push(Ast::RedirectAppend {
                        left: Box::new(left),
                        right,
                    });
                }
                Token::And => {
                    let left = nodes.pop()?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::And {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                Token::Or => {
                    let left = nodes.pop()?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::Or {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                Token::End => {
                    let left = nodes.pop()?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::Sequence {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                t if !is_operator(t) => {
                    let command = parse_command(&tokens[i..]);
                    i += command.args.len();
                    nodes.push(command.into());
                }
                _ => {
                    panic!("Unsupported token: {:?}", token)
                }
            }

            i += 1;
        }

        nodes.pop()
    }
}

fn parse_command(tokens: &[Token]) -> Command {
    let command = tokens[0].clone();
    let mut args = vec![];

    for token in &tokens[1..] {
        if is_operator(token) {
            break;
        }

        args.push(token.clone());
    }

    Command { command, args }
}

#[derive(Debug)]
struct Command {
    command: Token,
    args: Vec<Token>,
}

impl From<Command> for Ast {
    fn from(command: Command) -> Ast {
        Ast::Command {
            command: command.command,
            args: command.args,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::input;

    #[test]
    fn test_ast() {
        let tokens = vec![
            input!("ls"),
            input!("-l"),
            Token::Pipe,
            input!("grep"),
            input!("main"),
        ];
        let ast = Parser::parse(&tokens).unwrap();

        match ast {
            Ast::Pipe { left, right } => {
                match *left {
                    Ast::Command { command, args } => {
                        assert_eq!(command, input!("ls"));
                        assert_eq!(args, vec![input!("-l")]);
                    }
                    _ => panic!("Expected Command"),
                }

                match *right {
                    Ast::Command { command, args } => {
                        assert_eq!(command, input!("grep"));
                        assert_eq!(args, vec![input!("main")]);
                    }
                    _ => panic!("Expected Command"),
                }
            }
            _ => panic!("Expected Pipe"),
        }
    }
}
