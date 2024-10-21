pub mod ast;
pub mod exec;
pub mod grammar;
pub mod lex;
pub mod parse;
pub mod pipeline;

#[macro_export]
macro_rules! input {
    ($token:expr) => {
        crate::grammar::Token::Input($token.to_string())
    };
}
