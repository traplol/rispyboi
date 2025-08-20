;; Test lambda with math operations
(print "Testing lambda with math operations")

;; Test lambda with multiplication
(define double-func (%%lambda (x) (* x 2)))
(print "Created double function")
(print (double-func 5))