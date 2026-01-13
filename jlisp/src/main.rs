//! JLisp REPL - A Lisp where all data is JSON and all functions are JMESPath

use jlisp::Jlisp;
use std::io::{self, BufRead, Write};

fn main() {
    let mut jlisp = Jlisp::new();
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("JLisp - A Lisp where all data is JSON and all functions are JMESPath");
    println!("Type expressions as JSON arrays, e.g.: [\"add\", 1, 2]");
    println!("Type 'quit' or Ctrl+D to exit\n");

    loop {
        print!("jlisp> ");
        stdout.flush().unwrap();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "quit" || line == "exit" {
                    break;
                }

                match serde_json::from_str::<serde_json::Value>(line) {
                    Ok(expr) => match jlisp.eval(&expr) {
                        Ok(result) => {
                            println!("{}", serde_json::to_string_pretty(&result).unwrap());
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    },
                    Err(e) => {
                        eprintln!("Parse error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("IO error: {}", e);
                break;
            }
        }
    }

    println!("\nGoodbye!");
}
