;; Debug lambda creation
(print "1. Testing lambda macro expansion")

;; Test expansion manually
(print "2. Creating lambda manually with %%lambda")
(define manual-double (%%lambda (x) (* x 2)))
(print "3. Manual lambda created")

;; Test with macro  
(print "4. Creating lambda with macro")
(define macro-double (lambda (x) (* x 2)))
(print "5. Macro lambda created")

(print "6. Testing manual lambda call")
(print (manual-double 5))

(print "7. Testing macro lambda call")  
(print (macro-double 5))