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
        write!(f, "{}", self.as_ref().to_string_lossy())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display() {
        assert_eq!(Token::Input("foo".to_string()).to_string(), "foo");
        assert_eq!(Token::Pipe.to_string(), "|");
        assert_eq!(Token::RedirectOut.to_string(), ">");
        assert_eq!(Token::RedirectAppend.to_string(), ">>");
        assert_eq!(Token::RedirectIn.to_string(), "<");
        assert_eq!(Token::And.to_string(), "&&");
        assert_eq!(Token::Or.to_string(), "||");
        assert_eq!(Token::Background.to_string(), "&");
        assert_eq!(Token::Semicolon.to_string(), ";");
        assert_eq!(Token::OpenParenthesis.to_string(), "(");
        assert_eq!(Token::CloseParenthesis.to_string(), ")");
    }
}
