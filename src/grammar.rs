use std::ffi::OsStr;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Input(String),
    Pipe,
    RedirectOut,
    RedirectAppend,
    RedirectIn,
    Background,
    And,
    Or,
    End,
    // TODO:
    // - Subshell
    // - Variable
}

impl AsRef<OsStr> for Token {
    fn as_ref(&self) -> &OsStr {
        match self {
            Token::Input(s) => s.as_ref(),
            Token::Pipe => "|".as_ref(),
            Token::RedirectOut => ">".as_ref(),
            Token::RedirectAppend => ">>".as_ref(),
            Token::RedirectIn => "<".as_ref(),
            Token::And => "&&".as_ref(),
            Token::Or => "||".as_ref(),
            Token::Background => "&".as_ref(),
            Token::End => ";".as_ref(),
        }
    }
}
