;; Simple test to verify basic functionality
(print "Testing basic functions")

;; Test AND macro
(print (and #t #f))
(print (and #t #t))

;; Test basic functions
(print (abs -5))
(print (length (quote (1 2 3))))

(print "Simple tests complete")