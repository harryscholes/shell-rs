use std::io::{self, BufRead, Write};

use shell::pipeline::Pipeline;

fn main() {
    let mut line = String::new();

    loop {
        print!("\n> ");
        let _ = io::stdout().flush();

        io::stdin().lock().read_line(&mut line).unwrap();

        if let Err(e) = Pipeline::run(line.trim()) {
            eprintln!("Error: {}", e);
        }

        line.clear();
    }
}
