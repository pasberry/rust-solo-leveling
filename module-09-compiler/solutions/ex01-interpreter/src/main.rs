use interpreter::*;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        // No arguments - run REPL
        run_repl();
    } else if args.len() == 2 {
        // File argument - execute file
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(source) => {
                let lexer = Lexer::new(&source);
                let mut parser = Parser::new(lexer);

                match parser.parse_program() {
                    Ok(program) => {
                        let mut evaluator = Evaluator::new();
                        match evaluator.eval_program(program) {
                            Ok(value) => {
                                if !matches!(value, Value::Null) {
                                    println!("{}", value);
                                }
                            }
                            Err(e) => {
                                eprintln!("Runtime error: {:?}", e);
                                process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Parse error: {:?}", e);
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading file '{}': {}", filename, e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Usage: {} [script.monkey]", args[0]);
        process::exit(1);
    }
}
