use std::fmt;

#[derive(Debug, Clone)]
pub enum RispyError {
    LexError(String),
    ParseError(String),
    RuntimeError(String),
    ArityError(String),
    TypeError(String),
    UndefinedSymbol(String),
}

impl fmt::Display for RispyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RispyError::LexError(msg) => write!(f, "Lexer error: {}", msg),
            RispyError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            RispyError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            RispyError::ArityError(msg) => write!(f, "Arity error: {}", msg),
            RispyError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RispyError::UndefinedSymbol(msg) => write!(f, "Undefined symbol: {}", msg),
        }
    }
}

impl std::error::Error for RispyError {}
