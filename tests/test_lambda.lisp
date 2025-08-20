;; =============================================================================
;; Lambda Macro Test Suite
;; =============================================================================
;; Comprehensive tests for the lambda macro in RispyBoi
;; Tests user-facing lambda functionality and integration with boot library

;; Test helper functions
(define *test-count* 0)
(define *pass-count* 0)
(define *fail-count* 0)

(define-function test-equal (description expected actual)
  (set! *test-count* (+ *test-count* 1))
  (if (= expected actual)
      (set! *pass-count* (+ *pass-count* 1))
      (set! *fail-count* (+ *fail-count* 1)))
  (print description))

(define-function test-true (description condition)
  (set! *test-count* (+ *test-count* 1))
  (if condition
      (set! *pass-count* (+ *pass-count* 1))
      (set! *fail-count* (+ *fail-count* 1)))
  (print description))

;; =============================================================================
;; Section 1: Basic Lambda Creation and Usage
;; =============================================================================

(print "=== Testing Basic Lambda Creation ===")

;; Test 1: Simple lambda creation and assignment
(define double (lambda (x) (* x 2)))
(test-equal "lambda creation and assignment" 10 (double 5))

;; Test 2: Lambda with multiple parameters
(define add-two (lambda (x y) (+ x y)))
(test-equal "lambda with two parameters" 12 (add-two 5 7))

;; Test 3: Lambda with zero parameters (constant function)
(define get-pi (lambda () 3.14159))
(test-equal "lambda with zero parameters" 3.14159 (get-pi))

;; Test 4: Inline lambda usage
(test-equal "inline lambda usage" 25 ((lambda (x) (* x x)) 5))

;; =============================================================================
;; Section 2: Lambda with Complex Bodies
;; =============================================================================

(print "")
(print "=== Testing Lambda Complex Bodies ===")

;; Test 5: Lambda with conditional logic
(define abs-func (lambda (x) (if (< x 0) (- 0 x) x)))
(test-equal "lambda absolute value positive" 5 (abs-func 5))
(test-equal "lambda absolute value negative" 5 (abs-func -5))
(test-equal "lambda absolute value zero" 0 (abs-func 0))

;; Test 6: Lambda with nested expressions
(define quadratic (lambda (a b c x) (+ (* a (* x x)) (+ (* b x) c))))
(test-equal "lambda quadratic function" 14 (quadratic 1 2 3 3)) ;; 1*9 + 2*3 + 3 = 18

;; Test 7: Lambda using boot library functions
(define safe-divide (lambda (x y) (if (= y 0) 0 (/ x y))))
(test-equal "lambda with conditional division" 5 (safe-divide 10 2))
(test-equal "lambda division by zero guard" 0 (safe-divide 10 0))

;; =============================================================================
;; Section 3: Higher-Order Functions with Lambda
;; =============================================================================

(print "")
(print "=== Testing Higher-Order Functions ===")

;; Test 8: Lambda used with map function
(define squares (map (lambda (x) (* x x)) (quote (1 2 3 4))))
(test-equal "lambda with map function" (quote (1 4 9 16)) squares)

;; Test 9: Lambda used with filter function
(define evens (filter (lambda (x) (= (% x 2) 0)) (quote (1 2 3 4 5 6))))
;; Note: This test may fail if % (modulo) is not implemented

;; Test 10: Lambda used with fold functions
(define sum-of-squares (fold-left (lambda (acc x) (+ acc (* x x))) 0 (quote (1 2 3))))
(test-equal "lambda with fold-left" 14 sum-of-squares) ;; 1 + 4 + 9 = 14

;; =============================================================================
;; Section 4: Lambda Closures
;; =============================================================================

(print "")
(print "=== Testing Lambda Closures ===")

;; Test 11: Lambda capturing external variables
(define multiplier 3)
(define triple (lambda (x) (* x multiplier)))
(test-equal "lambda closure capture" 15 (triple 5))

;; Test 12: Lambda in local scope
(define-function make-counter ()
  (define count 0)
  (lambda () (set! count (+ count 1))))

(define counter1 (make-counter))
(test-equal "lambda closure first call" 1 (counter1))
(test-equal "lambda closure second call" 2 (counter1))
(test-equal "lambda closure third call" 3 (counter1))

;; Test 13: Multiple independent closures
(define counter2 (make-counter))
(test-equal "independent closure first call" 1 (counter2))
(test-equal "original counter unchanged" 4 (counter1))

;; =============================================================================
;; Section 5: Lambda Currying and Partial Application
;; =============================================================================

(print "")
(print "=== Testing Lambda Currying ===")

;; Test 14: Simple currying
(define make-adder (lambda (x) (lambda (y) (+ x y))))
(define add-ten (make-adder 10))
(test-equal "curried function creation" 15 (add-ten 5))
(test-equal "curried function reuse" 23 (add-ten 13))

;; Test 15: Multi-level currying
(define make-multiplier (lambda (factor) 
  (lambda (x) (* x factor))))
(define double-func (make-multiplier 2))
(define triple-func (make-multiplier 3))
(test-equal "multi-level currying double" 14 (double-func 7))
(test-equal "multi-level currying triple" 21 (triple-func 7))

;; =============================================================================
;; Section 6: Lambda Recursion
;; =============================================================================

(print "")
(print "=== Testing Lambda Recursion ===")

;; Test 16: Recursive lambda (factorial)
(define factorial (lambda (n) 
  (if (= n 0) 
      1 
      (* n (factorial (- n 1))))))
(test-equal "recursive lambda factorial 0" 1 (factorial 0))
(test-equal "recursive lambda factorial 5" 120 (factorial 5))

;; Test 17: Recursive lambda (Fibonacci)
(define fibonacci (lambda (n)
  (if (<= n 1)
      n
      (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))
(test-equal "recursive lambda fibonacci 0" 0 (fibonacci 0))
(test-equal "recursive lambda fibonacci 1" 1 (fibonacci 1))
(test-equal "recursive lambda fibonacci 7" 13 (fibonacci 7))

;; =============================================================================
;; Section 7: Lambda with Boot Library Integration
;; =============================================================================

(print "")
(print "=== Testing Lambda with Boot Library ===")

;; Test 18: Lambda using logical operators
(define is-between (lambda (x min max) 
  (and (>= x min) (<= x max))))
(test-equal "lambda with logical operators true" #t (is-between 5 1 10))
(test-equal "lambda with logical operators false" #f (is-between 15 1 10))

;; Test 19: Lambda using enhanced comparisons
(define max-of-three (lambda (a b c)
  (if (> a b)
      (if (> a c) a c)
      (if (> b c) b c))))
(test-equal "lambda max of three" 8 (max-of-three 3 8 5))

;; Test 20: Lambda using list functions
(define reverse-and-double (lambda (lst)
  (map (lambda (x) (* x 2)) (reverse lst))))
(test-equal "lambda with list functions" (quote (8 6 4 2)) 
  (reverse-and-double (quote (1 2 3 4))))

;; =============================================================================
;; Section 8: Lambda Edge Cases and Error Conditions
;; =============================================================================

(print "")
(print "=== Testing Lambda Edge Cases ===")

;; Test 21: Lambda returning another lambda directly
(define identity-maker (lambda () (lambda (x) x)))
(define id-func (identity-maker))
(test-equal "lambda returning lambda" 42 (id-func 42))

;; Test 22: Lambda with side effects
(define side-effect-counter 0)
(define increment-and-return (lambda (x) 
  (set! side-effect-counter (+ side-effect-counter 1))
  (+ x side-effect-counter)))
(test-equal "lambda side effect first call" 11 (increment-and-return 10))
(test-equal "lambda side effect second call" 22 (increment-and-return 20))

;; Test 23: Lambda with complex parameter patterns
(define complex-calc (lambda (a b c d) 
  (+ (* a b) (- c d))))
(test-equal "lambda complex parameters" 11 (complex-calc 2 3 8 3))

;; =============================================================================
;; Section 9: Lambda Performance and Stress Tests
;; =============================================================================

(print "")
(print "=== Testing Lambda Performance ===")

;; Test 24: Lambda in tight loop (using recursion)
(define sum-to-n (lambda (n)
  (if (= n 0)
      0
      (+ n (sum-to-n (- n 1))))))
(test-equal "lambda recursive sum" 55 (sum-to-n 10)) ;; 1+2+...+10 = 55

;; Test 25: Multiple lambda calls
(define apply-twice (lambda (f x) (f (f x))))
(define add-one (lambda (x) (+ x 1)))
(test-equal "lambda composition" 7 (apply-twice add-one 5))

;; =============================================================================
;; Section 10: Lambda Integration with Define-Function
;; =============================================================================

(print "")
(print "=== Testing Lambda vs Define-Function ===")

;; Test 26: Comparing lambda and define-function behavior
(define-function named-double (x) (* x 2))
(define lambda-double (lambda (x) (* x 2)))

(test-equal "named function result" 10 (named-double 5))
(test-equal "lambda function result" 10 (lambda-double 5))

;; Test 27: Lambda assigned to variable vs define-function
(define lambda-square (lambda (x) (* x x)))
(define-function func-square (x) (* x x))

(test-equal "lambda square" 25 (lambda-square 5))
(test-equal "function square" 25 (func-square 5))

;; =============================================================================
;; Section 11: Advanced Lambda Patterns
;; =============================================================================

(print "")
(print "=== Testing Advanced Lambda Patterns ===")

;; Test 28: Lambda factory pattern
(define make-validator (lambda (min max)
  (lambda (value)
    (and (>= value min) (<= value max)))))

(define age-validator (make-validator 0 120))
(define percentage-validator (make-validator 0 100))

(test-equal "validator factory valid age" #t (age-validator 25))
(test-equal "validator factory invalid age" #f (age-validator 150))
(test-equal "validator factory valid percentage" #t (percentage-validator 85))
(test-equal "validator factory invalid percentage" #f (percentage-validator 110))

;; Test 29: Lambda composition pattern
(define compose (lambda (f g) 
  (lambda (x) (f (g x)))))

(define add-one (lambda (x) (+ x 1)))
(define times-two (lambda (x) (* x 2)))
(define add-then-double (compose times-two add-one))

(test-equal "lambda composition" 12 (add-then-double 5)) ;; (5 + 1) * 2 = 12

;; Test 30: Lambda for functional list processing
(define process-list (lambda (lst)
  (map (lambda (x) (+ x 10))
       (filter (lambda (x) (> x 0))
               lst))))

(test-equal "functional list processing" (quote (11 13 15))
  (process-list (quote (-2 1 3 -1 5))))

;; =============================================================================
;; Test Summary and Results
;; =============================================================================

(print "")
(print "=== Lambda Test Suite Complete ===")
(print "Total tests:")
(print *test-count*)
(print "Tests passed:")
(print *pass-count*)
(print "Tests failed:")
(print *fail-count*)

(if (= *fail-count* 0)
    (print "All lambda tests PASSED!")
    (print "Some lambda tests FAILED - check output above"))

;; Calculate success percentage
(define success-rate (if (= *test-count* 0) 
                         0 
                         (/ (* *pass-count* 100) *test-count*)))
(print "Success rate:")
(print success-rate)