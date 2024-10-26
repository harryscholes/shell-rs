use std::io::{self, BufRead, Write};

use shell::{exec::RunningProcess, pipeline::Pipeline};

fn main() {
    let mut line = String::new();

    loop {
        print!("\n> ");
        let _ = io::stdout().flush();

        io::stdin().lock().read_line(&mut line).unwrap();

        match Pipeline::run(line.trim()) {
            Ok(p) => match p {
                RunningProcess::Foreground(status) => {
                    if !status.success() {
                        if let Some(code) = status.code() {
                            eprintln!("Error: {}", code);
                        }
                    }
                }
                RunningProcess::Background => {
                    println!("Background process started");
                }
            },
            Err(e) => eprintln!("Error: {}", e),
        }

        line.clear();
    }
}
