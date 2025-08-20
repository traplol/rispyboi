;; Test simple macro functionality
(print "Testing simple macro")

;; Define a simple macro to test basic macro expansion
(define-macro test-macro (x) (+ x 1))

;; Use the macro
(print "Using test macro:")
(print (test-macro 5))