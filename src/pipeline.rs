use std::io;

use crate::{
    exec::{execute, RunningProcess},
    lex::Lexer,
    parse::Parser,
};

pub struct Pipeline;

impl Pipeline {
    pub fn run(input: &str) -> io::Result<RunningProcess> {
        let tokens = Lexer::lex(input)?;
        let ast = Parser::parse(&tokens)?;
        execute(&ast)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use io::{Read, Write};
    use tempdir::TempDir;

    use super::*;

    #[test]
    fn test_pipeline_pipe() {
        let input = "echo 'foo\nbar\nbaz' | grep bar";
        let status = Pipeline::run(input).unwrap();
        assert!(status.success());

        let input = "echo 'foo\nbar\nbaz' | grep qux";
        let status = Pipeline::run(input).unwrap();
        assert!(!status.success());
    }

    #[test]
    fn test_pipeline_redirect_append() {
        let dir = TempDir::new("").unwrap();
        let path = dir.path().join("output.txt");
        File::create(&path)
            .unwrap()
            .write_all("foo\n".as_bytes())
            .unwrap();
        let input = format!("echo bar >> {}", path.to_str().unwrap());
        let status = Pipeline::run(&input).unwrap();
        assert!(status.success());
        let mut result = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();
        assert_eq!(&result, "foo\nbar\n");
    }

    #[test]
    fn test_pipeline_redirect_out() {
        let dir = TempDir::new("").unwrap();
        let path = dir.path().join("output.txt");
        let input = format!(
            "echo foo | cat | cat|cat  |  cat > {}",
            path.to_str().unwrap()
        );
        let status = Pipeline::run(&input).unwrap();
        assert!(status.success());
        let mut result = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();
        assert_eq!(&result, "foo\n");
    }

    #[test]
    fn test_nested_subshells() {
        let dir = TempDir::new("").unwrap();
        let path = dir.path().join("output.txt");
        let input = format!(
            "((( echo foo | cat ) | cat ) | cat ) | cat > {}",
            path.to_str().unwrap()
        );
        let status = Pipeline::run(&input).unwrap();
        assert!(status.success());
        let mut result = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();
        assert_eq!(&result, "foo\n");
    }
}
