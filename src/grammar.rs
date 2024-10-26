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
    Semicolon,
    OpenParenthesis,
    CloseParenthesis,
    // TODO:
    // - Variable
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_ref())
    }
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
            Token::Semicolon => ";".as_ref(),
            Token::OpenParenthesis => "(".as_ref(),
            Token::CloseParenthesis => ")".as_ref(),
        }
    }
}
