# RispyBoi Macro System Architecture

## Overview
This document defines the architecture for RispyBoi's macro system, which enables compile-time code transformation and metaprogramming capabilities. Macros allow users to extend the language syntax and create domain-specific constructs while maintaining the core simplicity of the interpreter.

## Design Philosophy

### Core Principles
1. **Hygiene**: Macros should not accidentally capture variables from their expansion context
2. **Compile-Time Evaluation**: Macros are expanded before runtime evaluation
3. **Code as Data**: Leverage Lisp's homoiconicity for powerful metaprogramming
4. **Separation of Concerns**: Clear distinction between macro-time and runtime environments
5. **Mutual Recursion Support**: Macros can safely reference each other

### Macro vs Function Distinction
```lisp
;; Function: Evaluated at runtime, arguments are evaluated first
(define-function double (x) (* 2 x))
(double (+ 1 2))  ; → (* 2 3) → 6

;; Macro: Expanded at compile-time, arguments are NOT evaluated
(define-macro when (test body) `(if ,test ,body nil))
(when (< 3 5) (print "yes"))  ; → (if (< 3 5) (print "yes") nil)
```

## Macro System Architecture

### Two-Phase Evaluation Model

#### Phase 1: Macro Expansion (Compile-Time)
1. **Macro Detection**: Identify macro calls in AST
2. **Macro Application**: Apply macro transformations to generate new AST
3. **Recursive Expansion**: Expand nested macros until fixed point
4. **Hygiene Enforcement**: Ensure variable capture safety

#### Phase 2: Runtime Evaluation
1. **Standard Evaluation**: Evaluate the expanded AST using normal eval rules
2. **No Macro Awareness**: Runtime evaluator sees only expanded code

### Macro Environments

#### Macro Environment (Compile-Time)
```rust
pub struct MacroEnvironment {
    // Macro definitions available during expansion
    macros: HashMap<String, MacroFunction>,
    // Parent macro environment for nested scopes
    parent: Option<Rc<MacroEnvironment>>,
}
```

#### Runtime Environment (Runtime)
```rust
pub struct RuntimeEnvironment {
    // Variables and functions available during execution
    bindings: RefCell<HashMap<String, Value>>,
    // Parent runtime environment for lexical scoping
    parent: Option<Rc<RuntimeEnvironment>>,
}
```

#### Separation of Environments
- **Macro definitions** exist only in MacroEnvironment
- **Runtime values** exist only in RuntimeEnvironment
- **No cross-contamination** between compile-time and runtime

## Macro Definition and Expansion

### Macro Definition Syntax
```lisp
(define-macro name (param1 param2 ...) template-body)
```

**Examples:**
```lisp
;; Simple substitution macro
(define-macro when (test body)
  `(if ,test ,body nil))

;; Multi-statement macro  
(define-macro begin (expr1 expr2)
  `((lambda () ,expr1 ,expr2)))

;; Conditional compilation
(define-macro debug (expr)
  `(if *debug-mode* (print ,expr) nil))
```

### Quasiquote and Unquote System

#### Template Language
- **Quasiquote** `` ` ``: Creates template with selective evaluation
- **Unquote** `,`: Evaluates expression within quasiquote
- **Unquote-splicing** `,@`: Splices list elements into template

```lisp
;; Template construction
`(if ,test ,then-clause ,else-clause)
;; With test=x, then-clause=y, else-clause=z
;; Expands to: (if x y z)

;; List splicing
`(begin ,@statements)
;; With statements=(a b c)
;; Expands to: (begin a b c)
```

### Macro Expansion Algorithm

#### 1. Macro Detection
```rust
fn is_macro_call(expr: &Value, macro_env: &MacroEnvironment) -> bool {
    match expr {
        Value::List(list) if !list.is_empty() => {
            if let Value::Symbol(name) = &list[0] {
                macro_env.get_macro(name).is_some()
            } else { false }
        },
        _ => false
    }
}
```

#### 2. Macro Application
```rust
fn expand_macro(
    macro_call: &Value, 
    macro_env: &MacroEnvironment
) -> Result<Value, RispyError> {
    let (macro_name, args) = parse_macro_call(macro_call)?;
    let macro_def = macro_env.get_macro(&macro_name)?;
    
    // Bind macro parameters to arguments
    let bindings = bind_macro_params(&macro_def.params, args)?;
    
    // Evaluate macro body in binding environment
    let template = eval_macro_body(&macro_def.body, &bindings)?;
    
    // Apply template substitution
    expand_template(template, &bindings)
}
```

#### 3. Recursive Expansion
```rust
fn expand_all_macros(
    expr: &Value, 
    macro_env: &MacroEnvironment
) -> Result<Value, RispyError> {
    let mut current = expr.clone();
    let mut changed = true;
    
    while changed {
        let expanded = expand_macros_once(&current, macro_env)?;
        changed = expanded != current;
        current = expanded;
    }
    
    Ok(current)
}
```

## Mutual Recursion Support

### Challenge: Forward References
```lisp
;; This should work even though 'odd?' is not yet defined
(define-macro even? (n) `(if (= ,n 0) #t (odd? (- ,n 1))))
(define-macro odd? (n) `(if (= ,n 0) #f (even? (- ,n 1))))
```

### Solution: Two-Pass Definition

#### Pass 1: Declaration Phase
```rust
// Register macro names without bodies
fn declare_macro(name: &str, macro_env: &mut MacroEnvironment) {
    macro_env.declare(name.to_string(), MacroFunction::placeholder());
}
```

#### Pass 2: Definition Phase  
```rust
// Define macro bodies after all names are declared
fn define_macro_body(
    name: &str, 
    params: Vec<String>, 
    body: Value,
    macro_env: &mut MacroEnvironment
) -> Result<(), RispyError> {
    let macro_func = MacroFunction { name, params, body };
    macro_env.define(name.to_string(), macro_func)
}
```

### Mutual Recursion Protocol
1. **Collect Definitions**: Parse all `define-macro` forms in current scope
2. **Declare Names**: Register all macro names as placeholders
3. **Define Bodies**: Replace placeholders with actual macro definitions
4. **Validate**: Ensure all references are resolved

## Hygiene System

### Variable Capture Problem
```lisp
;; BAD: Macro accidentally captures 'result' variable
(define-macro bad-when (test body)
  `(let ((result ,test))
     (if result ,body nil)))

(let ((result 42))
  (bad-when #t (print result)))  ; Prints #t instead of 42!
```

### Solution: Gensym (Generated Symbols)
```rust
fn gensym(prefix: &str) -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{}#{}", prefix, id)
}
```

```lisp
;; GOOD: Macro uses generated symbol
(define-macro good-when (test body)
  (let ((temp-var (gensym "test-result")))
    `(let ((,temp-var ,test))
       (if ,temp-var ,body nil))))
```

### Automatic Hygiene
- **Automatic Renaming**: Variables introduced by macros get unique names
- **Lexical Scope Preservation**: Variables from call site maintain their bindings
- **No Manual Gensym**: System automatically generates safe names

## Implementation Architecture

### Core Data Structures

#### MacroFunction
```rust
#[derive(Debug, Clone)]
pub struct MacroFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Value,
    pub env: Rc<MacroEnvironment>,  // Lexical closure for macro definitions
}
```

#### MacroExpander
```rust
pub struct MacroExpander {
    macro_env: Rc<MacroEnvironment>,
    gensym_counter: AtomicUsize,
}

impl MacroExpander {
    pub fn expand(&self, expr: &Value) -> Result<Value, RispyError>;
    pub fn expand_once(&self, expr: &Value) -> Result<Value, RispyError>;
    pub fn is_macro_call(&self, expr: &Value) -> bool;
}
```

### Integration with Evaluator

#### Modified Evaluation Pipeline
```rust
pub fn eval_with_macros(
    expr: &Value, 
    runtime_env: &Rc<Environment>,
    macro_expander: &MacroExpander
) -> Result<Value, RispyError> {
    // Phase 1: Macro expansion
    let expanded = macro_expander.expand(expr)?;
    
    // Phase 2: Runtime evaluation  
    eval(&expanded, runtime_env)
}
```

#### Special Form: define-macro
```rust
fn eval_define_macro(
    args: &[Value], 
    macro_env: &mut MacroEnvironment
) -> Result<Value, RispyError> {
    if args.len() != 3 {
        return Err(RispyError::ArityError(
            "define-macro expects exactly 3 arguments".to_string()
        ));
    }
    
    let name = args[0].expect_symbol()?;
    let params = parse_param_list(&args[1])?;
    let body = args[2].clone();
    
    let macro_func = MacroFunction {
        name: name.to_string(),
        params,
        body,
        env: macro_env.clone(),
    };
    
    macro_env.define(name.to_string(), macro_func);
    Ok(Value::Nil)
}
```

## Quasiquote Implementation

### Quasiquote Evaluation Rules
1. **Literal values**: Return as-is
2. **Unquoted expressions** `,expr`: Evaluate expr in current environment
3. **Unquote-splicing** `,@expr`: Evaluate expr and splice into list
4. **Nested structures**: Recursively apply rules

### Implementation Strategy
```rust
fn eval_quasiquote(
    expr: &Value, 
    bindings: &HashMap<String, Value>
) -> Result<Value, RispyError> {
    match expr {
        Value::List(list) => {
            if let Some((Value::Symbol(op), rest)) = list.split_first() {
                match op.as_str() {
                    "unquote" => eval(&rest[0], bindings),
                    "unquote-splicing" => eval_splice(&rest[0], bindings),
                    _ => eval_quasiquote_list(list, bindings)
                }
            } else {
                eval_quasiquote_list(list, bindings)
            }
        },
        _ => Ok(expr.clone())
    }
}
```

## Error Handling

### Macro-Specific Errors
```rust
#[derive(Debug, Clone)]
pub enum MacroError {
    UndefinedMacro(String),
    MacroExpansionError(String),
    CircularMacroReference(String),
    InvalidMacroDefinition(String),
    QuasiquoteError(String),
    HygieneViolation(String),
}
```

### Error Recovery Strategies
- **Partial Expansion**: Continue expansion even if some macros fail
- **Debugging Support**: Provide macro expansion traces
- **Fallback Behavior**: Treat failed macro calls as regular function calls

## Testing Strategy

### Unit Tests for Macro Components
```rust
#[test]
fn test_simple_macro_expansion() {
    let mut macro_env = MacroEnvironment::new();
    define_macro(&mut macro_env, "when", vec!["test", "body"], 
                 parse("`(if ,test ,body nil)").unwrap());
    
    let call = parse("(when (< 3 5) (print \"yes\"))").unwrap();
    let expanded = expand_macro(&call, &macro_env).unwrap();
    let expected = parse("(if (< 3 5) (print \"yes\") nil)").unwrap();
    
    assert_eq!(expanded, expected);
}
```

### Integration Tests
```rust
#[test]
fn test_macro_with_runtime_evaluation() {
    let input = r#"
        (define-macro when (test body) `(if ,test ,body nil))
        (define x 10)
        (when (< x 20) (set! x (* x 2)))
        x
    "#;
    
    let result = eval_program(input).unwrap();
    assert_eq!(result, Value::Number(20.0));
}
```

### Mutual Recursion Tests
```rust
#[test]
fn test_mutually_recursive_macros() {
    let input = r#"
        (define-macro even? (n) `(if (= ,n 0) #t (odd? (- ,n 1))))
        (define-macro odd? (n) `(if (= ,n 0) #f (even? (- ,n 1))))
        (even? 4)
    "#;
    
    let result = eval_program(input).unwrap();
    assert_eq!(result, Value::Bool(true));
}
```

## Standard Macro Library

### Essential Macros to Implement
```lisp
;; Control Flow
(define-macro when (test body)
  `(if ,test ,body nil))

(define-macro unless (test body)
  `(if ,test nil ,body))

(define-macro cond clauses
  ;; Multi-way conditional (complex implementation)
  )

;; Let Bindings
(define-macro let (bindings body)
  `((lambda ,(map car bindings) ,body) 
    ,@(map cadr bindings)))

;; Logic Operations
(define-macro and (a b)
  `(if ,a ,b #f))

(define-macro or (a b)
  `(if ,a #t ,b))

;; List Processing
(define-macro push (item list)
  `(set! ,list (cons ,item ,list)))

;; Debugging
(define-macro assert (test)
  `(if ,test nil (error "Assertion failed: " ',test)))
```

## Performance Considerations

### Expansion Caching
- **Memoization**: Cache expanded forms to avoid re-expansion
- **Invalidation**: Clear cache when macro definitions change
- **Memory Management**: Use weak references to prevent memory leaks

### Compile-Time Optimization
- **Static Analysis**: Detect expansion patterns that can be optimized
- **Partial Evaluation**: Pre-evaluate constant expressions in templates
- **Dead Code Elimination**: Remove unused macro definitions

## Future Extensions

### Advanced Features
1. **Syntax-Rules**: Pattern-based macro definition system
2. **Procedural Macros**: Full programmatic macro generation
3. **Macro Debugging**: Step-through macro expansion
4. **Reader Macros**: Custom syntax at the lexical level
5. **Module System**: Namespace-aware macro scoping

### Integration Points
- **IDE Support**: Macro expansion visualization
- **Documentation**: Automatic macro documentation generation
- **Optimization**: Compile-time constant folding
- **Type System**: Macro-aware type checking (future)

## Security Considerations

### Sandboxing
- **Limited Evaluation**: Restrict macro-time evaluation capabilities
- **Resource Limits**: Prevent infinite expansion loops
- **Access Control**: Limit macro access to system functions

### Safety Guarantees
- **Hygiene Enforcement**: Prevent accidental variable capture
- **Type Safety**: Ensure macro expansions produce well-typed code
- **Memory Safety**: Leverage Rust's ownership system for macro implementation

## Summary

The RispyBoi macro system provides a powerful metaprogramming foundation while maintaining the language's core simplicity. Key features include:

- ✅ **Two-phase evaluation** separating macro-time from runtime
- ✅ **Mutual recursion support** through two-pass definition
- ✅ **Automatic hygiene** preventing variable capture
- ✅ **Quasiquote system** for template construction
- ✅ **Clean integration** with existing evaluator
- ✅ **Comprehensive testing** strategy for reliability

This architecture enables users to extend RispyBoi's syntax while maintaining correctness and performance, making it suitable for both educational use and practical applications.