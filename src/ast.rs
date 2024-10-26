use crate::grammar::Token;

#[derive(Debug, PartialEq)]
pub enum Ast {
    Command { command: Token, args: Vec<Token> },
    Pipe { left: Box<Ast>, right: Box<Ast> },
    RedirectOut { left: Box<Ast>, right: Token },
    RedirectAppend { left: Box<Ast>, right: Token },
    And { left: Box<Ast>, right: Box<Ast> },
    Or { left: Box<Ast>, right: Box<Ast> },
    Sequence { left: Box<Ast>, right: Box<Ast> },
    Subshell { inner: Box<Ast> },
    Background { inner: Box<Ast> },
}
