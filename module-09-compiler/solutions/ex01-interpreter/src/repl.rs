use crate::eval::Evaluator;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::value::Value;
use std::io::{self, Write};

pub fn run_repl() {
    let mut evaluator = Evaluator::new();

    println!("Welcome to the Monkey REPL!");
    println!("Type 'exit' to quit");
    println!();

    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        if input == "exit" || input == "quit" {
            break;
        }

        if input.is_empty() {
            continue;
        }

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        match parser.parse_program() {
            Ok(program) => match evaluator.eval_program(program) {
                Ok(value) => {
                    if !matches!(value, Value::Null) {
                        println!("{}", value);
                    }
                }
                Err(e) => println!("Error: {:?}", e),
            },
            Err(e) => println!("Parse error: {:?}", e),
        }
    }

    println!("Goodbye!");
}
