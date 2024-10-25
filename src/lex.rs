use std::io;

use crate::{grammar::Token, input};

pub struct Lexer;

impl Lexer {
    pub fn lex(line: &str) -> io::Result<Vec<Token>> {
        let mut tokens = vec![];
        let mut token = String::new();
        let mut escape = false;
        let mut in_double_quotes = false;
        let mut in_single_quotes = false;

        let mut iter = line.chars().peekable();

        while let Some(c) = iter.next() {
            if escape {
                token.push(c);
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' && !in_single_quotes {
                in_double_quotes = !in_double_quotes;
                if !in_double_quotes {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
            } else if c == '\'' && !in_double_quotes {
                in_single_quotes = !in_single_quotes;
                if !in_single_quotes {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
            } else if (c == ' ' || c == '\t') && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
            } else if c == ';' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                tokens.push(Token::End);
            } else if c == '|' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                if iter.peek() == Some(&'|') {
                    iter.next();
                    if iter.peek() == Some(&'|') {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "parse error near `|`",
                        ));
                    }
                    tokens.push(Token::Or);
                } else {
                    tokens.push(Token::Pipe);
                }
            } else if c == '>' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                if iter.peek() == Some(&'>') {
                    iter.next();
                    if iter.peek() == Some(&'>') {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "parse error near `>`",
                        ));
                    }
                    tokens.push(Token::RedirectAppend);
                } else {
                    tokens.push(Token::RedirectOut);
                }
            } else if c == '&' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                if iter.peek() == Some(&'&') {
                    iter.next();
                    if iter.peek() == Some(&'&') {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            "parse error near `&`",
                        ));
                    }
                    tokens.push(Token::And);
                } else {
                    tokens.push(Token::Background);
                }
            } else if c == '<' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                tokens.push(Token::RedirectIn);
            } else if c == '(' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                tokens.push(Token::OpenParenthesis);
            } else if c == ')' && !in_single_quotes && !in_double_quotes {
                if !token.is_empty() {
                    tokens.push(input!(std::mem::take(&mut token)));
                }
                tokens.push(Token::CloseParenthesis);
            } else {
                token.push(c);
            }
        }

        if !token.is_empty() {
            tokens.push(input!(token));
        }

        Ok(tokens)
    }
}

pub fn is_operator(token: &Token) -> bool {
    !matches!(token, Token::Input(_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_operators() {
        let line = "|";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::Pipe]);

        let line = ">";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::RedirectOut]);

        let line = ">>";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::RedirectAppend]);

        let line = "<";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::RedirectIn]);

        let line = "&&";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::And]);

        let line = "||";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::Or]);

        let line = "&";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::Background]);

        let line = ";";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![Token::End]);
    }

    #[test]
    fn test_lex() {
        let line = "echo";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![input!("echo")]);

        let line = "echo foo";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![input!("echo"), input!("foo")]);

        let line = "echo foo bar";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![input!("echo"), input!("foo"), input!("bar")]);

        let line = "echo Hello, World!";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(
            tokens,
            vec![input!("echo"), input!("Hello,"), input!("World!")]
        );

        let line = "echo \"Hello, World!\"";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![input!("echo"), input!("Hello, World!")]);

        let line = "echo \'Hello, World!\'";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![input!("echo"), input!("Hello, World!")]);

        let line = "echo \'Hello,\\ World!\'";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(tokens, vec![input!("echo"), input!("Hello, World!")]);

        let line = "echo Hello, World! > output.txt";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(
            tokens,
            vec![
                input!("echo"),
                input!("Hello,"),
                input!("World!"),
                Token::RedirectOut,
                input!("output.txt")
            ]
        );

        let line = "echo Hello, World! | wc -w";
        let tokens = Lexer::lex(line).unwrap();
        assert_eq!(
            tokens,
            vec![
                input!("echo"),
                input!("Hello,"),
                input!("World!"),
                Token::Pipe,
                input!("wc"),
                input!("-w")
            ]
        );
    }
}
