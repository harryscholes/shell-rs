use std::{
    fs::OpenOptions,
    io,
    process::{Child, Command, ExitStatus, Stdio},
};

use crate::{ast::Ast, grammar::Token};

pub fn execute(ast: &Ast) -> io::Result<ExitStatus> {
    exec_impl(ast, None, None)?.wait()
}

fn exec_impl(ast: &Ast, stdin: Option<Stdio>, stdout: Option<Stdio>) -> io::Result<Child> {
    match ast {
        Ast::Command { command, args } => exec_command(command, args, stdin, stdout),
        Ast::Pipe { left, right } => exec_pipe(left, right, stdout),
        Ast::RedirectOut { left, right } => exec_redirect_out(left, right),
        Ast::RedirectAppend { left, right } => exec_redirect_append(left, right),
        Ast::And { left, right } => exec_and(left, right, stdout),
        Ast::Or { left, right } => exec_or(left, right, stdout),
        Ast::Sequence { left, right } => exec_sequence(left, right, stdout),
    }
}

fn exec_command(
    command: &Token,
    args: &[Token],
    stdin: Option<Stdio>,
    stdout: Option<Stdio>,
) -> io::Result<Child> {
    let mut cmd = Command::new(command.as_ref());

    for arg in args {
        cmd.arg(arg.as_ref());
    }

    if let Some(stdin) = stdin {
        cmd.stdin(stdin);
    }

    if let Some(stdout) = stdout {
        cmd.stdout(stdout);
    }

    cmd.spawn()
}

fn exec_pipe(left: &Ast, right: &Ast, stdout: Option<Stdio>) -> io::Result<Child> {
    let child = exec_impl(left, None, Some(Stdio::piped()))?;
    exec_impl(right, Some(Stdio::from(child.stdout.unwrap())), stdout)
}

fn exec_redirect_out(left: &Ast, right: &Token) -> io::Result<Child> {
    let file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(right.as_ref())?;
    exec_impl(left, None, Some(Stdio::from(file)))
}

fn exec_redirect_append(left: &Ast, right: &Token) -> io::Result<Child> {
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(right.as_ref())?;
    exec_impl(left, None, Some(Stdio::from(file)))
}

fn exec_and(left: &Ast, right: &Ast, stdout: Option<Stdio>) -> io::Result<Child> {
    let mut child = exec_impl(left, None, None)?;
    if child.wait()?.success() {
        exec_impl(right, None, stdout)
    } else {
        Ok(child)
    }
}

fn exec_or(left: &Ast, right: &Ast, stdout: Option<Stdio>) -> io::Result<Child> {
    let mut child = exec_impl(left, None, None)?;
    if !child.wait()?.success() {
        exec_impl(right, None, stdout)
    } else {
        Ok(child)
    }
}

fn exec_sequence(left: &Ast, right: &Ast, stdout: Option<Stdio>) -> io::Result<Child> {
    exec_impl(left, None, None)?;
    exec_impl(right, None, stdout)
}

#[cfg(test)]
mod tests {
    use io::Write;
    use std::{fs::File, io::Read};
    use tempdir::TempDir;

    use crate::input;

    use super::*;

    #[test]
    fn test_exec_command() {
        let command = input!("echo");
        let args = vec![input!("foo")];
        let stdout = Stdio::piped();
        let child = exec_command(&command, &args, None, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "foo\n");
    }

    #[test]
    fn test_exec_redirect_out() {
        let left = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo")],
        };
        let dir = TempDir::new("").unwrap();
        let path = dir.path().join("output.txt");
        let right = input!(path.to_str().unwrap());
        exec_redirect_out(&left, &right).unwrap().wait().unwrap();
        let mut result = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();
        assert_eq!(&result, "foo\n");
    }

    #[test]
    fn test_exec_redirect_append() {
        let dir = TempDir::new("").unwrap();
        let path = dir.path().join("output.txt");
        File::create(&path)
            .unwrap()
            .write_all("foo\n".as_bytes())
            .unwrap();

        let left = Ast::Command {
            command: input!("echo"),
            args: vec![input!("bar")],
        };
        let right = input!(path.to_str().unwrap());
        exec_redirect_append(&left, &right).unwrap().wait().unwrap();
        let mut result = String::new();
        File::open(&path)
            .unwrap()
            .read_to_string(&mut result)
            .unwrap();
        assert_eq!(&result, "foo\nbar\n");
    }

    #[test]
    fn test_exec_pipe() {
        let left = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo\nbar")],
        };
        let right = Ast::Command {
            command: input!("grep"),
            args: vec![input!("foo")],
        };
        let stdout = Stdio::piped();
        let child = exec_pipe(&left, &right, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "foo\n");

        let left = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo   bar")],
        };
        let right = Ast::Command {
            command: input!("tr"),
            args: vec![input!("-s"), input!(" ")],
        };
        let stdout = Stdio::piped();
        let out = exec_pipe(&left, &right, Some(stdout)).unwrap();
        let output = out.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "foo bar\n");
    }

    #[test]
    fn test_exec_and() {
        let left = Ast::Command {
            command: input!("true"),
            args: vec![],
        };
        let right = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo")],
        };
        let stdout = Stdio::piped();
        let child = exec_and(&left, &right, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "foo\n");

        let left = Ast::Command {
            command: input!("false"),
            args: vec![],
        };
        let right = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo")],
        };
        let stdout = Stdio::piped();
        let child = exec_and(&left, &right, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "");
    }

    #[test]
    fn test_exec_or() {
        let left = Ast::Command {
            command: input!("false"),
            args: vec![],
        };
        let right = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo")],
        };
        let stdout = Stdio::piped();
        let child = exec_or(&left, &right, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "foo\n");

        let left = Ast::Command {
            command: input!("true"),
            args: vec![],
        };
        let right = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo")],
        };
        let stdout = Stdio::piped();
        let child = exec_or(&left, &right, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "");
    }

    #[test]
    fn test_exec_sequence() {
        let left = Ast::Command {
            command: input!("echo"),
            args: vec![input!("foo")],
        };
        let right = Ast::Command {
            command: input!("echo"),
            args: vec![input!("bar")],
        };
        let stdout = Stdio::piped();
        let child = exec_sequence(&left, &right, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "bar\n");
    }

    #[test]
    fn test_exec_impl_with_pipe() {
        let ast = Ast::Pipe {
            left: Box::new(Ast::Command {
                command: input!("echo"),
                args: vec![input!("foo\nbar")],
            }),
            right: Box::new(Ast::Command {
                command: input!("grep"),
                args: vec![input!("foo")],
            }),
        };
        let stdout = Stdio::piped();
        let child = exec_impl(&ast, None, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(std::str::from_utf8(&output.stdout).unwrap(), "foo\n");
    }

    #[test]
    fn test_exec_impl_with_chained_pipes() {
        let ast = Ast::Pipe {
            left: Box::new(Ast::Pipe {
                left: Box::new(Ast::Command {
                    command: input!("echo"),
                    args: vec![input!("foo")],
                }),
                right: Box::new(Ast::Command {
                    command: input!("wc"),
                    args: vec![],
                }),
            }),
            right: Box::new(Ast::Command {
                command: input!("wc"),
                args: vec![],
            }),
        };
        let stdout = Stdio::piped();
        let child = exec_impl(&ast, None, Some(stdout)).unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(
            std::str::from_utf8(&output.stdout).unwrap(),
            "       1       3      25\n"
        );
    }
}
