# Evaluator Architecture for RispyBoi

## Overview
The evaluator module (`src/eval.rs`) is the heart of the RispyBoi interpreter, responsible for executing parsed Abstract Syntax Trees (AST) represented as `Value` structures. It implements the core evaluation semantics of Lisp, including function application, symbol resolution, and special form handling.

## Design Philosophy

### Core Principles
1. **Recursive Evaluation**: Natural fit for nested S-expressions
2. **Environment-Based**: Lexical scoping with environment chains
3. **Fail Fast**: Stop evaluation immediately on errors
4. **Pure Functions**: Evaluation doesn't modify AST, only environment
5. **Special Form Integration**: Seamless handling of built-ins vs special forms

### Evaluation Strategy
- **Call-by-Value**: Arguments evaluated before function application
- **Lexical Scoping**: Variable lookup follows environment chain
- **Tail Call Ready**: Architecture supports future tail call optimization

## Core Evaluation Function

### Main Evaluator
```rust
pub fn eval(expr: &Value, env: &mut Environment) -> Result<Value, RispyError>
```
- **Input**: Expression AST and mutable environment
- **Output**: Evaluated result value
- **Side Effects**: May modify environment (define, set!)
- **Error Handling**: Propagates all evaluation errors

### Evaluation Dispatch
```rust
pub fn eval(expr: &Value, env: &mut Environment) -> Result<Value, RispyError> {
    match expr {
        // Self-evaluating forms
        Value::Nil | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            Ok(expr.clone())
        }
        
        // Symbol lookup
        Value::Symbol(name) => eval_symbol(name, env),
        
        // List evaluation (function application or special form)
        Value::List(list) => eval_list(list, env),
        
        // Functions are values but shouldn't be directly evaluated
        Value::Function(_) => Ok(expr.clone()),
    }
}
```

## Expression Type Evaluation

### Self-Evaluating Forms
```rust
// Numbers, strings, booleans, and nil evaluate to themselves
42        → 42
"hello"   → "hello" 
#t        → #t
nil       → nil
```

### Symbol Evaluation
```rust
fn eval_symbol(name: &str, env: &Environment) -> Result<Value, RispyError> {
    env.get(name)
        .cloned()
        .ok_or_else(|| RispyError::UndefinedSymbol(format!("Undefined variable: {}", name)))
}
```
- **Lookup Chain**: Current environment → parent → ... → global
- **Error**: Undefined symbol if not found in any environment
- **Examples**:
  - `x` → looks up value bound to symbol `x`
  - `+` → returns built-in addition function

### List Evaluation
```rust
fn eval_list(list: &[Value], env: &mut Environment) -> Result<Value, RispyError> {
    if list.is_empty() {
        return Ok(Value::List(vec![])); // Empty list evaluates to itself
    }

    let first = &list[0];
    
    // Check for special forms first
    if let Value::Symbol(name) = first {
        match name.as_str() {
            "quote" => eval_quote(&list[1..], env),
            "if" => eval_if(&list[1..], env),
            "define" => eval_define(&list[1..], env),
            "lambda" => eval_lambda(&list[1..], env),
            _ => eval_function_application(list, env),
        }
    } else {
        eval_function_application(list, env)
    }
}
```

## Function Application

### Function Call Evaluation
```rust
fn eval_function_application(list: &[Value], env: &mut Environment) -> Result<Value, RispyError> {
    // Evaluate the operator (first element)
    let operator = eval(&list[0], env)?;
    
    // Evaluate all arguments
    let mut args = Vec::new();
    for arg in &list[1..] {
        args.push(eval(arg, env)?);
    }
    
    // Apply the function
    apply_function(&operator, &args, env)
}
```

### Function Application Dispatch
```rust
fn apply_function(func: &Value, args: &[Value], env: &mut Environment) -> Result<Value, RispyError> {
    match func {
        Value::Function(function) => {
            match function.as_ref() {
                Function::Builtin(builtin) => apply_builtin(builtin, args),
                Function::UserDefined(user_func) => apply_user_function(user_func, args, env),
                Function::Macro(macro_func) => apply_macro(macro_func, args, env),
            }
        }
        _ => Err(RispyError::TypeError(format!(
            "Cannot apply non-function: {}", 
            func.type_name()
        ))),
    }
}
```

### Built-in Function Application
```rust
fn apply_builtin(builtin: &BuiltinFunction, args: &[Value]) -> Result<Value, RispyError> {
    // Check arity
    check_arity(args, &builtin.arity)?;
    
    // Call the built-in function
    (builtin.func)(args)
}
```

### User-Defined Function Application
```rust
fn apply_user_function(
    func: &UserFunction, 
    args: &[Value], 
    env: &mut Environment
) -> Result<Value, RispyError> {
    // Check arity
    if args.len() != func.params.len() {
        return Err(RispyError::ArityError(format!(
            "Expected {} arguments, got {}", 
            func.params.len(), 
            args.len()
        )));
    }
    
    // Create new environment with function's closure as parent
    let mut new_env = Environment::with_parent(func.closure.clone());
    
    // Bind parameters to arguments
    for (param, arg) in func.params.iter().zip(args.iter()) {
        new_env.define(param.clone(), arg.clone());
    }
    
    // Evaluate function body in new environment
    eval(&func.body, &mut new_env)
}
```

## Special Forms Implementation

### Quote Special Form
```rust
fn eval_quote(args: &[Value], _env: &mut Environment) -> Result<Value, RispyError> {
    if args.len() != 1 {
        return Err(RispyError::ArityError(
            "quote expects exactly 1 argument".to_string()
        ));
    }
    
    // Return the argument without evaluation
    Ok(args[0].clone())
}
```
- **Purpose**: Prevent evaluation of expressions
- **Examples**:
  - `(quote x)` → `x` (symbol, not variable lookup)
  - `(quote (1 2 3))` → `(1 2 3)` (list, not function call)

### If Conditional
```rust
fn eval_if(args: &[Value], env: &mut Environment) -> Result<Value, RispyError> {
    if args.len() != 3 {
        return Err(RispyError::ArityError(
            "if expects exactly 3 arguments (condition then else)".to_string()
        ));
    }
    
    let condition = eval(&args[0], env)?;
    
    if is_truthy(&condition) {
        eval(&args[1], env)  // then branch
    } else {
        eval(&args[2], env)  // else branch
    }
}
```
- **Arity**: Exactly 3 arguments (condition, then, else)
- **Semantics**: Only one branch is evaluated based on condition
- **Truthiness**: Only `nil` and `#f` are falsy, everything else is truthy

### Define Variable/Function
```rust
fn eval_define(args: &[Value], env: &mut Environment) -> Result<Value, RispyError> {
    if args.len() != 2 {
        return Err(RispyError::ArityError(
            "define expects exactly 2 arguments".to_string()
        ));
    }
    
    let name = args[0].expect_symbol()?;
    let value = eval(&args[1], env)?;
    
    env.define(name.to_string(), value.clone());
    Ok(value)
}
```
- **Purpose**: Bind symbols to values in current environment
- **Examples**:
  - `(define x 42)` → binds `x` to `42`
  - `(define f (lambda (x) (* x x)))` → binds `f` to function

### Lambda Function Creation
```rust
fn eval_lambda(args: &[Value], env: &mut Environment) -> Result<Value, RispyError> {
    if args.len() != 2 {
        return Err(RispyError::ArityError(
            "lambda expects exactly 2 arguments (params body)".to_string()
        ));
    }
    
    // Extract parameter list
    let params_list = args[0].expect_list()?;
    let mut params = Vec::new();
    for param in params_list {
        params.push(param.expect_symbol()?.to_string());
    }
    
    // Create user-defined function
    let user_func = UserFunction {
        name: None,
        params,
        body: Box::new(args[1].clone()),
        closure: env.clone(), // Capture current environment
    };
    
    Ok(Value::Function(Box::new(Function::UserDefined(Box::new(user_func)))))
}
```
- **Closure Capture**: Function captures current environment
- **Lexical Scoping**: Parameters shadow outer bindings
- **Examples**: `(lambda (x y) (+ x y))` → creates binary addition function

## Environment Management

### Environment Creation and Scoping
```rust
pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    
    // Populate with built-in functions
    let registry = crate::builtins::BuiltinRegistry::new();
    registry.populate_environment(&mut env);
    
    env
}
```

### Variable Binding Semantics
- **Define**: Creates new binding in current environment
- **Set**: Modifies existing binding (searches environment chain)
- **Lookup**: Searches current → parent → ... → global environment

### Lexical Scoping Implementation
```rust
// Function application creates new environment with closure as parent
let mut new_env = Environment::with_parent(func.closure.clone());

// Parameters are bound in the new environment
for (param, arg) in func.params.iter().zip(args.iter()) {
    new_env.define(param.clone(), arg.clone());
}

// Body evaluated in new environment
eval(&func.body, &mut new_env)
```

## Error Handling Strategy

### Error Types and Propagation
```rust
// All evaluation errors propagate up the call stack
eval(&expr, env)?  // ? operator for error propagation

// Specific error types for different failure modes
RispyError::UndefinedSymbol("Undefined variable: x".to_string())
RispyError::TypeError("Cannot apply non-function: number".to_string())  
RispyError::ArityError("Expected 2 arguments, got 3".to_string())
RispyError::RuntimeError("Division by zero".to_string())
```

### Error Context Enhancement
- Function names included in error messages
- Argument count mismatches clearly reported
- Type mismatches show expected vs actual types

## Evaluation Examples

### Simple Arithmetic
```lisp
Input:  (+ 1 2)
Parse:  List([Symbol("+"), Number(1.0), Number(2.0)])
Eval:   
  1. eval_list([Symbol("+"), Number(1.0), Number(2.0)])
  2. eval_function_application()
  3. eval(Symbol("+")) → builtin_add function
  4. eval(Number(1.0)) → 1.0
  5. eval(Number(2.0)) → 2.0  
  6. apply_builtin(builtin_add, [1.0, 2.0]) → 3.0
Result: Number(3.0)
```

### Variable Definition and Use
```lisp
Input:  (define x 42)
Eval:   
  1. eval_define([Symbol("x"), Number(42.0)])
  2. eval(Number(42.0)) → 42.0
  3. env.define("x", 42.0)
Result: Number(42.0)

Input:  x  
Eval:   
  1. eval_symbol("x") 
  2. env.get("x") → Some(42.0)
Result: Number(42.0)
```

### Function Definition and Application
```lisp
Input:  (define square (lambda (x) (* x x)))
Eval:   
  1. eval_define([Symbol("square"), List([...])])
  2. eval_lambda([List([Symbol("x")]), List([...])])
  3. Create UserFunction with closure
  4. env.define("square", function)

Input:  (square 5)
Eval:   
  1. eval_function_application([Symbol("square"), Number(5.0)])
  2. eval(Symbol("square")) → user function
  3. eval(Number(5.0)) → 5.0
  4. apply_user_function(square_func, [5.0])
  5. Create new environment, bind x=5.0
  6. eval(List([Symbol("*"), Symbol("x"), Symbol("x")]))
  7. Arithmetic evaluation → 25.0
Result: Number(25.0)
```

## Integration Points

### With Parser
- Evaluator receives parsed `Value` AST from parser
- No syntax analysis needed, pure semantic evaluation
- Error locations could be enhanced with parser position info

### With Built-ins
- Built-in functions registered in global environment
- Function application dispatches to built-in implementations
- Arity checking coordinated between evaluator and built-ins

### With Special Forms
- Special forms handled before function application
- Each special form has specific evaluation semantics
- Future special forms can be added to dispatch table

## Performance Considerations

### Evaluation Efficiency
- **Tail Call Optimization**: Architecture supports future TCO implementation
- **Environment Copying**: Current clone-based approach trades memory for simplicity
- **Function Call Overhead**: Direct dispatch minimizes virtual call overhead

### Memory Usage
- **Value Cloning**: Frequent cloning during evaluation
- **Environment Chains**: Parent references prevent garbage collection cycles
- **Future Optimization**: Reference counting or arena allocation

### Stack Usage
- **Recursive Evaluation**: Stack depth proportional to expression nesting
- **Function Calls**: Each application creates new stack frame
- **Tail Call Future**: TCO would eliminate stack growth for tail-recursive calls

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_number() {
        let mut env = Environment::new();
        let result = eval(&Value::Number(42.0), &mut env).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut env = create_global_environment();
        let expr = parse_single("(+ 1 2)").unwrap();
        let result = eval(&expr, &mut env).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_undefined_variable() {
        let mut env = Environment::new();
        let result = eval(&Value::Symbol("undefined".to_string()), &mut env);
        assert!(matches!(result, Err(RispyError::UndefinedSymbol(_))));
    }
}
```

### Integration Tests
- Complete expressions through lexer → parser → evaluator
- Error propagation testing
- Environment scoping validation
- Function definition and application

### REPL Testing
- Interactive evaluation scenarios
- Multi-expression evaluation
- Error recovery and reporting

## Future Enhancements

### Tail Call Optimization
```rust
// Future TCO implementation point
fn eval_in_tail_position(expr: &Value, env: &mut Environment) -> Result<Value, RispyError> {
    // Iterative evaluation for tail calls
    // Eliminates stack growth for recursive functions
}
```

### Debugging Support
- **Call Stack Traces**: Track evaluation context for better error messages
- **Step-by-Step Evaluation**: Support for debugger integration
- **Variable Inspection**: Environment introspection capabilities

### Performance Optimizations
- **Bytecode Compilation**: Compile AST to bytecode for faster evaluation
- **Inline Caching**: Cache function lookups and type checks
- **Specialized Evaluation**: Fast paths for common expression patterns

### Extended Special Forms
- **cond**: Multi-branch conditional
- **let**: Local variable binding
- **begin**: Sequential evaluation
- **set!**: Variable mutation
