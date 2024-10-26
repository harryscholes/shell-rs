use crate::{ast::Ast, error::Error, grammar::Token, lex::is_operator};

pub struct Parser;

impl Parser {
    pub fn parse(tokens: &[Token]) -> Result<Ast, Error> {
        let mut i = 0;
        let mut nodes = vec![];

        while i < tokens.len() {
            let token = &tokens[i];

            match token {
                Token::Pipe => {
                    let left = nodes.pop().ok_or(Error::Parse(Token::Pipe))?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::Pipe {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                Token::RedirectOut => {
                    let left = nodes.pop().ok_or(Error::Parse(Token::RedirectOut))?;
                    let right = tokens[i + 1].clone();
                    i += 1;
                    nodes.push(Ast::RedirectOut {
                        left: Box::new(left),
                        right,
                    });
                }
                Token::RedirectAppend => {
                    let left = nodes.pop().ok_or(Error::Parse(Token::RedirectAppend))?;
                    let right = tokens[i + 1].clone();
                    i += 1;
                    nodes.push(Ast::RedirectAppend {
                        left: Box::new(left),
                        right,
                    });
                }
                Token::And => {
                    let left = nodes.pop().ok_or(Error::Parse(Token::And))?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::And {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                Token::Or => {
                    let left = nodes.pop().ok_or(Error::Parse(Token::Or))?;
                    let command = parse_command(&tokens[i + 1..]);
                    i += command.args.len() + 1;
                    nodes.push(Ast::Or {
                        left: Box::new(left),
                        right: Box::new(command.into()),
                    });
                }
                Token::Semicolon => {
                    let right_tokens = &tokens[i + 1..];
                    if !right_tokens.is_empty() {
                        let left = nodes.pop().ok_or(Error::Parse(Token::Semicolon))?;
                        let command = parse_command(right_tokens);
                        i += command.args.len() + 1;
                        nodes.push(Ast::Sequence {
                            left: Box::new(left),
                            right: Box::new(command.into()),
                        });
                    }
                }
                Token::OpenParenthesis => {
                    let (subshell, l) = Self::parse_subshell(&tokens[i + 1..]);
                    let subshell = subshell?;
                    i += l + 1;
                    nodes.push(Ast::Subshell {
                        inner: Box::new(subshell),
                    });
                }
                Token::CloseParenthesis => {
                    Err(Error::Parse(Token::CloseParenthesis))?;
                }
                Token::Background => {
                    let left = nodes.pop().ok_or(Error::Parse(Token::Background))?;
                    nodes.push(Ast::Background {
                        inner: Box::new(left),
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

        nodes.pop().ok_or(Error::Parse(Token::Input(String::new())))
    }

    fn parse_subshell(tokens: &[Token]) -> (Result<Ast, Error>, usize) {
        let mut inner_tokens = vec![];
        let mut paren_level = 1;

        for token in tokens {
            match token {
                Token::OpenParenthesis => {
                    paren_level += 1;
                    inner_tokens.push(token.clone());
                }
                Token::CloseParenthesis => {
                    paren_level -= 1;
                    if paren_level == 0 {
                        break;
                    }
                    inner_tokens.push(token.clone());
                }
                _ => {
                    inner_tokens.push(token.clone());
                }
            }
        }

        let l = inner_tokens.len();

        (Self::parse(&inner_tokens), l)
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
        // ls -l | grep main
        let tokens = vec![
            input!("ls"),
            input!("-l"),
            Token::Pipe,
            input!("grep"),
            input!("main"),
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Pipe {
                left: Box::new(Ast::Command {
                    command: input!("ls"),
                    args: vec![input!("-l")],
                }),
                right: Box::new(Ast::Command {
                    command: input!("grep"),
                    args: vec![input!("main")],
                }),
            }
        );
    }

    #[test]
    fn test_subshell() {
        // (echo foo)
        let tokens = vec![
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Subshell {
                inner: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("foo")],
                }),
            }
        );

        // (echo foo;)
        let tokens = vec![
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::Semicolon,
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Subshell {
                inner: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("foo")],
                }),
            }
        );

        // (((echo foo)))
        let tokens = vec![
            Token::OpenParenthesis,
            Token::OpenParenthesis,
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::CloseParenthesis,
            Token::CloseParenthesis,
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Subshell {
                inner: Box::new(Ast::Subshell {
                    inner: Box::new(Ast::Subshell {
                        inner: Box::new(Ast::Command {
                            command: input!("echo"),
                            args: vec![input!("foo")],
                        }),
                    }),
                }),
            }
        );

        // (echo foo | cat)
        let tokens = vec![
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::Pipe,
            input!("cat"),
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Subshell {
                inner: Box::new(Ast::Pipe {
                    left: Box::new(Ast::Command {
                        command: input!("echo"),
                        args: vec![input!("foo")],
                    }),
                    right: Box::new(Ast::Command {
                        command: input!("cat"),
                        args: vec![],
                    }),
                }),
            }
        );

        // (echo foo | cat) | cat
        let tokens = vec![
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::Pipe,
            input!("cat"),
            Token::CloseParenthesis,
            Token::Pipe,
            input!("cat"),
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Pipe {
                left: Box::new(Ast::Subshell {
                    inner: Box::new(Ast::Pipe {
                        left: Box::new(Ast::Command {
                            command: input!("echo"),
                            args: vec![input!("foo")],
                        }),
                        right: Box::new(Ast::Command {
                            command: input!("cat"),
                            args: vec![],
                        }),
                    }),
                }),
                right: Box::new(Ast::Command {
                    command: input!("cat"),
                    args: vec![],
                }),
            }
        );

        // ((echo foo | cat) | cat)
        let tokens = vec![
            Token::OpenParenthesis,
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::Pipe,
            input!("cat"),
            Token::CloseParenthesis,
            Token::Pipe,
            input!("cat"),
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Subshell {
                inner: Box::new(Ast::Pipe {
                    left: Box::new(Ast::Subshell {
                        inner: Box::new(Ast::Pipe {
                            left: Box::new(Ast::Command {
                                command: input!("echo"),
                                args: vec![input!("foo")],
                            }),
                            right: Box::new(Ast::Command {
                                command: input!("cat"),
                                args: vec![],
                            }),
                        }),
                    }),
                    right: Box::new(Ast::Command {
                        command: input!("cat"),
                        args: vec![],
                    }),
                }),
            }
        );
    }

    #[test]
    fn test_subshell_parentheses_balanced() {
        // (echo foo))
        let tokens = vec![
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::CloseParenthesis,
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens);
        assert_eq!(ast, Err(Error::Parse(Token::CloseParenthesis)));
    }

    #[test]
    fn test_semicolon() {
        // echo foo;
        let tokens = vec![input!("echo"), input!("foo"), Token::Semicolon];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Command {
                command: input!("echo"),
                args: vec![input!("foo")],
            }
        );

        // echo foo; echo bar
        let tokens = vec![
            input!("echo"),
            input!("foo"),
            Token::Semicolon,
            input!("echo"),
            input!("bar"),
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Sequence {
                left: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("foo")],
                }),
                right: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("bar")],
                }),
            }
        );

        // echo foo; echo bar;
        let tokens = vec![
            input!("echo"),
            input!("foo"),
            Token::Semicolon,
            input!("echo"),
            input!("bar"),
            Token::Semicolon,
        ];
        let ast = Parser::parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Sequence {
                left: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("foo")],
                }),
                right: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("bar")],
                }),
            }
        );
    }
}
