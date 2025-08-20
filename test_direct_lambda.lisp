;; Test parameter binding directly
(print "Testing direct %%lambda")

;; Create and call lambda immediately 
(print "Creating lambda...")
(define test-func (%%lambda (y) y))
(print "Lambda created, now calling...")
(print (test-func 42))