;; Debug lambda macro expansion
(print "Testing lambda macro expansion")

;; Test what the lambda macro expands to
(print "Testing simple lambda expansion:")

;; First test the %%lambda primitive directly
(print "Direct %%lambda test:")
(define test-direct (%%lambda (x) (* x 2)))
(print (test-direct 5))

;; Now test lambda macro
(print "Lambda macro test:")