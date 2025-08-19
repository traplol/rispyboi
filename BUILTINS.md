# Built-in Functions for RispyBoi

## Overview
The built-ins module (`src/builtins.rs`) implements the minimal set of primitive functions required by the RispyBoi interpreter. Following the design philosophy, only essential operations are implemented in Rust, with most language features to be built as Lisp macros.

## Current Implementation Status
✅ **Implemented**: 12 built-in functions  
✅ **Special Forms**: `set!` implemented as special form (not built-in)
🔄 **Macro System**: Core infrastructure completed, integration in progress
📋 **To Be Implemented as Macros**: Multi-argument arithmetic, derived comparisons, list utilities

## Design Philosophy

### Minimal Primitive Set
- **Only Essential Operations**: Functions that cannot be reasonably implemented as macros
- **Binary Operations Only**: Multi-argument functions will be macro-expanded
- **Type Safety**: All built-ins perform strict type checking
- **Error Propagation**: Comprehensive error handling with meaningful messages

### Built-in Function Categories
1. **Arithmetic**: Basic mathematical operations
2. **Comparison**: Essential comparison operations  
3. **List Operations**: Core list manipulation primitives
4. **Type Predicates**: Runtime type checking
5. **I/O**: Basic input/output operations

## Currently Implemented Built-ins

### Arithmetic Operations (Binary Only)

#### `+` (Addition)
```rust
fn builtin_add(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: Both arguments must be numbers
- **Returns**: Sum as `Value::Number`
- **Examples**: 
  - `(+ 1 2)` → `3`
  - `(+ 3.14 2.86)` → `6.0`
- **Errors**: Type mismatch, wrong arity

#### `-` (Subtraction)
```rust
fn builtin_subtract(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: Both arguments must be numbers
- **Returns**: Difference as `Value::Number`
- **Examples**:
  - `(- 5 3)` → `2`
  - `(- 1.5 0.5)` → `1.0`

#### `*` (Multiplication)
```rust
fn builtin_multiply(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: Both arguments must be numbers
- **Returns**: Product as `Value::Number`
- **Examples**:
  - `(* 3 4)` → `12`
  - `(* 2.5 4)` → `10.0`

#### `/` (Division)
```rust
fn builtin_divide(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: Both arguments must be numbers
- **Returns**: Quotient as `Value::Number`
- **Special Cases**: Division by zero returns error
- **Examples**:
  - `(/ 8 2)` → `4`
  - `(/ 7 2)` → `3.5`
- **Errors**: Division by zero, type mismatch

### Comparison Operations

#### `=` (Equality)
```rust
fn builtin_equal(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: Any two values (structural comparison)
- **Returns**: `Value::Bool`
- **Semantics**: Deep structural equality for all types
- **Examples**:
  - `(= 1 1)` → `#t`
  - `(= "hello" "hello")` → `#t`
  - `(= '(1 2) '(1 2))` → `#t`

#### `<` (Less Than)
```rust
fn builtin_less_than(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: Both arguments must be numbers
- **Returns**: `Value::Bool`
- **Examples**:
  - `(< 1 2)` → `#t`
  - `(< 5 3)` → `#f`

### List Operations

#### `car` (First Element)
```rust
fn builtin_car(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 1 argument
- **Types**: Argument must be a non-empty list
- **Returns**: First element of the list
- **Examples**:
  - `(car '(1 2 3))` → `1`
  - `(car '(a b c))` → `a`
- **Errors**: Empty list, non-list argument

#### `cdr` (Rest of List)
```rust
fn builtin_cdr(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 1 argument
- **Types**: Argument must be a non-empty list
- **Returns**: List containing all elements except the first
- **Examples**:
  - `(cdr '(1 2 3))` → `(2 3)`
  - `(cdr '(a))` → `()`
- **Errors**: Empty list, non-list argument

#### `cons` (Construct List)
```rust
fn builtin_cons(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: First argument any value, second must be list or nil
- **Returns**: New list with first argument prepended
- **Examples**:
  - `(cons 1 '(2 3))` → `(1 2 3)`
  - `(cons 'a nil)` → `(a)`
- **Errors**: Second argument not a list

### Type Predicates

#### `atom?` (Atom Predicate)
```rust
fn builtin_atomp(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 1 argument
- **Types**: Any value
- **Returns**: `#t` if argument is not a list, `#f` otherwise
- **Examples**:
  - `(atom? 42)` → `#t`
  - `(atom? '(1 2))` → `#f`
  - `(atom? nil)` → `#t`

#### `null?` (Null Predicate)
```rust
fn builtin_nullp(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 1 argument  
- **Types**: Any value
- **Returns**: `#t` if argument is nil or empty list, `#f` otherwise
- **Examples**:
  - `(null? nil)` → `#t`
  - `(null? '())` → `#t`
  - `(null? 0)` → `#f`

### I/O Operations

#### `print` (Print Values)
```rust
fn builtin_print(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 1 argument *(Note: Currently differs from spec above)*
- **Types**: Any value
- **Returns**: `nil`
- **Side Effect**: Prints the argument to stdout
- **Examples**:
  - `(print 42)` → prints `42`, returns `nil`
  - `(print "Hello")` → prints `"Hello"`, returns `nil`

## Missing Core Primitives

### Variable Mutation

#### `set!` (Set Variable) - **MISSING**
```rust
fn builtin_set(args: &[Value]) -> Result<Value, RispyError>
```
- **Arity**: Exactly 2 arguments
- **Types**: First must be symbol, second any value  
- **Returns**: The new value
- **Side Effect**: Mutates existing variable in environment
- **Examples**:
  - `(set! x 42)` → sets `x` to `42`, returns `42`
- **Errors**: Undefined variable, non-symbol first argument
- **Status**: ⚠️ **CRITICAL - Required for proper Lisp semantics**

### Additional Comparisons - **COULD BE MACROS**

#### `>` (Greater Than)
- Can be implemented as macro: `(define-macro > (a b) (< b a))`
- **Status**: 📋 Should be macro

#### `<=` (Less Than or Equal)  
- Can be implemented as macro: `(define-macro <= (a b) (not (< b a)))`
- **Status**: 📋 Should be macro

#### `>=` (Greater Than or Equal)
- Can be implemented as macro: `(define-macro >= (a b) (not (< a b)))`  
- **Status**: 📋 Should be macro

#### `!=` (Not Equal)
- Can be implemented as macro: `(define-macro != (a b) (not (= a b)))`
- **Status**: 📋 Should be macro

### Additional Type Predicates - **COULD BE FUNCTIONS OR MACROS**

#### `number?`, `string?`, `symbol?`, `list?`, `function?`
- Can be implemented as built-ins or derived from `atom?` and type checking
- **Status**: 📋 Consider for future implementation

## Functions That Should Be Macros (Not Built-ins)

### Multi-Argument Arithmetic (Will Be Macros)
```lisp
;; Will be macro-expanded to binary operations once macro system is complete
(+ 1 2 3 4)      ; → (+ (+ (+ 1 2) 3) 4)
(* 2 3 4 5)      ; → (* (* (* 2 3) 4) 5)
(- 10 2 3)       ; → (- (- 10 2) 3)
```

### Logical Operations (Will Be Macros)
```lisp
;; Will be implemented as macros using 'if' once macro system is complete
(define-macro and (a b) `(if ,a ,b #f))
(define-macro or (a b) `(if ,a #t ,b))
(define-macro not (a) `(if ,a #f #t))
```

### List Utilities
```lisp
;; Recursive functions, not primitives
(define-function length (lst)
  (if (null? lst) 0 (+ 1 (length (cdr lst)))))

(define-function append (lst1 lst2)
  (if (null? lst1) lst2 (cons (car lst1) (append (cdr lst1) lst2))))
```

### Conditional Macros
```lisp
;; Multi-branch conditionals
(define-macro cond (clauses...)  ; Complex macro expansion
(define-macro when (test body)   ; → (if test body nil)
(define-macro unless (test body) ; → (if test nil body)
```

## Priority Implementation Order

### High Priority (Core Language)
1. ✅ **Arithmetic**: `+`, `-`, `*`, `/` 
2. ✅ **Comparison**: `=`, `<`
3. ✅ **Lists**: `car`, `cdr`, `cons`
4. ✅ **Predicates**: `atom?`, `null?`
5. ⚠️ **MISSING**: `set!` - Essential for variable mutation

### Medium Priority (Quality of Life)
6. 📋 **Logic**: `not` (could be built-in for efficiency)
7. 📋 **Type Predicates**: `number?`, `string?`, `symbol?` 
8. 📋 **Math**: `mod`, `abs` (if used frequently)

### Low Priority (Should Be Macros)
9. 📋 **Multi-arg arithmetic**: `(+ 1 2 3)`
10. 📋 **Derived comparisons**: `>`, `<=`, `>=`, `!=`
11. 📋 **List utilities**: `length`, `append`, `reverse`

## Implementation Architecture

### Current Built-in Function Registry
```rust
pub fn create_builtin_registry() -> HashMap<String, Function> {
    let mut registry = HashMap::new();

    // Arithmetic operations (✅ Implemented)
    registry.insert("+".to_string(), Function::Builtin(builtin_add()));
    registry.insert("-".to_string(), Function::Builtin(builtin_subtract()));
    registry.insert("*".to_string(), Function::Builtin(builtin_multiply()));
    registry.insert("/".to_string(), Function::Builtin(builtin_divide()));

    // Comparison operations (✅ Implemented)
    registry.insert("=".to_string(), Function::Builtin(builtin_equal()));
    registry.insert("<".to_string(), Function::Builtin(builtin_less_than()));

    // List operations (✅ Implemented)
    registry.insert("car".to_string(), Function::Builtin(builtin_car()));
    registry.insert("cdr".to_string(), Function::Builtin(builtin_cdr()));
    registry.insert("cons".to_string(), Function::Builtin(builtin_cons()));

    // Type predicates (✅ Implemented)
    registry.insert("atom?".to_string(), Function::Builtin(builtin_atom_p()));
    registry.insert("null?".to_string(), Function::Builtin(builtin_null_p()));

    // I/O (✅ Implemented)
    registry.insert("print".to_string(), Function::Builtin(builtin_print()));

    registry
}
```

**Total: 12 built-in functions currently implemented**

### Function Implementation Pattern
```rust
fn builtin_add(args: &[Value]) -> Result<Value, RispyError> {
    // Arity checking is done by the evaluator
    let a = args[0].expect_number()?;
    let b = args[1].expect_number()?;
    Ok(Value::Number(a + b))
}

fn builtin_equal(args: &[Value]) -> Result<Value, RispyError> {
    Ok(Value::Bool(args[0] == args[1]))
}

fn builtin_car(args: &[Value]) -> Result<Value, RispyError> {
    let list = args[0].expect_list()?;
    if list.is_empty() {
        Err(RispyError::RuntimeError("car: cannot get car of empty list".to_string()))
    } else {
        Ok(list[0].clone())
    }
}
```

## Error Handling Strategy

### Type Checking
- Use `Value::expect_*` methods for type validation
- Provide specific error messages indicating expected vs actual types
- Include function name in error messages for context

### Arity Validation  
- Performed by evaluator before calling built-in function
- Built-ins can assume correct arity
- Variadic functions handle variable argument counts internally

### Runtime Errors
- Division by zero in arithmetic operations
- Empty list operations (car, cdr on empty lists)
- Type mismatches with clear error messages

### Error Message Format
```
RuntimeError: "car: cannot get car of empty list"
TypeError: "Expected number, got string"
ArityError: "Expected 2 arguments, got 3"
```

## Integration with Evaluator

### Function Resolution
1. Symbol lookup in environment returns `Value::Function`
2. Pattern match on `Function::Builtin(builtin_func)`
3. Arity checking using `builtin_func.arity`
4. Argument evaluation and type checking
5. Function call via `(builtin_func.func)(args)`

### Environment Initialization
```rust
pub fn create_global_environment() -> Environment {
    let mut env = Environment::new();
    let registry = BuiltinRegistry::new();
    registry.populate_environment(&mut env);
    env
}
```

## Testing Strategy

### Unit Tests for Each Built-in
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_add() {
        let args = vec![Value::Number(1.0), Value::Number(2.0)];
        let result = builtin_add(&args).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_builtin_add_type_error() {
        let args = vec![Value::Number(1.0), Value::String("hello".to_string())];
        assert!(builtin_add(&args).is_err());
    }

    #[test]
    fn test_builtin_car() {
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let args = vec![list];
        let result = builtin_car(&args).unwrap();
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_builtin_car_empty_list() {
        let args = vec![Value::List(vec![])];
        assert!(builtin_car(&args).is_err());
    }
}
```

### Integration Tests
- Test built-ins through the evaluator
- Verify environment population
- Test error propagation through evaluation chain

### Property Tests (Future)
- Arithmetic operation properties (commutativity, etc.)
- List operation invariants
- Type predicate consistency

## Performance Considerations

### Function Call Overhead
- Direct function pointer calls minimize overhead
- No dynamic dispatch beyond initial pattern matching
- Argument validation done efficiently with `expect_*` methods

### Memory Usage
- Built-in functions are stateless
- No closure capture overhead
- Minimal heap allocation during execution

### Future Optimizations
1. **Inline Caching**: Cache function lookups for repeated calls
2. **Specialized Arithmetic**: Separate integer/float arithmetic paths
3. **SIMD Operations**: Vector operations for list processing

## Macro Integration Points

### Multi-Argument Arithmetic
```lisp
;; This macro expansion happens before evaluation
(+ 1 2 3 4)  ; Expands to: (+ (+ (+ 1 2) 3) 4)
```

### Derived Comparisons
```lisp
;; Built as macros using primitive comparisons
(define-macro > (a b) `(< ,b ,a))
(define-macro <= (a b) `(not (< ,b ,a)))
(define-macro >= (a b) `(not (< ,a ,b)))
(define-macro /= (a b) `(not (= ,a ,b)))
```

### List Utilities
```lisp
;; Higher-level list functions as macros
(define-macro length (lst)
  `(if (null? ,lst) 0 (+ 1 (length (cdr ,lst)))))
```

## Extension Points

### Adding New Built-ins
1. Implement function following the signature pattern
2. Add to `register_all()` method
3. Add comprehensive tests
4. Update documentation

### Custom Types Support
- Built-ins can be extended to handle new `Value` variants
- Type predicates can be added for custom types
- Error handling remains consistent

### Platform-Specific Built-ins
- I/O operations can be platform-specialized
- File system operations
- Network operations (future)

## Summary

### What We Have (12 functions)
- ✅ **Complete core arithmetic**: `+`, `-`, `*`, `/`
- ✅ **Complete basic comparison**: `=`, `<`  
- ✅ **Complete list primitives**: `car`, `cdr`, `cons`
- ✅ **Complete type predicates**: `atom?`, `null?`
- ✅ **Basic I/O**: `print`

### What We're Missing
- ⚠️ **CRITICAL**: `set!` - Variable mutation (required for proper Lisp)
- 📋 **Nice to have**: `not`, more type predicates (`number?`, `string?`, etc.)
- 📋 **Should be macros**: `>`, `<=`, `>=`, `!=`, multi-arg arithmetic

### Current Status and Recommendation
**✅ Complete**: `set!` has been implemented as a special form and is working correctly.

**🔄 Current Priority**: The macro system is being actively developed to enable derived operators and multi-argument functions. Core infrastructure is complete and integration is in progress.

## Security Considerations

### Input Validation
- All built-ins validate input types strictly
- No buffer overflows possible with Rust's safety
- Arithmetic operations check for edge cases

### Resource Limits
- No unbounded operations in built-ins
- Stack depth limited by Rust's stack size
- Memory usage proportional to input size
