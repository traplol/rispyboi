;; boot.lisp - RispyBoi Standard Library
;; Automatically loaded at interpreter startup
;; Provides essential utility macros and functions

;; =============================================================================
;; Section 1: Logical Operators
;; =============================================================================

;; AND - short-circuiting logical and
(define-macro and (a b)
  (cons (quote if) (cons a (cons b (cons #f ())))))

;; OR - short-circuiting logical or  
(define-macro or (a b)
  (cons (quote if) (cons a (cons #t (cons b ())))))

;; NOT - logical negation
(define-macro not (a)
  (cons (quote if) (cons a (cons #f (cons #t ())))))