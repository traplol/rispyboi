;; Test what macro arguments look like
(print "Testing macro argument evaluation")

;; Define a macro that just prints its arguments
(define-macro debug-macro (a b) 
  (print "Macro arg a:")
  (print a)
  (print "Macro arg b:")
  (print b)
  (quote result))

;; Use it
(print "Calling debug macro:")
(debug-macro (x) (* x 2))