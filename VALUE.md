# Value Representation in RispyBoi

## Overview
This document details how Lisp values are represented internally in the RispyBoi interpreter, including memory layout, type system design, and implementation considerations.

## Core Value Types

### Primary Value Enum
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Value>),
    Function(Function),
}
```

### Value Type Descriptions

#### `Nil`
- Represents the empty/null value in Lisp
- Used for: empty lists `()`, uninitialized variables, function returns with no value
- Memory: Zero-cost enum variant
- Display: `nil`

#### `Bool(bool)`
- Represents truth values
- Values: `true` (`#t`) and `false` (`#f`)
- Memory: Single byte
- Display: `#t` or `#f`

#### `Number(f64)`
- All numeric values represented as 64-bit floating point
- Simplifies arithmetic operations (no integer/float distinction)
- Supports: integers, decimals, negative numbers, scientific notation
- Memory: 8 bytes
- Display: Standard number formatting

#### `String(String)`
- UTF-8 encoded text values
- Supports escape sequences: `\n`, `\t`, `\r`, `\\`, `\"`
- Memory: Heap-allocated, growable
- Display: Quoted strings `"hello"`

#### `Symbol(String)`
- Identifiers and keywords in Lisp code
- Used for: variable names, function names, operators
- Memory: Heap-allocated string
- Display: Unquoted symbol name
- Examples: `+`, `define`, `my-variable`

#### `List(Vec<Value>)`
- Recursive data structure for S-expressions
- Used for: function calls, data structures, quoted lists
- Memory: Heap-allocated vector of values
- Display: Parenthesized list `(1 2 3)`

#### `Function(Function)`
- Callable function values
- Contains both built-in and user-defined functions
- Memory: Function struct (see Function Types below)

## Function Type Hierarchy

### Function Enum
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Builtin(BuiltinFunction),
    UserDefined(UserFunction),
    Macro(MacroFunction),
}
```

### BuiltinFunction
```rust
#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub arity: Arity,
    pub func: fn(&[Value]) -> Result<Value, RispyError>,
}
```
- Represents primitive functions implemented in Rust
- `arity`: Expected number of arguments (fixed, variable, or range)
- `func`: Function pointer to Rust implementation
- Examples: `+`, `-`, `car`, `cdr`, `cons`

### UserFunction
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UserFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Value,
    pub closure: Environment,
}
```
- Functions defined with `lambda` or `define`
- `params`: Parameter names
- `body`: Function body expression
- `closure`: Captured lexical environment
- Supports closures and lexical scoping

### MacroFunction
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct MacroFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Value,
    pub env: Environment,
}
```
- Macros defined with `macro` special form
- Similar to user functions but expand at compile-time
- Transform code before evaluation

## Arity System

### Arity Enum
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Arity {
    Exact(usize),           // Exactly N arguments
    AtLeast(usize),         // N or more arguments  
    Range(usize, usize),    // Between min and max arguments
    Variadic,               // Any number of arguments
}
```

### Arity Examples
- `+`: `AtLeast(2)` - requires at least 2 arguments
- `car`: `Exact(1)` - requires exactly 1 argument
- `if`: `Exact(3)` - requires exactly 3 arguments (condition, then, else)
- `print`: `Variadic` - accepts any number of arguments

## Memory Management Strategy

### Value Cloning
- `Value` implements `Clone` for easy copying
- Lists and strings are heap-allocated but cloned when needed
- Functions contain environments which may be large
- Trade-off: Simplicity over memory efficiency

### Garbage Collection
- **Phase 1**: Manual memory management via Rust's ownership
- **Phase 2**: Potential future addition of mark-and-sweep GC for cycles
- Current approach sufficient for most Lisp programs

### String Interning (Future)
- Symbols could benefit from string interning
- Reduce memory usage for repeated symbol names
- Improve equality comparison performance

## Type Checking and Predicates

### Built-in Type Predicates
```rust
pub fn is_nil(value: &Value) -> bool
pub fn is_bool(value: &Value) -> bool  
pub fn is_number(value: &Value) -> bool
pub fn is_string(value: &Value) -> bool
pub fn is_symbol(value: &Value) -> bool
pub fn is_list(value: &Value) -> bool
pub fn is_function(value: &Value) -> bool
pub fn is_atom(value: &Value) -> bool     // Not a list
pub fn is_truthy(value: &Value) -> bool   // For conditionals
```

### Truthiness Rules
- `Nil` and `Bool(false)` are falsy
- All other values are truthy
- Follows common Lisp conventions

## Value Construction Helpers

### Convenience Constructors
```rust
impl Value {
    pub fn nil() -> Value
    pub fn bool(b: bool) -> Value
    pub fn number(n: f64) -> Value
    pub fn string(s: impl Into<String>) -> Value
    pub fn symbol(s: impl Into<String>) -> Value
    pub fn list(values: Vec<Value>) -> Value
}
```

### List Utilities
```rust
impl Value {
    pub fn car(&self) -> Result<&Value, RispyError>
    pub fn cdr(&self) -> Result<Value, RispyError>
    pub fn cons(self, tail: Value) -> Value
    pub fn length(&self) -> Result<usize, RispyError>
    pub fn is_empty_list(&self) -> bool
}
```

## Display Implementation

### Pretty Printing
```rust
impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::List(list) => {
                write!(f, "(")?;
                for (i, value) in list.iter().enumerate() {
                    if i > 0 { write!(f, " ")?; }
                    write!(f, "{}", value)?;
                }
                write!(f, ")")
            },
            Value::Function(func) => write!(f, "#<function:{}>", func.name()),
        }
    }
}
```

### String Escaping
- Escape special characters in string display
- Handle: `"`, `\`, `\n`, `\t`, `\r`
- Ensure round-trip parsing works correctly

## Performance Considerations

### Memory Layout
- `Value` enum size dominated by `Vec<Value>` in `List` variant
- Consider `Box<[Value]>` for immutable lists in future
- Function variants contain environment data

### Clone Performance
- Frequent cloning of values during evaluation
- Strings and symbols cloned on each access
- Consider `Rc<String>` for symbols in future optimization

### Equality Comparison
- Structural equality for lists (recursive)
- String comparison for symbols/strings
- Numeric comparison with floating-point considerations

## Error Handling Integration

### Type Errors
```rust
pub fn expect_number(&self) -> Result<f64, RispyError>
pub fn expect_string(&self) -> Result<&str, RispyError>
pub fn expect_symbol(&self) -> Result<&str, RispyError>
pub fn expect_list(&self) -> Result<&[Value], RispyError>
pub fn expect_function(&self) -> Result<&Function, RispyError>
```

### Arity Checking
```rust
pub fn check_arity(args: &[Value], expected: &Arity) -> Result<(), RispyError>
```

## Future Enhancements

### Potential Optimizations
1. **Symbol Interning**: Reduce memory usage for repeated symbols
2. **Copy-on-Write**: For large lists that are frequently copied
3. **Specialized Number Types**: Separate integer/float for better performance
4. **Reference Counting**: For sharing large data structures

### Additional Value Types
1. **Vectors**: Indexed arrays `#(1 2 3)`
2. **Hash Maps**: Key-value pairs `#{:key value}`  
3. **Characters**: Single character values `#\a`
4. **Keywords**: Self-evaluating symbols `:keyword`

## Testing Strategy

### Unit Tests
- Value construction and accessors
- Type predicate functions
- List manipulation utilities
- Display formatting
- Equality comparisons

### Property Tests (Future)
- Round-trip parsing/display
- Clone equivalence
- Type predicate consistency