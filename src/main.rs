#[macro_use]
extern crate lazy_static;

use std::{
    env,
    error::Error,
    io::{stdin, stdout, Write},
    process::exit,
};

use parser::Parser;
use scanner::Scanner;

mod expr;
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
        1 => run_file(args[0].clone()),
        _ => run_prompt(),
    }
}

fn run_file(filename: String) -> Result<()> {
    let bytes = std::fs::read(filename)?;
    run(String::from_utf8(bytes)?)
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
        run(input.clone())?;
    }
    Ok(())
}

fn run(source: String) -> Result<()> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    for token in &tokens {
        println!("{:?}", token);
    }
    let mut parser = Parser::new(tokens);
    let expr = parser.parse()?;
    println!("{}", expr);
    Ok(())
}
