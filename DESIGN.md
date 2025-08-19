# Lisp Interpreter in Rust - Design Document

## Overview
A minimal Lisp interpreter implementation in Rust, focusing on core language features with minimal external dependencies.

## Core Components

### 1. Lexer/Tokenizer (`src/lexer.rs`)
**Purpose**: Convert raw text input into tokens
**Responsibilities**:
- Tokenize parentheses `(`, `)`
- Identify symbols, numbers, strings
- Handle whitespace and comments
- Error reporting for invalid characters

**Token Types**:
```rust
enum Token {
    LeftParen,
    RightParen,
    Symbol(String),
    Number(f64),
    String(String),
    Quote,
}
```

### 2. Parser (`src/parser.rs`)
**Purpose**: Convert tokens into Abstract Syntax Tree (AST)
**Responsibilities**:
- Parse S-expressions
- Handle nested structures
- Validate parentheses matching
- Convert tokens to AST nodes

### 3. AST/Value Types (`src/value.rs`)
**Purpose**: Represent Lisp values and expressions
**Core Types**:
```rust
enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Value>),
    Function(Function),
}
```

### 4. Environment (`src/env.rs`)
**Purpose**: Variable and function binding management
**Responsibilities**:
- Symbol table management
- Scope handling (lexical scoping)
- Built-in function definitions
- Variable lookup and binding

### 5. Evaluator (`src/eval.rs`)
**Purpose**: Execute parsed Lisp expressions
**Responsibilities**:
- Evaluate atoms (numbers, symbols, strings)
- Function application
- Special form handling (if, define, lambda, quote)
- Recursive evaluation of lists

### 6. Built-in Functions (`src/builtins.rs`)
**Purpose**: Minimal primitive functions (most features implemented as Lisp macros)
**Essential Primitives Only**:
- Arithmetic: `+`, `-`, `*`, `/` (basic binary operations)
- Comparison: `=`, `<` (other comparisons built as macros)
- List operations: `car`, `cdr`, `cons`
- Type predicates: `atom?`, `null?`
- I/O: `print`

### 7. Special Forms (`src/special_forms.rs`)
**Purpose**: Absolute minimum special syntax required
**Core Forms Only**:
- `quote` - prevent evaluation
- `if` - conditional execution (ternary only)
- `define` - variable/function definition
- `lambda` - function creation
- `macro` - macro definition (for implementing other features)

### 8. Macro System (`src/macros.rs`)
**Purpose**: Enable Lisp macros for implementing most language features
**Responsibilities**:
- Macro definition and expansion
- Macro environment management
- Recursive macro expansion
- Hygiene (basic)

### 9. REPL (`src/repl.rs`)
**Purpose**: Interactive Read-Eval-Print Loop
**Responsibilities**:
- Read user input
- Parse and evaluate expressions
- Print results
- Handle errors gracefully
- Load standard library macros on startup

### 10. Error Handling (`src/error.rs`)
**Purpose**: Comprehensive error management
**Error Types**:
- Parse errors (syntax)
- Runtime errors (undefined symbols, type mismatches)
- Arity errors (wrong number of arguments)
- Macro expansion errors
- User-friendly error messages

### 11. Main Entry Point (`src/main.rs`)
**Purpose**: Application entry and CLI handling
**Responsibilities**:
- Command-line argument parsing
- File execution vs REPL mode
- Error handling and reporting
- Load standard library on startup

## File Structure
```
src/
├── main.rs           # Entry point and CLI
├── lexer.rs          # Tokenization
├── parser.rs         # AST generation
├── value.rs          # Core data types
├── env.rs            # Environment/scope management
├── eval.rs           # Expression evaluation
├── builtins.rs       # Minimal primitive functions
├── special_forms.rs  # Minimal special syntax
├── macros.rs         # Macro system
├── repl.rs           # Interactive shell
└── error.rs          # Error types and handling
lisp/
├── boot.lisp         # The bare minimum lisp library to run a REPL
└── stdlib.lisp       # Standard library as Lisp macros
```

## Dependencies
**Standard Library Only**:
- `std::collections::HashMap` - for environment/symbol tables
- `std::io` - for REPL input/output
- `std::env` - for command-line arguments
- `std::fs` - for file reading
- `std::fmt` - for error display

**No External Crates Required** - keeping it minimal with just Rust standard library.

## Language Features to Support

### Rust Primitives (Minimal):
- Basic arithmetic (`+`, `-`, `*`, `/` - binary only)
- Basic comparison (`=`, `<`)
- List operations (`car`, `cdr`, `cons`)
- Type predicates (`atom?`, `null?`)
- Variable definition (`define`)
- Function definition (`lambda`)
- Conditional (`if` - ternary only)
- Quote mechanism (`quote`)
- Macro definition (`macro`)
- I/O (`print`)

### Lisp Macros (Most Features):
- Multi-argument arithmetic (`(+ 1 2 3)` expands to `(+ (+ 1 2) 3)`)
- Additional comparisons (`>`, `<=`, `>=`, `/=`)
- Logic operators (`and`, `or`, `not`)
- Local bindings (`let`, `let*`)
- Conditional variants (`when`, `unless`, `cond`)
- List utilities (`list`, `length`, `append`, `reverse`)
- Higher-order functions (`map`, `filter`, `reduce`)
- Control flow (`do`, `while`, `for`)
- Type predicates (`number?`, `symbol?`, `list?`)

### Implementation Phases:
1. ✅ **Core Infrastructure**: Entry point, lexer, error handling  
2. **Parser & AST**: Value types and S-expression parsing
3. **Core Primitives**: Implement minimal Rust primitives
4. **Macro System**: Add macro definition and expansion
5. **Standard Library**: Implement features as Lisp macros
6. **Optimization**: Tail call optimization, better error messages

## Progress Status

### ✅ Completed Components
- **main.rs**: CLI argument parsing, file execution, REPL mode
- **lexer.rs**: Complete tokenizer with support for all Lisp syntax elements
- **error.rs**: Comprehensive error type definitions
- **value.rs**: Complete value system with all Lisp types and utilities

### 🚧 Current Implementation Status
Core infrastructure is complete with these features:
- **Tokenization**: Full Lisp syntax support with comprehensive tests
- **Value System**: All data types (Nil, Bool, Number, String, Symbol, List, Function)
- **Type Safety**: Comprehensive type checking and validation
- **Environment**: Variable binding with lexical scoping
- **Error Handling**: Detailed error types and user-friendly messages
- **Test Coverage**: 17 passing tests across all components

### 📋 Next Steps
1. Implement `parser.rs` for token → AST conversion
2. Implement `builtins.rs` for essential primitive functions  
3. Implement `eval.rs` for expression evaluation
4. Complete working REPL with basic functionality

**Current Status**: ~30% complete, ready for parser implementation

## Example Usage
```lisp
;; Primitive arithmetic (binary only in Rust)
(+ 1 2)    ; => 3
(+ (+ 1 2) 3)  ; => 6

;; Multi-argument arithmetic (implemented as macro)
(+ 1 2 3)  ; expands to (+ (+ 1 2) 3) => 6

;; Variable definition
(define x 10)
(* x 2)    ; => 20

;; Function definition
(define square (lambda (x) (* x x)))
(square 5) ; => 25

;; Primitive conditional (ternary only)
(if (< 3 5) "yes" "no") ; => "yes"

;; Macro-based conditional
(when (< 3 5) (print "yes")) ; expands to (if (< 3 5) (print "yes") nil)

;; Lists
(define lst '(1 2 3))
(car lst)  ; => 1
(cdr lst)  ; => (2 3)

;; Macro example
(define-macro when (test . body)
  `(if ,test (do ,@body) nil))
```

## Code Style Guidelines

### Formatting
- Use `cargo fmt` on all files before committing
- Follow standard Rust formatting conventions

### Comments
- Keep comments minimal - code should be self-documenting
- Only comment to explain **why** something is done, not **what** is being done
- Avoid obvious comments like `// increment counter`
- Focus on business logic reasoning, algorithmic choices, or non-obvious design decisions

### Naming
- Use descriptive names for functions and variables
- Prefer `parse_expression` over `parse_expr`
- Use standard Rust naming conventions (snake_case for functions/variables, PascalCase for types)

### Error Handling
- Use `Result<T, E>` for fallible operations
- Prefer `?` operator over explicit match when appropriate
- Create meaningful error types rather than using strings

### Code Organization
- Keep functions focused and small
- Group related functionality in the same module
- Use `pub` sparingly - only expose what needs to be public
- Prefer composition over inheritance-like patterns

## Testing Strategy
- Unit tests for each component
- Integration tests for full expressions
- Error case testing
- REPL interaction testing
