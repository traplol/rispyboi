;; Test lambda macro with correct arguments
(print "Testing lambda macro with correct arity")

;; This should work - two arguments to lambda macro
(define test-func (lambda (x) x))
(print "Lambda function created")
(print (test-func 42))