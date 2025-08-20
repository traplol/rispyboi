use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process;

mod builtins;
mod error;
mod eval;
mod lexer;
mod macros;
mod parser;
mod value;

use error::RispyError;
use value::Environment;
use std::rc::Rc;

fn load_boot_library(
    env: &Rc<Environment>,
    macro_expander: &macros::MacroExpander,
) -> Result<(), RispyError> {
    // Try to find boot.lisp in several locations
    let boot_paths = vec![
        "lisp/boot.lisp",                              // Relative to current directory
        "./lisp/boot.lisp",                            // Explicit relative
        concat!(env!("CARGO_MANIFEST_DIR"), "/lisp/boot.lisp"), // Relative to project root
    ];

    let mut boot_content = None;
    let mut boot_path = None;

    // Try each path until we find the boot file
    for path in &boot_paths {
        if Path::new(path).exists() {
            match fs::read_to_string(path) {
                Ok(content) => {
                    boot_content = Some(content);
                    boot_path = Some(path);
                    break;
                }
                Err(_) => continue,
            }
        }
    }

    match boot_content {
        Some(content) => {
            // Parse and execute the boot library
            let tokens = lexer::tokenize(&content)?;
            let ast = parser::parse(tokens)?;

            for expr in ast {
                // Execute each top-level expression in the boot library
                // Suppress output for most expressions except the final confirmation messages
                let _result = eval::eval_with_macros(&expr, env, macro_expander)?;
            }

            Ok(())
        }
        None => {
            // Boot library not found - print warning but continue
            eprintln!("Warning: boot.lisp not found. Standard library functions unavailable.");
            eprintln!("Looked in: {:?}", boot_paths);
            Ok(())
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        eprintln!("Usage: {} [file]", args[0]);
        process::exit(1);
    }

    if args.len() == 2 {
        run_file(&args[1]);
    } else {
        run_repl();
    }
}

fn run_file(path: &str) {
    match fs::read_to_string(path) {
        Ok(contents) => {
            if let Err(e) = execute_program(&contents) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to read file '{}': {}", path, e);
            process::exit(1);
        }
    }
}

fn run_repl() {
    println!("RispyBoi Lisp Interpreter");
    println!("Type expressions to evaluate, or 'exit' to quit.");

    // Create persistent environment and macro expander for REPL session
    let (env, macro_expander) = eval::create_global_environment_with_macros();
    
    // Load boot library
    if let Err(e) = load_boot_library(&env, &macro_expander) {
        eprintln!("Warning: Failed to load boot library: {}", e);
    }

    loop {
        print!("rispy> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached
                println!("Goodbye!");
                break;
            }
            Ok(_) => {
                let input = input.trim();
                if input == "exit" || input == "quit" {
                    println!("Goodbye!");
                    break;
                }

                if !input.is_empty() {
                    if let Err(e) = execute_repl_input(input, &env, &macro_expander) {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                break;
            }
        }
    }
}

fn execute_repl_input(
    source: &str,
    env: &Rc<Environment>,
    macro_expander: &macros::MacroExpander,
) -> Result<(), RispyError> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;

    for expr in ast {
        let result = eval::eval_with_macros(&expr, env, macro_expander)?;
        println!("{}", result);
    }

    Ok(())
}

fn execute_program(source: &str) -> Result<(), RispyError> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;
    let (env, macro_expander) = eval::create_global_environment_with_macros();
    
    // Load boot library
    if let Err(e) = load_boot_library(&env, &macro_expander) {
        eprintln!("Warning: Failed to load boot library: {}", e);
    }

    for expr in ast {
        let result = eval::eval_with_macros(&expr, &env, &macro_expander)?;
        println!("{}", result);
    }

    Ok(())
}
