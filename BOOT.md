# RispyBoi Boot Library Specification

## Overview
This document specifies the `boot.lisp` standard library that provides essential utility macros and functions for RispyBoi. The boot library is automatically loaded at startup and provides the foundational building blocks that make Lisp programming more convenient and expressive.

## Boot Loading Strategy

### Automatic Loading
- `boot.lisp` is loaded automatically when RispyBoi starts
- Located in the same directory as the executable or a standard library path
- Loaded before any user code is executed
- All definitions become part of the global environment

### Error Handling
- Boot loading failures are reported clearly to the user
- Critical errors prevent interpreter startup
- Missing boot.lisp file results in a warning but allows continued operation

## Core Design Philosophy

### Minimal Primitives + Rich Macros
- Leverage RispyBoi's minimal primitive set (arithmetic, comparison, lists, etc.)
- Build comprehensive functionality through well-designed macros
- Prefer macros over functions where compile-time expansion provides benefits

### Backward Compatibility
- All existing RispyBoi code continues to work unchanged
- Boot library extends but does not override existing functionality
- Standard Lisp conventions where possible

## Logical Operators

### Binary Logic Operations
```lisp
;; AND - short-circuiting logical and
(define-macro and (a b)
  `(if ,a ,b #f))

;; OR - short-circuiting logical or  
(define-macro or (a b)
  `(if ,a #t ,b))

;; NOT - logical negation
(define-macro not (a)
  `(if ,a #f #t))
```

**Usage Examples:**
```lisp
(and #t #t)        ; → #t
(and #t #f)        ; → #f
(or #f #t)         ; → #t
(not #t)           ; → #f
(and (< 3 5) (> 10 8))  ; → #t (short-circuits)
```

### Multi-Argument Logic (Future)
```lisp
;; Multi-argument versions for convenience
(define-macro and* args
  ;; Expand to nested binary operations
  )
```

## Enhanced Comparison Operators

### Derived Comparisons
```lisp
;; Greater than
(define-macro > (a b)
  `(< ,b ,a))

;; Less than or equal
(define-macro <= (a b)
  `(not (< ,b ,a)))

;; Greater than or equal
(define-macro >= (a b)
  `(not (< ,a ,b)))

;; Not equal
(define-macro != (a b)
  `(not (= ,a ,b)))
```

**Usage Examples:**
```lisp
(> 5 3)           ; → #t
(<= 3 3)          ; → #t
(>= 5 3)          ; → #t
(!= 3 4)          ; → #t
```

## Multi-Argument Arithmetic

### Extended Arithmetic Operations
```lisp
;; Multi-argument addition
(define-macro +* args
  ;; Expand (+ a b c d) to (+ (+ (+ a b) c) d)
  ;; Implementation uses recursive macro expansion
  )

;; Multi-argument multiplication
(define-macro ** args
  ;; Similar expansion for multiplication
  )

;; Multi-argument subtraction (left-associative)
(define-macro -* args
  ;; (- a b c) → (- (- a b) c)
  )
```

**Usage Examples:**
```lisp
(+* 1 2 3 4)      ; → 10
(** 2 3 4)        ; → 24
(-* 10 3 2)       ; → 5
```

## Control Flow Macros

### Conditional Macros
```lisp
;; WHEN - conditional execution without else clause
(define-macro when (test body)
  `(if ,test ,body nil))

;; UNLESS - conditional execution with negated test
(define-macro unless (test body)
  `(if ,test nil ,body))

;; COND - multi-way conditional
(define-macro cond clauses
  ;; Expands to nested if statements
  ;; (cond (test1 result1) (test2 result2) (else result3))
  ;; → (if test1 result1 (if test2 result2 result3))
  )
```

**Usage Examples:**
```lisp
(when (< x 10)
  (print "x is small"))

(unless (null? lst)
  (process-list lst))

(cond 
  ((< x 0) "negative")
  ((= x 0) "zero")
  (else "positive"))
```

### Loop Constructs (Future)
```lisp
;; Basic looping - requires additional primitive support
(define-macro while (test body)
  ;; Implementation pending tail call optimization
  )
```

## Variable Binding Macros

### LET - Local Variable Binding
```lisp
;; LET - create local variable scope
(define-macro let (bindings body)
  ;; (let ((x 1) (y 2)) (+ x y))
  ;; → ((lambda (x y) (+ x y)) 1 2)
  )

;; LET* - sequential variable binding
(define-macro let* (bindings body)
  ;; Expands to nested let expressions for sequential binding
  )
```

**Usage Examples:**
```lisp
(let ((x 10) (y 20))
  (+ x y))             ; → 30

(let* ((x 10) (y (* x 2)))
  (+ x y))             ; → 30
```

## List Processing Functions

### Essential List Utilities
```lisp
;; LENGTH - calculate list length
(define-function length (lst)
  (if (null? lst) 
      0 
      (+ 1 (length (cdr lst)))))

;; APPEND - concatenate two lists
(define-function append (lst1 lst2)
  (if (null? lst1)
      lst2
      (cons (car lst1) (append (cdr lst1) lst2))))

;; REVERSE - reverse a list
(define-function reverse (lst)
  ;; Tail-recursive implementation for efficiency
  (define-function reverse-helper (lst acc)
    (if (null? lst)
        acc
        (reverse-helper (cdr lst) (cons (car lst) acc))))
  (reverse-helper lst ()))

;; MAP - apply function to each element
(define-function map (f lst)
  (if (null? lst)
      ()
      (cons (f (car lst)) (map f (cdr lst)))))

;; FILTER - select elements matching predicate
(define-function filter (pred lst)
  (if (null? lst)
      ()
      (if (pred (car lst))
          (cons (car lst) (filter pred (cdr lst)))
          (filter pred (cdr lst)))))

;; FOLD-LEFT - left-associative fold
(define-function fold-left (f init lst)
  (if (null? lst)
      init
      (fold-left f (f init (car lst)) (cdr lst))))

;; FOLD-RIGHT - right-associative fold
(define-function fold-right (f init lst)
  (if (null? lst)
      init
      (f (car lst) (fold-right f init (cdr lst)))))
```

**Usage Examples:**
```lisp
(length '(1 2 3 4))              ; → 4
(append '(1 2) '(3 4))           ; → (1 2 3 4)
(reverse '(1 2 3))               ; → (3 2 1)
(map (lambda (x) (* x 2)) '(1 2 3))  ; → (2 4 6)
(filter atom? '(1 (2 3) 4))      ; → (1 4)
(fold-left + 0 '(1 2 3 4))       ; → 10
```

## Type Predicates and Utilities

### Enhanced Type Checking
```lisp
;; Additional type predicates
(define-function number? (x)
  (and (atom? x) (not (null? x)) (not (atom? x))))  ; Needs refinement

(define-function string? (x)
  ;; Implementation depends on internal type representation
  )

(define-function symbol? (x)
  ;; Implementation depends on internal type representation
  )

(define-function list? (x)
  (not (atom? x)))

(define-function function? (x)
  ;; Check if x is a callable function
  )

;; PAIR? - check if value is a non-empty list
(define-function pair? (x)
  (and (list? x) (not (null? x))))

;; ZERO? - check if number is zero
(define-function zero? (x)
  (= x 0))

;; POSITIVE? - check if number is positive
(define-function positive? (x)
  (> x 0))

;; NEGATIVE? - check if number is negative
(define-function negative? (x)
  (< x 0))
```

## Mathematical Utilities

### Extended Math Functions
```lisp
;; ABS - absolute value
(define-function abs (x)
  (if (< x 0) (- 0 x) x))

;; MIN - minimum of two values
(define-function min (a b)
  (if (< a b) a b))

;; MAX - maximum of two values  
(define-function max (a b)
  (if (< a b) b a))

;; SQUARE - square a number
(define-function square (x)
  (* x x))

;; EVEN? - check if number is even
(define-function even? (x)
  (= (% x 2) 0))  ; Requires modulo operator

;; ODD? - check if number is odd
(define-function odd? (x)
  (not (even? x)))
```

## I/O and Debugging Utilities

### Enhanced Output Functions
```lisp
;; PRINTLN - print with newline
(define-function println (x)
  ;; Implementation requires enhanced print primitive
  )

;; DISPLAY - print without quotes for strings
(define-function display (x)
  ;; Implementation requires display primitive
  )

;; NEWLINE - print a newline
(define-function newline ()
  ;; Implementation requires newline primitive
  )
```

### Debugging Macros
```lisp
;; DEBUG - conditional debug output
(define-macro debug (expr)
  `(if *debug-mode*
       (print (quote ,expr) " = " ,expr)
       ,expr))

;; ASSERT - runtime assertion checking
(define-macro assert (test)
  `(if ,test
       #t
       (error "Assertion failed: " (quote ,test))))

;; TRACE - function call tracing
(define-macro trace (name)
  ;; Implementation requires function wrapping capability
  )
```

## Advanced Macros

### Function Definition Utilities
```lisp
;; LAMBDA - anonymous function shorthand
(define-macro lambda (params body)
  `(define-function ,(gensym "lambda") ,params ,body))

;; DEFUN - alternative function definition syntax
(define-macro defun (name params body)
  `(define-function ,name ,params ,body))
```

### List Construction Macros
```lisp
;; LIST - construct list from arguments
(define-macro list args
  ;; Expand to nested cons operations
  `(cons ,(car args) 
         ,(if (null? (cdr args)) 
              'nil 
              `(list ,@(cdr args)))))
```

## Error Handling

### Exception-Style Error Handling
```lisp
;; ERROR - signal an error with message
(define-function error (message)
  ;; Implementation requires error signaling primitive
  )

;; TRY-CATCH - basic exception handling (future)
(define-macro try (body catch-clause)
  ;; Requires exception handling primitives
  )
```

## Boot Library Implementation Strategy

### Phase 1: Basic Macros (Immediate)
1. Logical operators (`and`, `or`, `not`)
2. Comparison operators (`>`, `<=`, `>=`, `!=`)
3. Control flow (`when`, `unless`)
4. Basic list functions (`length`, `append`, `reverse`)

### Phase 2: Enhanced Functionality
1. Multi-argument arithmetic
2. Advanced control flow (`cond`)
3. Variable binding (`let`, `let*`)
4. Higher-order functions (`map`, `filter`, `fold`)

### Phase 3: Advanced Features
1. Lambda expressions
2. Debugging utilities
3. Enhanced I/O functions
4. Error handling primitives

## Testing Strategy

### Boot Library Tests
```lisp
;; Comprehensive test suite for all boot library functions
;; Tests should verify:
;; - Correct expansion of macros
;; - Proper evaluation semantics
;; - Edge case handling
;; - Integration with existing primitives
```

### Integration Tests
- Verify boot library loads correctly at startup
- Test interaction between boot functions and user code
- Performance testing for recursive functions
- Memory usage verification

## Documentation Requirements

### Function Documentation
Each function/macro should include:
- Purpose and behavior description
- Parameter types and meanings
- Return value specification
- Usage examples
- Implementation notes

### User Manual Integration
- Boot library functions become part of standard language reference
- Examples integrated into tutorials
- Migration guide for users transitioning from minimal primitives

## Compatibility and Evolution

### Backward Compatibility
- Existing RispyBoi code remains fully functional
- Boot library provides extensions, not replacements
- Clear separation between primitive and derived operations

### Future Extensions
- Boot library designed for easy extension
- Plugin architecture for additional libraries
- Standard library modules for specialized domains

## File Structure and Organization

### boot.lisp Organization
```lisp
;; boot.lisp - RispyBoi Standard Library
;; Automatically loaded at interpreter startup

;; Section 1: Logical Operators
;; Section 2: Enhanced Comparisons  
;; Section 3: Control Flow Macros
;; Section 4: List Processing Functions
;; Section 5: Mathematical Utilities
;; Section 6: Type Predicates
;; Section 7: I/O and Debugging
;; Section 8: Advanced Macros
```

### Loading Order
1. Basic logical operators (used by subsequent definitions)
2. Comparison operators
3. Control flow macros
4. Utility functions
5. Advanced features

## Performance Considerations

### Macro vs Function Trade-offs
- Macros: Better performance (compile-time expansion), no function call overhead
- Functions: More flexible, can be passed as values, easier to debug

### Recursive Function Optimization
- Tail-recursive implementations where possible
- Iterative alternatives for stack-intensive operations
- Clear documentation of performance characteristics

## Security and Safety

### Safe Defaults
- Error checking in utility functions
- Graceful handling of edge cases (empty lists, null values)
- Clear error messages for misuse

### Resource Management
- Prevent infinite recursion where possible
- Memory-conscious implementations
- Stack overflow protection

## Summary

The boot.lisp library transforms RispyBoi from a minimal Lisp implementation into a practical programming environment by providing essential utilities and syntactic conveniences. The library leverages the macro system to build rich functionality on top of minimal primitives while maintaining the simplicity and elegance that makes Lisp powerful.

This specification provides a roadmap for implementing a comprehensive standard library that makes RispyBoi suitable for real-world Lisp programming while serving as an excellent educational platform for understanding how complex language features can be built from simple foundations.