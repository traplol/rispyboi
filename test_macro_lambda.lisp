;; Test lambda macro specifically
(print "Testing lambda macro")

;; Test macro expansion
(define triple-func (lambda (x) (* x 3)))
(print "Created triple function with macro")
(print (triple-func 4))