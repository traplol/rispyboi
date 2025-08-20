;; Test the lambda macro
(print "Testing lambda macro")

;; Create a simple lambda function using the macro
(define double (lambda (x) (* x 2)))
(print double)

;; Test calling the lambda
(print (double 5))

;; Create a lambda with multiple parameters
(define multiply (lambda (x y) (* x y)))
(print (multiply 3 7))

;; Test lambda in higher-order function context
(print "Testing lambda with map:")
(print (map (lambda (x) (+ x 10)) (quote (1 2 3 4))))

;; Test inline lambda usage
(print "Testing inline lambda:")
(print ((lambda (x) (+ x 100)) 42))

(print "Lambda macro tests complete")