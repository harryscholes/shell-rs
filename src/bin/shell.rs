use std::{
    f32::consts::E,
    io::{self, BufRead, Write},
};

use shell::pipeline::Pipeline;

fn main() {
    let mut line = String::new();

    loop {
        print!("\n> ");
        let _ = io::stdout().flush();

        io::stdin().lock().read_line(&mut line).unwrap();

        match Pipeline::run(line.trim()) {
            Ok(status) => {
                if !status.success() {
                    eprintln!("Exit status code: {:?}", status.code());
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }

        line.clear();
    }
}
