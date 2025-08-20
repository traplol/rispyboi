;; Test what lambda macro expands to
(print "Testing lambda macro expansion")

;; Test creating a simple macro that shows expansion
(define-macro show-expansion (expr)
  (cons (quote print) (cons expr ())))

;; Show what lambda expands to
(define-macro test-lambda (params body)
  (cons (quote print) (cons (cons (quote %%lambda) (cons params (cons body ()))) ())))

(print "Testing expansion:")
(test-lambda (x) (* x 2))