/// Text-based interactive shell for the mini SQL engine.
use crate::core::engine::DatabaseEngine;
use std::io::{self, Write};

const PROMPT: &str = "db> ";
const EXIT_COMMANDS: &[&str] = &["quit", "exit", ":q"];

pub fn run_shell() {
    let mut engine = DatabaseEngine::new();
    println!("Welcome to the mini SQL shell. Type 'exit' to quit.");

    loop {
        print!("{}", PROMPT);
        io::stdout().flush().unwrap();

        let mut query = String::new();
        match io::stdin().read_line(&mut query) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let trimmed = query.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if EXIT_COMMANDS.contains(&trimmed.to_lowercase().as_str()) {
                    break;
                }

                let results = engine.execute(trimmed);
                for line in results {
                    println!("{}", line);
                }
            }
            Err(error) => {
                eprintln!("Error reading input: {}", error);
                break;
            }
        }
    }
}
