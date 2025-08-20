;; Basic lambda test
(print "Testing basic lambda functionality")

;; Test 1: Simple lambda creation
(define double (lambda (x) (* x 2)))
(print "Created double function")

;; Test 2: Execute lambda
(print "Testing double function:")
(print (double 5))

;; Test 3: Inline lambda
(print "Testing inline lambda:")
(print ((lambda (x) (* x x)) 6))

(print "Basic lambda tests complete")