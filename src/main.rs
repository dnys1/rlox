#[macro_use]
extern crate lazy_static;

use std::{
    env,
    error::Error,
    io::{stdin, stdout, Write},
    process::exit,
};

use scanner::Scanner;

mod expr;
mod scanner;
mod token;

fn main() -> Result<(), Box<dyn Error>> {
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

fn run_file(filename: String) -> Result<(), Box<dyn Error>> {
    let bytes = std::fs::read(filename)?;
    run(String::from_utf8(bytes)?)
}

fn run_prompt() -> Result<(), Box<dyn Error>> {
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

fn run(source: String) -> Result<(), Box<dyn Error>> {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens()?;
    for token in tokens {
        println!("{:?}", token);
    }
    Ok(())
}
