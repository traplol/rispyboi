;; Test short-circuiting with side effects
;; This should print "first evaluated" but NOT "second evaluated" 
(and #f (print "second evaluated"))

;; This should print "first evaluated" and "second evaluated"
(and #t (print "second evaluated"))

;; Test OR short-circuiting
;; This should print "first evaluated" but NOT "second evaluated"
(or #t (print "second evaluated in or"))

;; This should print "first evaluated" and "second evaluated"  
(or #f (print "second evaluated in or"))