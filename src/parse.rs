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
                    let right_tokens = &tokens[i + 1..];
                    if right_tokens.is_empty() {
                        nodes.push(Ast::Sequence {
                            left: Box::new(left),
                            right: Box::new(Ast::Empty),
                        });
                    } else {
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
                    i += l + 1;
                    if let Some(subshell) = subshell {
                        nodes.push(Ast::Subshell {
                            inner: Box::new(subshell),
                        });
                    }
                }
                Token::CloseParenthesis => {
                    i += 1;
                    break;
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

    fn parse_subshell(tokens: &[Token]) -> (Option<Ast>, usize) {
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

        match ast {
            Ast::Subshell { inner } => match *inner {
                Ast::Command { command, args } => {
                    assert_eq!(command, input!("echo"));
                    assert_eq!(args, vec![input!("foo")]);
                }
                _ => panic!("Expected Command"),
            },
            _ => panic!("Expected Subshell"),
        }

        // (echo foo;)
        let tokens = vec![
            Token::OpenParenthesis,
            input!("echo"),
            input!("foo"),
            Token::End,
            Token::CloseParenthesis,
        ];
        let ast = Parser::parse(&tokens).unwrap();

        match ast {
            Ast::Subshell { inner } => match *inner {
                Ast::Sequence { left, right } => {
                    match *left {
                        Ast::Command { command, args } => {
                            assert_eq!(command, input!("echo"));
                            assert_eq!(args, vec![input!("foo")]);
                        }
                        _ => panic!("Expected Command"),
                    };

                    match *right {
                        Ast::Empty => {}
                        _ => panic!("Expected Empty"),
                    }
                }
                _ => panic!("Expected Sequence"),
            },
            _ => panic!("Expected Subshell"),
        }

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

        match ast {
            Ast::Subshell { inner } => match *inner {
                Ast::Subshell { inner } => match *inner {
                    Ast::Subshell { inner } => match *inner {
                        Ast::Command { command, args } => {
                            assert_eq!(command, input!("echo"));
                            assert_eq!(args, vec![input!("foo")]);
                        }
                        _ => panic!("Expected Command"),
                    },
                    _ => panic!("Expected Subshell"),
                },
                _ => panic!("Expected Subshell"),
            },
            _ => panic!("Expected Subshell"),
        }

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

        match ast {
            Ast::Subshell { inner } => match *inner {
                Ast::Pipe { left, right } => {
                    match *left {
                        Ast::Command { command, args } => {
                            assert_eq!(command, input!("echo"));
                            assert_eq!(args, vec![input!("foo")]);
                        }
                        _ => panic!("Expected Command"),
                    };
                    match *right {
                        Ast::Command { command, args } => {
                            assert_eq!(command, input!("cat"));
                            assert_eq!(args, vec![]);
                        }
                        _ => panic!("Expected Command"),
                    };
                }
                _ => panic!("Expected Pipe"),
            },
            _ => panic!("Expected Subshell"),
        }

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

        match ast {
            Ast::Pipe { left, right } => {
                match *left {
                    Ast::Subshell { inner } => match *inner {
                        Ast::Pipe { left, right } => {
                            match *left {
                                Ast::Command { command, args } => {
                                    assert_eq!(command, input!("echo"));
                                    assert_eq!(args, vec![input!("foo")]);
                                }
                                _ => panic!("Expected Command"),
                            };
                            match *right {
                                Ast::Command { command, args } => {
                                    assert_eq!(command, input!("cat"));
                                    assert_eq!(args, vec![]);
                                }
                                _ => panic!("Expected Command"),
                            };
                        }
                        _ => panic!("Expected Pipe"),
                    },
                    _ => panic!("Expected Subshell"),
                };
                match *right {
                    Ast::Command { command, args } => {
                        assert_eq!(command, input!("cat"));
                        assert_eq!(args, vec![]);
                    }
                    _ => panic!("Expected Command"),
                };
            }
            _ => panic!("Expected Pipe"),
        }

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

        match ast {
            Ast::Subshell { inner } => match *inner {
                Ast::Pipe { left, right } => {
                    match *left {
                        Ast::Subshell { inner } => match *inner {
                            Ast::Pipe { left, right } => {
                                match *left {
                                    Ast::Command { command, args } => {
                                        assert_eq!(command, input!("echo"));
                                        assert_eq!(args, vec![input!("foo")]);
                                    }
                                    _ => panic!("Expected Command"),
                                };
                                match *right {
                                    Ast::Command { command, args } => {
                                        assert_eq!(command, input!("cat"));
                                        assert_eq!(args, vec![]);
                                    }
                                    _ => panic!("Expected Command"),
                                };
                            }
                            _ => panic!("Expected Pipe"),
                        },
                        _ => panic!("Expected Subshell"),
                    };

                    match *right {
                        Ast::Command { command, args } => {
                            assert_eq!(command, input!("cat"));
                            assert_eq!(args, vec![]);
                        }
                        _ => panic!("Expected Command"),
                    };
                }
                _ => panic!("Expected Pipe"),
            },
            _ => panic!("Expected Subshell"),
        }
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
        assert!(Parser::parse(&tokens).is_none());
    }

    #[test]
    fn test_end() {
        // echo foo;
        let tokens = vec![input!("echo"), input!("foo"), Token::End];
        let ast = Parser::parse(&tokens).unwrap();

        match ast {
            Ast::Sequence { left, right } => {
                match *left {
                    Ast::Command { command, args } => {
                        assert_eq!(command, input!("echo"));
                        assert_eq!(args, vec![input!("foo")]);
                    }
                    _ => panic!("Expected Command"),
                }

                match *right {
                    Ast::Empty => {}
                    _ => panic!("Expected Command"),
                }
            }
            _ => panic!("Expected Sequence"),
        }
    }
}
