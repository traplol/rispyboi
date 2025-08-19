# Parser Architecture for RispyBoi

## Overview
The parser module (`src/parser.rs`) is responsible for converting a stream of tokens from the lexer into an Abstract Syntax Tree (AST) represented as `Value` structures. This document details the parsing strategy, error handling, and implementation approach for Lisp S-expressions.

## Design Philosophy

### Key Principles
1. **Simplicity**: Lisp's uniform syntax makes parsing straightforward
2. **Error Recovery**: Provide meaningful error messages with context
3. **Zero-Copy Where Possible**: Minimize string allocations during parsing
4. **Recursive Descent**: Natural fit for nested S-expressions
5. **Fail Fast**: Stop parsing immediately on syntax errors

### S-Expression Grammar
```
program     → expression*
expression  → atom | list | quoted
atom        → NUMBER | STRING | SYMBOL | NIL | BOOLEAN
list        → '(' expression* ')'
quoted      → "'" expression
```

## Core Parser Structure

### Main Parser Function
```rust
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Value>, RispyError>
```
- **Input**: Token stream from lexer
- **Output**: Vector of parsed expressions (multiple top-level forms)
- **Error**: `ParseError` with descriptive message and position

### Parser State
```rust
struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self
    fn parse_expressions(&mut self) -> Result<Vec<Value>, RispyError>
    fn parse_expression(&mut self) -> Result<Value, RispyError>
    fn parse_list(&mut self) -> Result<Value, RispyError>
    fn parse_atom(&mut self) -> Result<Value, RispyError>
    
    // Utility methods
    fn advance(&mut self) -> Option<&Token>
    fn peek(&self) -> Option<&Token>
    fn is_at_end(&self) -> bool
    fn previous(&self) -> &Token
    fn check(&self, token_type: &Token) -> bool
    fn consume(&mut self, expected: Token, message: &str) -> Result<(), RispyError>
}
```

## Parsing Algorithm

### Top-Level Parsing
```rust
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Value>, RispyError> {
    let mut parser = Parser::new(tokens);
    parser.parse_expressions()
}

impl Parser {
    fn parse_expressions(&mut self) -> Result<Vec<Value>, RispyError> {
        let mut expressions = Vec::new();
        
        while !self.is_at_end() {
            expressions.push(self.parse_expression()?);
        }
        
        Ok(expressions)
    }
}
```

### Expression Parsing
```rust
fn parse_expression(&mut self) -> Result<Value, RispyError> {
    match self.peek() {
        Some(Token::LeftParen) => self.parse_list(),
        Some(Token::Quote) => self.parse_quoted(),
        Some(_) => self.parse_atom(),
        None => Err(RispyError::ParseError("Unexpected end of input".to_string())),
    }
}
```

### List Parsing
```rust
fn parse_list(&mut self) -> Result<Value, RispyError> {
    // Consume opening paren
    self.consume(Token::LeftParen, "Expected '('")?;
    
    let mut elements = Vec::new();
    
    // Parse elements until closing paren
    while !self.check(&Token::RightParen) && !self.is_at_end() {
        elements.push(self.parse_expression()?);
    }
    
    // Consume closing paren
    self.consume(Token::RightParen, "Expected ')' after list elements")?;
    
    Ok(Value::List(elements))
}
```

### Quote Parsing
```rust
fn parse_quoted(&mut self) -> Result<Value, RispyError> {
    // Consume quote token
    self.advance();
    
    // Parse the quoted expression
    let expr = self.parse_expression()?;
    
    // Transform 'expr into (quote expr)
    Ok(Value::List(vec![
        Value::Symbol("quote".to_string()),
        expr,
    ]))
}
```

### Atom Parsing
```rust
fn parse_atom(&mut self) -> Result<Value, RispyError> {
    match self.advance() {
        Some(Token::Number(n)) => Ok(Value::Number(*n)),
        Some(Token::String(s)) => Ok(Value::String(s.clone())),
        Some(Token::Symbol(s)) => {
            match s.as_str() {
                "nil" => Ok(Value::Nil),
                "#t" | "true" => Ok(Value::Bool(true)),
                "#f" | "false" => Ok(Value::Bool(false)),
                _ => Ok(Value::Symbol(s.clone())),
            }
        },
        Some(unexpected) => Err(RispyError::ParseError(
            format!("Unexpected token: {:?}", unexpected)
        )),
        None => Err(RispyError::ParseError("Unexpected end of input".to_string())),
    }
}
```

## Error Handling Strategy

### Error Types and Messages
```rust
// In error.rs - already defined
RispyError::ParseError(String)
```

### Common Parse Errors
1. **Unmatched Parentheses**
   ```
   ParseError: "Expected ')' after list elements"
   ParseError: "Unexpected ')' - no matching '('"
   ```

2. **Unexpected End of Input**
   ```
   ParseError: "Unexpected end of input while parsing list"
   ParseError: "Expected expression after quote"
   ```

3. **Invalid Token Sequences**
   ```
   ParseError: "Unexpected token: RightParen"
   ParseError: "Expected expression, found )"
   ```

### Error Recovery
- **Fail Fast**: Stop parsing on first error
- **Context Information**: Include position and context in error messages
- **Future Enhancement**: Position tracking for better error reporting

## Special Cases and Edge Handling

### Empty Lists
```lisp
()  ; → Value::List(vec![])
```

### Nested Structures
```lisp
(+ (* 2 3) (- 5 1))  ; → Deeply nested Value::List structures
```

### Quote Syntax
```lisp
'(1 2 3)     ; → (quote (1 2 3))
'x           ; → (quote x)
''x          ; → (quote (quote x))
```

### Boolean and Nil Literals
```lisp
#t           ; → Value::Bool(true)
#f           ; → Value::Bool(false)
nil          ; → Value::Nil
true         ; → Value::Bool(true) - alternative syntax
false        ; → Value::Bool(false) - alternative syntax
```

## Integration with Lexer

### Token Stream Processing
```rust
// main.rs integration
fn execute_program(source: &str) -> Result<(), RispyError> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;
    
    // Future: evaluate AST
    for expr in ast {
        println!("Parsed: {}", expr);
    }
    
    Ok(())
}
```

### Error Propagation
- Lexer errors propagate through parser unchanged
- Parser adds its own layer of syntax validation
- Both use the same `RispyError` enum for consistency

## Performance Considerations

### Memory Usage
- **Token Ownership**: Parser takes ownership of token vector
- **AST Construction**: Values are heap-allocated (strings, lists)
- **Cloning Strategy**: Minimal cloning during parsing

### Time Complexity
- **Linear Parsing**: O(n) where n is number of tokens
- **Recursive Descent**: Stack depth proportional to nesting level
- **No Backtracking**: Single-pass parsing for deterministic grammar

### Future Optimizations
1. **Arena Allocation**: For large ASTs, consider arena-allocated nodes
2. **String Interning**: For repeated symbols in large programs
3. **Position Tracking**: Add source position information for better errors

## Testing Strategy

### Unit Test Categories
1. **Basic Atoms**
   ```rust
   #[test]
   fn test_parse_atoms() {
       assert_eq!(parse_single("42"), Value::Number(42.0));
       assert_eq!(parse_single("\"hello\""), Value::String("hello".to_string()));
       assert_eq!(parse_single("foo"), Value::Symbol("foo".to_string()));
       assert_eq!(parse_single("nil"), Value::Nil);
       assert_eq!(parse_single("#t"), Value::Bool(true));
   }
   ```

2. **Simple Lists**
   ```rust
   #[test]
   fn test_parse_lists() {
       assert_eq!(parse_single("()"), Value::List(vec![]));
       assert_eq!(parse_single("(1 2 3)"), 
                  Value::List(vec![Value::Number(1.0), Value::Number(2.0), Value::Number(3.0)]));
   }
   ```

3. **Nested Structures**
   ```rust
   #[test]
   fn test_nested_lists() {
       let result = parse_single("(+ (* 2 3) 4)");
       // Verify deeply nested structure
   }
   ```

4. **Quote Syntax**
   ```rust
   #[test]
   fn test_quote_parsing() {
       assert_eq!(parse_single("'x"), 
                  Value::List(vec![Value::Symbol("quote".to_string()), Value::Symbol("x".to_string())]));
   }
   ```

5. **Error Cases**
   ```rust
   #[test]
   fn test_parse_errors() {
       assert!(parse_single("(").is_err());
       assert!(parse_single(")").is_err());
       assert!(parse_single("(1 2").is_err());
   }
   ```

6. **Multiple Expressions**
   ```rust
   #[test]
   fn test_multiple_expressions() {
       let result = parse("(+ 1 2) (* 3 4)").unwrap();
       assert_eq!(result.len(), 2);
   }
   ```

### Integration Tests
- **Lexer → Parser Chain**: Test complete tokenization and parsing
- **Error Propagation**: Verify error messages flow correctly
- **Real Lisp Programs**: Parse actual Lisp code snippets

## Future Enhancements

### Phase 1 Extensions
1. **Source Position Tracking**
   ```rust
   struct Token {
       token_type: TokenType,
       line: usize,
       column: usize,
   }
   ```

2. **Better Error Messages**
   ```
   ParseError: "Expected ')' at line 3, column 15"
   ```

### Phase 2 Extensions
1. **Reader Macros**: Support for `#()` vector syntax
2. **Comments in AST**: Preserve comments for pretty-printing
3. **Dotted Pairs**: Support for `(a . b)` syntax

### Advanced Features
1. **Error Recovery**: Continue parsing after errors
2. **Incremental Parsing**: For REPL efficiency
3. **Pretty Printing**: AST → formatted source code

## API Design

### Public Interface
```rust
// Primary parsing function
pub fn parse(tokens: Vec<Token>) -> Result<Vec<Value>, RispyError>

// Utility for single expressions (testing)
pub fn parse_single(source: &str) -> Result<Value, RispyError> {
    let tokens = crate::lexer::tokenize(source)?;
    let mut expressions = parse(tokens)?;
    
    if expressions.len() != 1 {
        return Err(RispyError::ParseError("Expected single expression".to_string()));
    }
    
    Ok(expressions.into_iter().next().unwrap())
}
```

### Error Handling Contract
- **Input Validation**: Assumes valid token stream from lexer
- **Fail Fast**: Returns immediately on syntax errors
- **Error Context**: Provides meaningful error descriptions
- **No Partial Results**: Either complete success or complete failure

## Integration Points

### With Lexer
- Consumes `Vec<Token>` from `lexer::tokenize()`
- Relies on lexer for valid token boundaries
- Inherits lexer's error handling for malformed input

### With Evaluator (Future)
- Produces `Vec<Value>` for evaluator consumption
- AST structure directly maps to evaluation strategy
- Symbol resolution handled by evaluator, not parser

### With REPL
- Supports parsing incomplete expressions (future)
- Multiple expression handling for REPL sessions
- Error recovery for interactive use
