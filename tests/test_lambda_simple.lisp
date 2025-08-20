;; =============================================================================
;; Simple Lambda Macro Test Suite
;; =============================================================================
;; Basic tests for the lambda macro functionality

;; Test counter
(define *test-count* 0)

(define-function run-test (description test-expr expected)
  (set! *test-count* (+ *test-count* 1))
  (print description)
  (print test-expr)
  (print expected))

;; =============================================================================
;; Section 1: Basic Lambda Tests
;; =============================================================================

(print "=== Basic Lambda Tests ===")

;; Test 1: Simple lambda creation and execution
(define double-func (lambda (x) (* x 2)))
(run-test "Simple lambda creation" (double-func 5) 10)

;; Test 2: Lambda with two parameters
(define add-func (lambda (x y) (+ x y)))
(run-test "Lambda with two parameters" (add-func 3 7) 10)

;; Test 3: Lambda with zero parameters
(define get-answer (lambda () 42))
(run-test "Lambda with zero parameters" (get-answer) 42)

;; Test 4: Inline lambda usage
(run-test "Inline lambda" ((lambda (x) (* x x)) 6) 36)

;; =============================================================================
;; Section 2: Lambda with Conditionals
;; =============================================================================

(print "")
(print "=== Lambda with Conditionals ===")

;; Test 5: Lambda with if statement
(define abs-func (lambda (x) (if (< x 0) (- 0 x) x)))
(run-test "Lambda absolute value positive" (abs-func 5) 5)
(run-test "Lambda absolute value negative" (abs-func -3) 3)
(run-test "Lambda absolute value zero" (abs-func 0) 0)

;; Test 6: Lambda with boot library macros
(define max-func (lambda (x y) (if (> x y) x y)))
(run-test "Lambda max function" (max-func 8 3) 8)
(run-test "Lambda max function reverse" (max-func 2 9) 9)

;; =============================================================================
;; Section 3: Lambda with Higher-Order Functions
;; =============================================================================

(print "")
(print "=== Lambda with Higher-Order Functions ===")

;; Test 7: Lambda with map
(define square-list (map (lambda (x) (* x x)) (quote (1 2 3 4))))
(run-test "Lambda with map" square-list (quote (1 4 9 16)))

;; Test 8: Lambda with filter (using positive? from boot library)
(define positives (filter positive? (quote (-2 1 -3 4 5))))
(run-test "Filter positive numbers" positives (quote (1 4 5)))

;; Test 9: Lambda with fold-left
(define sum-squares (fold-left (lambda (acc x) (+ acc (* x x))) 0 (quote (1 2 3))))
(run-test "Lambda with fold-left" sum-squares 14)

;; =============================================================================
;; Section 4: Lambda Closures (Simple Cases)
;; =============================================================================

(print "")
(print "=== Lambda Closures ===")

;; Test 10: Lambda capturing external variable
(define multiplier 3)
(define triple-func (lambda (x) (* x multiplier)))
(run-test "Lambda closure capture" (triple-func 4) 12)

;; Test 11: Simple currying
(define make-adder-simple (lambda (x) (lambda (y) (+ x y))))
(define add-five (make-adder-simple 5))
(run-test "Simple currying" (add-five 7) 12)

;; =============================================================================
;; Section 5: Lambda Recursion
;; =============================================================================

(print "")
(print "=== Lambda Recursion ===")

;; Test 12: Recursive lambda (factorial)
(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))
(run-test "Recursive factorial 0" (factorial 0) 1)
(run-test "Recursive factorial 5" (factorial 5) 120)

;; Test 13: Recursive lambda (countdown)
(define countdown (lambda (n) (if (= n 0) 0 (+ n (countdown (- n 1))))))
(run-test "Recursive countdown" (countdown 5) 15)

;; =============================================================================
;; Section 6: Lambda Integration with Boot Library
;; =============================================================================

(print "")
(print "=== Lambda with Boot Library ===")

;; Test 14: Lambda using logical operators
(define is-positive (lambda (x) (and (> x 0) #t)))
(run-test "Lambda with logical and" (is-positive 5) #t)
(run-test "Lambda with logical and false" (is-positive -2) #f)

;; Test 15: Lambda using list functions
(define double-reverse (lambda (lst) (reverse (map (lambda (x) (* x 2)) lst))))
(run-test "Lambda with list functions" (double-reverse (quote (1 2 3))) (quote (6 4 2)))

;; Test 16: Lambda using mathematical functions
(define distance-from-origin (lambda (x y) (+ (square x) (square y))))
(run-test "Lambda with math functions" (distance-from-origin 3 4) 25)

;; =============================================================================
;; Section 7: Advanced Lambda Patterns
;; =============================================================================

(print "")
(print "=== Advanced Lambda Patterns ===")

;; Test 17: Lambda composition
(define add-one (lambda (x) (+ x 1)))
(define times-two (lambda (x) (* x 2)))
(define composed (lambda (x) (times-two (add-one x))))
(run-test "Lambda composition" (composed 5) 12)

;; Test 18: Lambda factory pattern
(define make-multiplier (lambda (factor) (lambda (x) (* x factor))))
(define times-three (make-multiplier 3))
(run-test "Lambda factory pattern" (times-three 7) 21)

;; Test 19: Lambda with complex expressions
(define complex-calc (lambda (a b c) (+ (* a b) (* c c)))
(run-test "Lambda complex calculation" (complex-calc 2 3 4) 22)

;; =============================================================================
;; Section 8: Edge Cases
;; =============================================================================

(print "")
(print "=== Lambda Edge Cases ===")

;; Test 20: Lambda returning constants
(define always-true (lambda (x) #t))
(run-test "Lambda always true" (always-true 123) #t)

;; Test 21: Lambda with same parameter names
(define nested-same-param (lambda (x) (+ x ((lambda (x) (* x 10)) 3))))
(run-test "Lambda with shadowed parameters" (nested-same-param 5) 35)

;; Test 22: Lambda in lambda (direct nesting)
(define nested-lambda (lambda (x) ((lambda (y) (+ x y)) 10)))
(run-test "Directly nested lambda" (nested-lambda 5) 15)

;; =============================================================================
;; Test Summary
;; =============================================================================

(print "")
(print "=== Lambda Test Suite Complete ===")
(print "Total tests executed:")
(print *test-count*)
(print "Review output above for test results")
(print "Expected vs actual values are shown for each test")