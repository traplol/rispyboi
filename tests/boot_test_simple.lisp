;; =============================================================================
;; RispyBoi Boot Library Test Suite (Simplified)
;; =============================================================================
;; Tests for all functions and macros in boot.lisp
;; This version simply runs the functions and prints results for manual verification

(print "=== RispyBoi Boot Library Test Suite ===")
(print "")

;; =============================================================================
;; Section 1: Logical Operators Tests
;; =============================================================================

(print "=== Testing Logical Operators ===")

(print "Testing AND macro:")
(print (and #f #f))  ;; Should be #f
(print (and #f #t))  ;; Should be #f  
(print (and #t #f))  ;; Should be #f
(print (and #t #t))  ;; Should be #t

(print "Testing OR macro:")
(print (or #f #f))   ;; Should be #f
(print (or #f #t))   ;; Should be #t
(print (or #t #f))   ;; Should be #t
(print (or #t #t))   ;; Should be #t

(print "Testing NOT macro:")
(print (not #f))     ;; Should be #t
(print (not #t))     ;; Should be #f
(print (not 42))     ;; Should be #f (truthy)
(print (not ()))     ;; Should be #t (nil)

;; =============================================================================
;; Section 2: Enhanced Comparison Operators Tests
;; =============================================================================

(print "")
(print "=== Testing Comparison Operators ===")

(print "Testing > macro:")
(print (> 5 3))      ;; Should be #t
(print (> 3 5))      ;; Should be #f
(print (> 3 3))      ;; Should be #f

(print "Testing <= macro:")
(print (<= 3 5))     ;; Should be #t
(print (<= 3 3))     ;; Should be #t
(print (<= 5 3))     ;; Should be #f

(print "Testing >= macro:")
(print (>= 5 3))     ;; Should be #t
(print (>= 3 3))     ;; Should be #t
(print (>= 3 5))     ;; Should be #f

(print "Testing != macro:")
(print (!= 3 5))     ;; Should be #t
(print (!= 3 3))     ;; Should be #f

;; =============================================================================
;; Section 3: Mathematical Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing Mathematical Utilities ===")

(print "Testing ABS function:")
(print (abs 5))      ;; Should be 5
(print (abs -5))     ;; Should be 5
(print (abs 0))      ;; Should be 0

(print "Testing MIN function:")
(print (min 3 5))    ;; Should be 3
(print (min 5 3))    ;; Should be 3
(print (min 3 3))    ;; Should be 3

(print "Testing MAX function:")
(print (max 3 5))    ;; Should be 5
(print (max 5 3))    ;; Should be 5
(print (max 3 3))    ;; Should be 3

(print "Testing SQUARE function:")
(print (square 5))   ;; Should be 25
(print (square 4))   ;; Should be 16
(print (square 0))   ;; Should be 0

;; =============================================================================
;; Section 4: List Processing Functions Tests
;; =============================================================================

(print "")
(print "=== Testing List Processing Functions ===")

(print "Testing LENGTH function:")
(print (length ()))                    ;; Should be 0
(print (length (quote (a))))           ;; Should be 1
(print (length (quote (a b c))))       ;; Should be 3
(print (length (quote (1 2 3 4))))     ;; Should be 4

(print "Testing APPEND function:")
(print (append (quote (1 2)) (quote (3 4))))  ;; Should be (1 2 3 4)
(print (append (quote (1 2)) ()))             ;; Should be (1 2)
(print (append () (quote (3 4))))             ;; Should be (3 4)
(print (append () ()))                        ;; Should be ()

(print "Testing REVERSE function:")
(print (reverse ()))                   ;; Should be ()
(print (reverse (quote (a))))          ;; Should be (a)
(print (reverse (quote (1 2 3))))      ;; Should be (3 2 1)
(print (reverse (quote (a b c d))))    ;; Should be (d c b a)

;; =============================================================================
;; Section 5: Type Predicates Tests
;; =============================================================================

(print "")
(print "=== Testing Type Predicates ===")

(print "Testing LIST? function:")
(print (list? ()))                     ;; Should be #t
(print (list? (quote (1 2 3))))        ;; Should be #t
(print (list? 42))                     ;; Should be #f
(print (list? #t))                     ;; Should be #f

(print "Testing PAIR? function:")
(print (pair? ()))                     ;; Should be #f
(print (pair? (quote (1))))            ;; Should be #t
(print (pair? (quote (1 2 3))))        ;; Should be #t
(print (pair? 42))                     ;; Should be #f

(print "Testing ZERO? function:")
(print (zero? 0))                      ;; Should be #t
(print (zero? 1))                      ;; Should be #f
(print (zero? -1))                     ;; Should be #f

(print "Testing POSITIVE? function:")
(print (positive? 1))                  ;; Should be #t
(print (positive? 42))                 ;; Should be #t
(print (positive? 0))                  ;; Should be #f
(print (positive? -1))                 ;; Should be #f

(print "Testing NEGATIVE? function:")
(print (negative? -1))                 ;; Should be #t
(print (negative? -42))                ;; Should be #t
(print (negative? 0))                  ;; Should be #f
(print (negative? 1))                  ;; Should be #f

;; =============================================================================
;; Section 6: List Construction Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing List Construction Utilities ===")

(define test-list (quote (a b c d e)))
(print "Testing FIRST, SECOND, THIRD, FOURTH:")
(print (first test-list))              ;; Should be a
(print (second test-list))             ;; Should be b
(print (third test-list))              ;; Should be c
(print (fourth test-list))             ;; Should be d

(print "Testing REST function:")
(print (rest test-list))               ;; Should be (b c d e)
(print (rest (quote (a))))             ;; Should be ()

;; =============================================================================
;; Section 7: Mathematical Functions Tests
;; =============================================================================

(print "")
(print "=== Testing Mathematical Functions ===")

(print "Testing SUM function:")
(print (sum ()))                       ;; Should be 0
(print (sum (quote (1 2 3 4))))        ;; Should be 10
(print (sum (quote (1 -3 2))))         ;; Should be 0

(print "Testing PRODUCT function:")
(print (product ()))                   ;; Should be 1
(print (product (quote (1 2 3 4))))    ;; Should be 24
(print (product (quote (1 -2 3))))     ;; Should be -6

(print "Testing AVERAGE function:")
(print (average ()))                   ;; Should be 0
(print (average (quote (1 2 3 4))))    ;; Should be 2.5

;; =============================================================================
;; Section 8: Testing and Debugging Utilities Tests
;; =============================================================================

(print "")
(print "=== Testing Testing and Debugging Utilities ===")

(print "Testing ALL? function:")
(print (all? positive? (quote (1 2 3 4))))     ;; Should be #t
(print (all? positive? (quote (1 -2 3 4))))    ;; Should be #f
(print (all? positive? ()))                    ;; Should be #t

(print "Testing ANY? function:")
(print (any? positive? (quote (1 -2 3 -4))))   ;; Should be #t
(print (any? positive? (quote (-1 -2 -3 -4)))) ;; Should be #f
(print (any? positive? ()))                    ;; Should be #f

(print "Testing MEMBER? function:")
(print (member? 2 (quote (1 2 3 4))))          ;; Should be #t
(print (member? 5 (quote (1 2 3 4))))          ;; Should be #f
(print (member? 1 ()))                         ;; Should be #f

;; =============================================================================
;; Test Complete
;; =============================================================================

(print "")
(print "=== Boot Library Test Suite Complete ===")
(print "Review the output above to verify correctness")
(print "Expected values are shown in comments")