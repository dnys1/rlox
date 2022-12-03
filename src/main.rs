#[macro_use]
extern crate lazy_static;

use std::{
    env,
    error::Error,
    io::{stdin, stdout, Write},
    process::exit, fs,
};

use parser::Parser;
use scanner::Scanner;

use crate::interpreter::evaluate;

mod expr;
mod interpreter;
mod parser;
mod scanner;
mod token;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.len() {
        2.. => {
            eprintln!("Usage: rlox [script]");
            exit(exitcode::USAGE);
        }
        1 => run_file(&args[0]),
        _ => run_prompt(),
    }
}

fn run_file(filename: &str) -> Result<()> {
    let source = fs::read_to_string(filename)?;
    run(&source)
}

fn run_prompt() -> Result<()> {
    loop {
        let mut input = String::new();
        print!("> ");
        stdout().flush()?;
        stdin().read_line(&mut input)?;
        if input.trim().is_empty() {
            break;
        }
        run(&input)?;
    }
    Ok(())
}

fn run(source: &str) -> Result<()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;
    match evaluate(&expr) {
        Ok(res) => println!("{}", res),
        Err(e) => eprintln!("{}", e),
    };
    Ok(())
}
