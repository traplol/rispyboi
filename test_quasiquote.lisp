;; Test basic quasiquote syntax
(quasiquote (+ 1 2))
(quasiquote (+ (unquote (+ 1 2)) 3))