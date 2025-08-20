;; =============================================================================
;; RispyBoi Boot Library Test Suite
;; =============================================================================
;; Comprehensive tests for all functions and macros in boot.lisp
;; Tests verify correctness, edge cases, and proper behavior

;; Test helper functions
(define *test-count* 0)
(define *pass-count* 0)
(define *fail-count* 0)

(define-function print-test-result (status description)
  (set! *test-count* (+ *test-count* 1))
  (if (= status #t)
      (set! *pass-count* (+ *pass-count* 1))
      (set! *fail-count* (+ *fail-count* 1)))
  (print description))

(define-function assert-equal (expected actual description)
  (print-test-result (= expected actual) description))

(define-function assert-true (condition description)
  (print-test-result condition description))

(define-function assert-false (condition description)
  (print-test-result (not condition) description))

;; =============================================================================
;; Section 1: Logical Operators Tests
;; =============================================================================

(print "")
(print "=== Testing Logical Operators ===")

;; AND macro tests
(assert-equal #f (and #f #f) "and #f #f")
(assert-equal #f (and #f #t) "and #f #t") 
(assert-equal #f (and #t #f) "and #t #f")
(assert-equal #t (and #t #t) "and #t #t")

;; Test short-circuiting of AND - second expression should not be evaluated when first is false
(define *side-effect-counter* 0)
(define-function increment-counter ()
  (set! *side-effect-counter* (+ *side-effect-counter* 1))
  #t)

(set! *side-effect-counter* 0)
(and #f (increment-counter))
(assert-equal 0 *side-effect-counter* "and short-circuits on false first argument")

(set! *side-effect-counter* 0)
(and #t (increment-counter))
(assert-equal 1 *side-effect-counter* "and evaluates second argument when first is true")

;; OR macro tests
(assert-equal #f (or #f #f) "or #f #f")
(assert-equal #t (or #f #t) "or #f #t")
(assert-equal #t (or #t #f) "or #t #f") 
(assert-equal #t (or #t #t) "or #t #t")

;; Test short-circuiting of OR - second expression should not be evaluated when first is true
(set! *side-effect-counter* 0)
(or #t (increment-counter))
(assert-equal 0 *side-effect-counter* "or short-circuits on true first argument")

(set! *side-effect-counter* 0)
(or #f (increment-counter))
(assert-equal 1 *side-effect-counter* "or evaluates second argument when first is false")

;; NOT macro tests
(assert-equal #t (not #f) "not #f")
(assert-equal #f (not #t) "not #t")
(assert-equal #f (not 42) "not 42 (truthy value)")
(assert-equal #t (not ()) "not () (nil)")

;; =============================================================================
;; Section 2: Enhanced Comparison Operators Tests
;; =============================================================================

(print "")
(print "=== Testing Comparison Operators ===")

;; Greater than tests
(assert-true (> 5 3) "> 5 3")
(assert-false (> 3 5) "> 3 5")
(assert-false (> 3 3) "> 3 3")

;; Less than or equal tests
(assert-true (<= 3 5) "<= 3 5")
(assert-true (<= 3 3) "<= 3 3")
(assert-false (<= 5 3) "<= 5 3")

;; Greater than or equal tests
(assert-true (>= 5 3) ">= 5 3")
(assert-true (>= 3 3) ">= 3 3")
(assert-false (>= 3 5) ">= 3 5")

;; Not equal tests
(assert-true (!= 3 5) "!= 3 5")
(assert-false (!= 3 3) "!= 3 3")
(assert-true (!= #t #f) "!= #t #f")

;; =============================================================================
;; Section 3: Control Flow Macros Tests
;; =============================================================================

(print "")
(print "=== Testing Control Flow Macros ===")

;; WHEN macro tests
(define *when-result* 0)
(when #t (set! *when-result* 42))
(assert-equal 42 *when-result* "when with true condition executes body")

(set! *when-result* 0)
(when #f (set! *when-result* 42))
(assert-equal 0 *when-result* "when with false condition does not execute body")

;; UNLESS macro tests
(define *unless-result* 0)
(unless #f (set! *unless-result* 42))
(assert-equal 42 *unless-result* "unless with false condition executes body")

(set! *unless-result* 0)
(unless #t (set! *unless-result* 42))
(assert-equal 0 *unless-result* "unless with true condition does not execute body")

;; =============================================================================
;; Section 4: Mathematical Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing Mathematical Utilities ===")

;; ABS function tests
(assert-equal 5 (abs 5) "abs of positive number")
(assert-equal 5 (abs -5) "abs of negative number")
(assert-equal 0 (abs 0) "abs of zero")

;; MIN function tests
(assert-equal 3 (min 3 5) "min 3 5")
(assert-equal 3 (min 5 3) "min 5 3")
(assert-equal 3 (min 3 3) "min 3 3")

;; MAX function tests
(assert-equal 5 (max 3 5) "max 3 5")
(assert-equal 5 (max 5 3) "max 5 3")
(assert-equal 3 (max 3 3) "max 3 3")

;; SQUARE function tests
(assert-equal 25 (square 5) "square 5")
(assert-equal 16 (square 4) "square 4")
(assert-equal 0 (square 0) "square 0")

;; =============================================================================
;; Section 5: List Processing Functions Tests
;; =============================================================================

(print "")
(print "=== Testing List Processing Functions ===")

;; LENGTH function tests
(assert-equal 0 (length ()) "length of empty list")
(assert-equal 1 (length (quote (a))) "length of single element list")
(assert-equal 3 (length (quote (a b c))) "length of three element list")
(assert-equal 4 (length (quote (1 2 3 4))) "length of number list")

;; APPEND function tests
(assert-equal (quote (1 2 3 4)) (append (quote (1 2)) (quote (3 4))) "append two lists")
(assert-equal (quote (1 2)) (append (quote (1 2)) ()) "append with empty list")
(assert-equal (quote (3 4)) (append () (quote (3 4))) "append empty list with list")
(assert-equal () (append () ()) "append two empty lists")

;; REVERSE function tests
(assert-equal () (reverse ()) "reverse empty list")
(assert-equal (quote (a)) (reverse (quote (a))) "reverse single element")
(assert-equal (quote (3 2 1)) (reverse (quote (1 2 3))) "reverse three elements")
(assert-equal (quote (d c b a)) (reverse (quote (a b c d))) "reverse four elements")

;; MAP function tests (using square as test function)
(assert-equal () (map square ()) "map over empty list")
(assert-equal (quote (25)) (map square (quote (5))) "map over single element")
(assert-equal (quote (1 4 9 16)) (map square (quote (1 2 3 4))) "map square over list")

;; FILTER function tests (using positive? as test function)
(assert-equal () (filter positive? ()) "filter empty list")
(assert-equal (quote (1 2 3)) (filter positive? (quote (1 2 3))) "filter all positive")
(assert-equal () (filter positive? (quote (-1 -2 -3))) "filter all negative")
(assert-equal (quote (1 3)) (filter positive? (quote (-1 1 -2 3))) "filter mixed signs")

;; FOLD-LEFT function tests
(assert-equal 0 (fold-left + 0 ()) "fold-left + over empty list")
(assert-equal 10 (fold-left + 0 (quote (1 2 3 4))) "fold-left + sum")
(assert-equal 24 (fold-left * 1 (quote (1 2 3 4))) "fold-left * product")

;; FOLD-RIGHT function tests
(assert-equal 0 (fold-right + 0 ()) "fold-right + over empty list")
(assert-equal 10 (fold-right + 0 (quote (1 2 3 4))) "fold-right + sum")
(assert-equal 24 (fold-right * 1 (quote (1 2 3 4))) "fold-right * product")

;; =============================================================================
;; Section 6: Type Predicates Tests
;; =============================================================================

(print "")
(print "=== Testing Type Predicates ===")

;; LIST? function tests
(assert-true (list? ()) "empty list is a list")
(assert-true (list? (quote (1 2 3))) "non-empty list is a list")
(assert-false (list? 42) "number is not a list")
(assert-false (list? #t) "boolean is not a list")

;; PAIR? function tests
(assert-false (pair? ()) "empty list is not a pair")
(assert-true (pair? (quote (1))) "single element list is a pair")
(assert-true (pair? (quote (1 2 3))) "multi-element list is a pair")
(assert-false (pair? 42) "number is not a pair")

;; ZERO? function tests
(assert-true (zero? 0) "zero is zero")
(assert-false (zero? 1) "one is not zero")
(assert-false (zero? -1) "negative one is not zero")

;; POSITIVE? function tests
(assert-true (positive? 1) "one is positive")
(assert-true (positive? 42) "42 is positive")
(assert-false (positive? 0) "zero is not positive")
(assert-false (positive? -1) "negative one is not positive")

;; NEGATIVE? function tests
(assert-true (negative? -1) "negative one is negative")
(assert-true (negative? -42) "negative 42 is negative")
(assert-false (negative? 0) "zero is not negative")
(assert-false (negative? 1) "one is not negative")

;; =============================================================================
;; Section 7: List Construction Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing List Construction Utilities ===")

;; FIRST, SECOND, THIRD, FOURTH function tests
(define test-list (quote (a b c d e)))
(assert-equal (quote a) (first test-list) "first element")
(assert-equal (quote b) (second test-list) "second element")
(assert-equal (quote c) (third test-list) "third element")
(assert-equal (quote d) (fourth test-list) "fourth element")

;; REST function tests
(assert-equal (quote (b c d e)) (rest test-list) "rest of list")
(assert-equal () (rest (quote (a))) "rest of single element list")

;; =============================================================================
;; Section 8: Higher-Order Function Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing Higher-Order Function Utilities ===")

;; COMPOSE function tests (simplified version)
;; Note: The current compose implementation is simplified and may not work as expected
;; We'll test what it currently does

;; APPLY function tests (limited implementation)
(assert-equal 6 (apply + (quote (2 4))) "apply + to two arguments")
(assert-equal 24 (apply * (quote (3 8))) "apply * to two arguments")

;; =============================================================================
;; Section 9: Utility Functions Tests
;; =============================================================================

(print "")
(print "=== Testing Utility Functions ===")

;; IDENTITY function tests
(assert-equal 42 (identity 42) "identity of number")
(assert-equal #t (identity #t) "identity of boolean")
(assert-equal () (identity ()) "identity of empty list")
(assert-equal (quote (1 2 3)) (identity (quote (1 2 3))) "identity of list")

;; =============================================================================
;; Section 10: List Generation and Processing Tests
;; =============================================================================

(print "")
(print "=== Testing List Generation and Processing ===")

;; RANGE function tests
(assert-equal () (range 5 5) "range with equal start and end")
(assert-equal (quote (0 1 2 3 4)) (range 0 5) "range 0 to 5")
(assert-equal (quote (2 3 4)) (range 2 5) "range 2 to 5")
(assert-equal () (range 5 3) "range with start > end")

;; TAKE function tests
(assert-equal () (take 0 (quote (1 2 3 4))) "take 0 elements")
(assert-equal (quote (1 2)) (take 2 (quote (1 2 3 4))) "take 2 elements")
(assert-equal (quote (1 2 3 4)) (take 5 (quote (1 2 3 4))) "take more than available")
(assert-equal () (take 3 ()) "take from empty list")

;; DROP function tests
(assert-equal (quote (1 2 3 4)) (drop 0 (quote (1 2 3 4))) "drop 0 elements")
(assert-equal (quote (3 4)) (drop 2 (quote (1 2 3 4))) "drop 2 elements")
(assert-equal () (drop 5 (quote (1 2 3 4))) "drop more than available")
(assert-equal () (drop 3 ()) "drop from empty list")

;; =============================================================================
;; Section 11: Testing and Debugging Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing Testing and Debugging Utilities ===")

;; ALL? function tests
(assert-true (all? positive? (quote (1 2 3 4))) "all positive numbers")
(assert-false (all? positive? (quote (1 -2 3 4))) "not all positive")
(assert-true (all? positive? ()) "all? on empty list")

;; ANY? function tests
(assert-true (any? positive? (quote (1 -2 3 -4))) "some positive numbers")
(assert-false (any? positive? (quote (-1 -2 -3 -4))) "no positive numbers")
(assert-false (any? positive? ()) "any? on empty list")

;; MEMBER? function tests
(assert-true (member? 2 (quote (1 2 3 4))) "member 2 in list")
(assert-false (member? 5 (quote (1 2 3 4))) "member 5 not in list")
(assert-false (member? 1 ()) "member in empty list")

;; =============================================================================
;; Section 12: Mathematical Functions Tests
;; =============================================================================

(print "")
(print "=== Testing Mathematical Functions ===")

;; SUM function tests
(assert-equal 0 (sum ()) "sum of empty list")
(assert-equal 10 (sum (quote (1 2 3 4))) "sum of numbers")
(assert-equal -2 (sum (quote (1 -3 2))) "sum with negative numbers")

;; PRODUCT function tests
(assert-equal 1 (product ()) "product of empty list")
(assert-equal 24 (product (quote (1 2 3 4))) "product of numbers")
(assert-equal -6 (product (quote (1 -2 3))) "product with negative number")

;; AVERAGE function tests
(assert-equal 0 (average ()) "average of empty list")
(assert-equal 2.5 (average (quote (1 2 3 4))) "average of numbers")

;; =============================================================================
;; Test Summary
;; =============================================================================

(print "")
(print "=== Boot Library Test Suite Complete ===")
(print "Total tests run:")
(print *test-count*)
(print "Tests passed:")
(print *pass-count*)
(print "Tests failed:")
(print *fail-count*)