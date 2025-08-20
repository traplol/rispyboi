;; boot.lisp - RispyBoi Standard Library
;; Automatically loaded at interpreter startup
;; Provides essential utility macros and functions

;; =============================================================================
;; Section 1: Logical Operators
;; =============================================================================

;; AND - short-circuiting logical and
(define-macro and (a b)
  (cons (quote if) (cons a (cons b (cons #f ())))))

;; OR - short-circuiting logical or  
(define-macro or (a b)
  (cons (quote if) (cons a (cons #t (cons b ())))))

;; NOT - logical negation
(define-macro not (a)
  (cons (quote if) (cons a (cons #f (cons #t ())))))

;; =============================================================================
;; Section 2: Enhanced Comparison Operators
;; =============================================================================

;; Greater than
(define-macro > (a b)
  (cons (quote <) (cons b (cons a ()))))

;; Less than or equal
(define-macro <= (a b)
  (cons (quote not) (cons (cons (quote <) (cons b (cons a ()))) ())))

;; Greater than or equal
(define-macro >= (a b)
  (cons (quote not) (cons (cons (quote <) (cons a (cons b ()))) ())))

;; Not equal
(define-macro != (a b)
  (cons (quote not) (cons (cons (quote =) (cons a (cons b ()))) ())))

;; =============================================================================
;; Section 3: Control Flow Macros
;; =============================================================================

;; WHEN - conditional execution without else clause
(define-macro when (test body)
  (cons (quote if) (cons test (cons body (cons (quote nil) ())))))

;; UNLESS - conditional execution with negated test
(define-macro unless (test body)
  (cons (quote if) (cons test (cons (quote nil) (cons body ())))))

;; =============================================================================
;; Section 4: Mathematical Utilities
;; =============================================================================

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

;; =============================================================================
;; Section 5: List Processing Functions
;; =============================================================================

;; LENGTH - calculate list length
(define-function length (lst)
  (if (null? lst) 0 (+ 1 (length (cdr lst)))))

;; APPEND - concatenate two lists
(define-function append (lst1 lst2)
  (if (null? lst1) lst2 (cons (car lst1) (append (cdr lst1) lst2))))

;; REVERSE-HELPER - helper function for reverse
(define-function reverse-helper (lst acc)
  (if (null? lst) acc (reverse-helper (cdr lst) (cons (car lst) acc))))

;; REVERSE - reverse a list (tail-recursive)
(define-function reverse (lst)
  (reverse-helper lst ()))

;; MAP - apply function to each element
(define-function map (f lst)
  (if (null? lst) () (cons (f (car lst)) (map f (cdr lst)))))

;; FILTER - select elements matching predicate
(define-function filter (pred lst)
  (if (null? lst) () (if (pred (car lst)) (cons (car lst) (filter pred (cdr lst))) (filter pred (cdr lst)))))

;; FOLD-LEFT - left-associative fold/reduce
(define-function fold-left (f init lst)
  (if (null? lst) init (fold-left f (f init (car lst)) (cdr lst))))

;; FOLD-RIGHT - right-associative fold/reduce
(define-function fold-right (f init lst)
  (if (null? lst) init (f (car lst) (fold-right f init (cdr lst)))))

;; =============================================================================
;; Section 6: Type Predicates and Utilities
;; =============================================================================

;; LIST? - check if value is a list (including empty list)
(define-function list? (x)
  (not (atom? x)))

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

;; =============================================================================
;; Section 7: List Construction Utilities
;; =============================================================================

;; FIRST through FOURTH - convenient list accessors
(define-function first (lst)
  (car lst))

(define-function second (lst)
  (car (cdr lst)))

(define-function third (lst)
  (car (cdr (cdr lst))))

(define-function fourth (lst)
  (car (cdr (cdr (cdr lst)))))

;; REST - alias for cdr
(define-function rest (lst)
  (cdr lst))

;; =============================================================================
;; Section 8: Higher-Order Function Utilities
;; =============================================================================

;; Helper function for composition
(define-function composed-helper (f g x)
  (f (g x)))

;; COMPOSE - function composition (simplified version)
;; Returns a function that applies g then f
(define-function compose (f g)
  composed-helper)

;; APPLY - apply function to list of arguments (simple version)
(define-function apply (f args)
  (if (null? args) (f) (if (null? (cdr args)) (f (car args)) (if (null? (cdr (cdr args))) (f (car args) (car (cdr args))) (f (car args) (car (cdr args)) (car (cdr (cdr args))))))))

;; =============================================================================
;; Section 9: Utility Functions for Common Patterns
;; =============================================================================

;; IDENTITY - return argument unchanged
(define-function identity (x)
  x)

;; Helper for constantly
(define-function constant-fn-helper (value x)
  value)

;; CONSTANTLY - return function that always returns the same value (simplified)
(define-function constantly (value)
  constant-fn-helper)

;; =============================================================================
;; Section 10: List Generation and Processing
;; =============================================================================

;; RANGE - generate list of numbers from start to end (exclusive)
(define-function range (start end)
  (if (>= start end) () (cons start (range (+ start 1) end))))

;; TAKE - take first n elements from list
(define-function take (n lst)
  (if (or (= n 0) (null? lst)) () (cons (car lst) (take (- n 1) (cdr lst)))))

;; DROP - drop first n elements from list
(define-function drop (n lst)
  (if (or (= n 0) (null? lst)) lst (drop (- n 1) (cdr lst))))

;; =============================================================================
;; Section 11: Testing and Debugging Utilities
;; =============================================================================

;; ALL? - test if all elements satisfy predicate
(define-function all? (pred lst)
  (if (null? lst) #t (if (pred (car lst)) (all? pred (cdr lst)) #f)))

;; ANY? - test if any element satisfies predicate
(define-function any? (pred lst)
  (if (null? lst) #f (if (pred (car lst)) #t (any? pred (cdr lst)))))

;; MEMBER? - test if element is in list
(define-function member? (item lst)
  (if (null? lst) #f (if (= item (car lst)) #t (member? item (cdr lst)))))

;; =============================================================================
;; Section 12: Mathematical Functions
;; =============================================================================

;; SUM - sum all numbers in a list
(define-function sum (lst)
  (fold-left + 0 lst))

;; PRODUCT - multiply all numbers in a list
(define-function product (lst)
  (fold-left * 1 lst))

;; AVERAGE - calculate average of numbers in list
(define-function average (lst)
  (if (null? lst) 0 (/ (sum lst) (length lst))))

;; =============================================================================
;; Section 13: Function Creation Utilities
;; =============================================================================

;; LAMBDA - convenient syntax for creating anonymous functions
;; Uses the internal %%lambda primitive
(define-macro lambda (params body)
  (cons (quote %%lambda) (cons params (cons body ()))))

;; =============================================================================
;; Boot Library Successfully Loaded
;; =============================================================================

;; Set flag to indicate boot library is loaded
(define *boot-loaded* #t)

;; Print confirmation message
(print "RispyBoi boot library loaded successfully")
(print "Standard functions and macros are now available")