use crate::grammar::Token;

#[derive(thiserror::Error, Debug, PartialEq)]
pub enum Error {
    #[error("parse error near {0}")]
    Parse(Token),
}

impl From<Error> for std::io::Error {
    fn from(e: Error) -> std::io::Error {
        match &e {
            Error::Parse(_) => std::io::Error::new(std::io::ErrorKind::InvalidInput, e),
        }
    }
}
