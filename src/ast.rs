use crate::grammar::Token;

#[derive(Debug)]
pub enum Ast {
    Command { command: Token, args: Vec<Token> },
    Pipe { left: Box<Ast>, right: Box<Ast> },
    RedirectOut { left: Box<Ast>, right: Token },
    RedirectAppend { left: Box<Ast>, right: Token },
    And { left: Box<Ast>, right: Box<Ast> },
    Or { left: Box<Ast>, right: Box<Ast> },
    Sequence { left: Box<Ast>, right: Box<Ast> },
}
