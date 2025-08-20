;; Test macro expansion manually
(print "Testing manual macro expansion")

;; Manually build what the lambda macro should produce
(define manual-expansion 
  (cons (quote %%lambda) 
        (cons (quote (x)) 
              (cons (quote (* x 3)) 
                    (quote ())))))

(print "Manual expansion created:")
(print manual-expansion)

;; Now apply it
(print "Evaluating manual expansion:")
(define manual-func (eval manual-expansion))
(print "Manual function created")