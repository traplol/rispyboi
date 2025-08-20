;; Test the %%lambda primitive
(print "Testing %%lambda primitive")

;; Create a simple lambda function
(define add-one (%%lambda (x) (+ x 1)))
(print add-one)

;; Test calling the lambda
(print (add-one 5))

;; Create a lambda with multiple parameters
(define add-two-numbers (%%lambda (x y) (+ x y)))
(print (add-two-numbers 3 7))

(print "%%lambda tests complete")