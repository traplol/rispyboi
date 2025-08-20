;; Test if macro args are evaluated
(print "Testing simple macro passthrough")

;; Define a macro that just returns its first argument literally
(define-macro passthrough (arg) arg)

;; Test with a simple symbol
(print "Simple symbol test:")
(print (passthrough x))

;; Test with a list that would error if evaluated
(print "List test:")
(print (passthrough (unknown-function arg1 arg2)))