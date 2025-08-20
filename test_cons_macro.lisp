;; Test lambda-style macro with cons
(print "Testing cons-based macro")

;; Define a macro that uses cons like lambda
(define-macro my-lambda (params body)
  (cons (quote %%lambda) (cons params (cons body ()))))

;; Use the macro
(print "Using my-lambda macro:")
(define my-func (my-lambda (x) (* x 4)))
(print "Function created")
(print (my-func 6))