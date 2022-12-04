#[macro_use]
extern crate lazy_static;

use std::{env, error::Error, process::exit};

use interpreter::Interpreter;

mod environment;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
mod token;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut interpreter = Interpreter::new();
    match args.len() {
        2.. => {
            eprintln!("Usage: rlox [script]");
            exit(exitcode::USAGE);
        }
        1 => interpreter.run_file(&args[0]),
        _ => interpreter.run_prompt(),
    }
}
