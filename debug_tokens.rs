use std::process;

mod error;
mod lexer;

use error::RispyError;

fn main() {
    let input = "(+ 1 2 3 4 5 6 7 8 9 10)";
    match lexer::tokenize(input) {
        Ok(tokens) => {
            println\!("Token count: {}", tokens.len());
            for (i, token) in tokens.iter().enumerate() {
                println\!("{}: {:?}", i, token);
            }
        }
        Err(e) => {
            eprintln\!("Error: {}", e);
            process::exit(1);
        }
    }
}
EOF < /dev/null