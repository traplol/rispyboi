;; Test short-circuiting macros
(define-macro and (a b)
  (cons (quote if) (cons a (cons b (cons #f ())))))

;; Test short-circuiting with side effects
;; This should NOT print "second evaluated" because first arg is #f
(and #f (print "second evaluated"))

;; This should print "second evaluated" because first arg is #t
(and #t (print "second evaluated"))