;; Test what macro arguments look like
(print "Testing macro argument evaluation")

;; Define a macro that just returns its first argument
(define-macro debug-macro (a b) a)

;; Use it
(print "Calling debug macro:")
(print (debug-macro (x) (* x 2)))