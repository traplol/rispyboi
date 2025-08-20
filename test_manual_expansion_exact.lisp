;; Test exact manual macro expansion
(print "Testing exact manual expansion")

;; What the lambda macro should produce
(define manual-result (cons (quote %%lambda) (cons (quote (x)) (cons (quote (* x 3)) (quote ())))))
(print "Manual result:")
(print manual-result)

;; Define function using manual result  
(print "Testing manual function:")
(define manual-func (%%lambda (x) (* x 3)))
(print "Manual func created")
(print (manual-func 7))