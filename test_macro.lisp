;; Test individual macro definition
(define-macro and (a b)
  (cons (quote if) (cons a (cons b (cons #f ())))))