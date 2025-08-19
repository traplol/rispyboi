use crate::error::RispyError;
use crate::lexer::Token;
use crate::value::Value;

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Value>, RispyError> {
    let mut parser = Parser::new(tokens);
    parser.parse_expressions()
}

pub fn parse_single(source: &str) -> Result<Value, RispyError> {
    let tokens = crate::lexer::tokenize(source)?;
    let expressions = parse(tokens)?;

    if expressions.len() != 1 {
        return Err(RispyError::ParseError(
            "Expected single expression".to_string(),
        ));
    }

    Ok(expressions.into_iter().next().unwrap())
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn parse_expressions(&mut self) -> Result<Vec<Value>, RispyError> {
        let mut expressions = Vec::new();

        while !self.is_at_end() {
            expressions.push(self.parse_expression()?);
        }

        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<Value, RispyError> {
        match self.peek() {
            Some(Token::LeftParen) => self.parse_list(),
            Some(Token::Quote) => self.parse_quoted(),
            Some(_) => self.parse_atom(),
            None => Err(RispyError::ParseError(
                "Unexpected end of input".to_string(),
            )),
        }
    }

    fn parse_list(&mut self) -> Result<Value, RispyError> {
        self.consume_left_paren()?;

        let mut elements = Vec::new();

        while !self.check_right_paren() && !self.is_at_end() {
            elements.push(self.parse_expression()?);
        }

        if self.is_at_end() {
            return Err(RispyError::ParseError(
                "Expected ')' after list elements".to_string(),
            ));
        }

        self.consume_right_paren()?;

        Ok(Value::List(elements))
    }

    fn parse_quoted(&mut self) -> Result<Value, RispyError> {
        self.advance();

        let expr = self.parse_expression()?;

        Ok(Value::List(vec![Value::Symbol("quote".to_string()), expr]))
    }

    fn parse_atom(&mut self) -> Result<Value, RispyError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(Value::Number(*n)),
            Some(Token::String(s)) => Ok(Value::String(s.clone())),
            Some(Token::Symbol(s)) => match s.as_str() {
                "nil" => Ok(Value::Nil),
                "#t" | "true" => Ok(Value::Bool(true)),
                "#f" | "false" => Ok(Value::Bool(false)),
                _ => Ok(Value::Symbol(s.clone())),
            },
            Some(Token::RightParen) => Err(RispyError::ParseError(
                "Unexpected ')' - no matching '('".to_string(),
            )),
            Some(unexpected) => Err(RispyError::ParseError(format!(
                "Unexpected token: {:?}",
                unexpected
            ))),
            None => Err(RispyError::ParseError(
                "Unexpected end of input".to_string(),
            )),
        }
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn previous(&self) -> Option<&Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1)
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn check_right_paren(&self) -> bool {
        matches!(self.peek(), Some(Token::RightParen))
    }

    fn consume_left_paren(&mut self) -> Result<(), RispyError> {
        match self.advance() {
            Some(Token::LeftParen) => Ok(()),
            Some(unexpected) => Err(RispyError::ParseError(format!(
                "Expected '(', found {:?}",
                unexpected
            ))),
            None => Err(RispyError::ParseError(
                "Expected '(', found end of input".to_string(),
            )),
        }
    }

    fn consume_right_paren(&mut self) -> Result<(), RispyError> {
        match self.advance() {
            Some(Token::RightParen) => Ok(()),
            Some(unexpected) => Err(RispyError::ParseError(format!(
                "Expected ')', found {:?}",
                unexpected
            ))),
            None => Err(RispyError::ParseError(
                "Expected ')', found end of input".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    fn parse_test(source: &str) -> Result<Value, RispyError> {
        parse_single(source)
    }

    #[test]
    fn test_parse_atoms() {
        assert_eq!(parse_test("42").unwrap(), Value::Number(42.0));
        assert_eq!(parse_test("-3.14").unwrap(), Value::Number(-3.14));
        assert_eq!(
            parse_test("\"hello\"").unwrap(),
            Value::String("hello".to_string())
        );
        assert_eq!(parse_test("foo").unwrap(), Value::Symbol("foo".to_string()));
        assert_eq!(parse_test("nil").unwrap(), Value::Nil);
        assert_eq!(parse_test("#t").unwrap(), Value::Bool(true));
        assert_eq!(parse_test("#f").unwrap(), Value::Bool(false));
        assert_eq!(parse_test("true").unwrap(), Value::Bool(true));
        assert_eq!(parse_test("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_parse_empty_list() {
        assert_eq!(parse_test("()").unwrap(), Value::List(vec![]));
    }

    #[test]
    fn test_parse_simple_lists() {
        let result = parse_test("(1 2 3)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );

        let result2 = parse_test("(+ 1 2)").unwrap();
        assert_eq!(
            result2,
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );
    }

    #[test]
    fn test_parse_nested_lists() {
        let result = parse_test("(+ (* 2 3) 4)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::List(vec![
                    Value::Symbol("*".to_string()),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ]),
                Value::Number(4.0)
            ])
        );
    }

    #[test]
    fn test_parse_quote() {
        let result = parse_test("'x").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::Symbol("x".to_string())
            ])
        );

        let result2 = parse_test("'(1 2 3)").unwrap();
        assert_eq!(
            result2,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0)
                ])
            ])
        );
    }

    #[test]
    fn test_parse_nested_quotes() {
        let result = parse_test("''x").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![
                    Value::Symbol("quote".to_string()),
                    Value::Symbol("x".to_string())
                ])
            ])
        );
    }

    #[test]
    fn test_parse_multiple_expressions() {
        let tokens = tokenize("(+ 1 2) (* 3 4)").unwrap();
        let result = parse(tokens).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );
        assert_eq!(
            result[1],
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Number(3.0),
                Value::Number(4.0)
            ])
        );
    }

    #[test]
    fn test_parse_errors() {
        assert!(parse_test("(").is_err());
        assert!(parse_test(")").is_err());
        assert!(parse_test("(1 2").is_err());
        assert!(parse_test("1 2)").is_err());
        assert!(parse_test("((1 2)").is_err());

        // Test quote without expression
        let tokens = vec![Token::Quote];
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn test_parse_error_messages() {
        match parse_test("(") {
            Err(RispyError::ParseError(msg)) => {
                assert!(msg.contains("Expected ')'"));
            }
            _ => panic!("Expected ParseError"),
        }

        match parse_test(")") {
            Err(RispyError::ParseError(msg)) => {
                assert!(msg.contains("Unexpected ')'"));
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_complex_nested_structure() {
        let source = "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))";
        let result = parse_test(source).unwrap();

        // Just verify it parses without error and has correct top-level structure
        if let Value::List(list) = result {
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::Symbol("define".to_string()));
            assert_eq!(list[1], Value::Symbol("factorial".to_string()));
            // Third element should be a lambda expression
            assert!(matches!(list[2], Value::List(_)));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_mixed_types_in_list() {
        let result = parse_test("(42 \"hello\" foo #t nil)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(42.0),
                Value::String("hello".to_string()),
                Value::Symbol("foo".to_string()),
                Value::Bool(true),
                Value::Nil
            ])
        );
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("").unwrap();
        let result = parse(tokens).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_whitespace_and_comments() {
        let source = "; This is a comment\n(+ 1 2) ; Another comment\n(* 3 4)";
        let tokens = tokenize(source).unwrap();
        let result = parse(tokens).unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );
    }

    #[test]
    fn test_deeply_nested_lists() {
        let result = parse_test("((((1))))").unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::List(vec![Value::List(vec![Value::List(
                vec![Value::Number(1.0)]
            )])])])
        );
    }

    #[test]
    fn test_multiple_nested_quotes() {
        let result = parse_test("''''x").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![
                    Value::Symbol("quote".to_string()),
                    Value::List(vec![
                        Value::Symbol("quote".to_string()),
                        Value::List(vec![
                            Value::Symbol("quote".to_string()),
                            Value::Symbol("x".to_string())
                        ])
                    ])
                ])
            ])
        );
    }

    #[test]
    fn test_quote_list_with_nested_quotes() {
        let result = parse_test("'(a 'b ''c)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![
                    Value::Symbol("a".to_string()),
                    Value::List(vec![
                        Value::Symbol("quote".to_string()),
                        Value::Symbol("b".to_string())
                    ]),
                    Value::List(vec![
                        Value::Symbol("quote".to_string()),
                        Value::List(vec![
                            Value::Symbol("quote".to_string()),
                            Value::Symbol("c".to_string())
                        ])
                    ])
                ])
            ])
        );
    }

    #[test]
    fn test_large_list() {
        let input = format!(
            "({})",
            (1..=100)
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        );
        let result = parse_test(&input).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 100);
            assert_eq!(list[0], Value::Number(1.0));
            assert_eq!(list[99], Value::Number(100.0));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_heterogeneous_list() {
        let result = parse_test("(42 \"hello\" #t nil foo '(1 2))").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(42.0),
                Value::String("hello".to_string()),
                Value::Bool(true),
                Value::Nil,
                Value::Symbol("foo".to_string()),
                Value::List(vec![
                    Value::Symbol("quote".to_string()),
                    Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
                ])
            ])
        );
    }

    #[test]
    fn test_boolean_variations() {
        let result = parse_test("(#t #f true false)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Bool(true),
                Value::Bool(false),
                Value::Bool(true),
                Value::Bool(false)
            ])
        );
    }

    #[test]
    fn test_nil_variations() {
        let result = parse_test("(nil () nil)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Nil, Value::List(vec![]), Value::Nil])
        );
    }

    #[test]
    fn test_symbols_with_numbers() {
        let result = parse_test("(var1 var2 x123 test-123 123test)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("var1".to_string()),
                Value::Symbol("var2".to_string()),
                Value::Symbol("x123".to_string()),
                Value::Symbol("test-123".to_string()),
                Value::Symbol("123test".to_string())
            ])
        );
    }

    #[test]
    fn test_single_character_symbols() {
        let result = parse_test("(+ - * / = < > ?)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("-".to_string()),
                Value::Symbol("*".to_string()),
                Value::Symbol("/".to_string()),
                Value::Symbol("=".to_string()),
                Value::Symbol("<".to_string()),
                Value::Symbol(">".to_string()),
                Value::Symbol("?".to_string())
            ])
        );
    }

    #[test]
    fn test_numbers_edge_cases() {
        let result = parse_test("(0 -0 +0 42 -42 +42 3.14 -3.14 +3.14)").unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 9);
            assert_eq!(list[0], Value::Number(0.0));
            assert_eq!(list[1], Value::Number(0.0));
            assert_eq!(list[2], Value::Number(0.0));
            assert_eq!(list[8], Value::Number(3.14));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_string_edge_cases() {
        let result = parse_test("(\"\" \"a\" \"hello world\" \"with\\nescapes\")").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::String("".to_string()),
                Value::String("a".to_string()),
                Value::String("hello world".to_string()),
                Value::String("with\nescapes".to_string())
            ])
        );
    }

    #[test]
    fn test_unbalanced_parens_left() {
        assert!(parse_test("((").is_err());
        assert!(parse_test("(((").is_err());
        assert!(parse_test("(+ 1 (+ 2").is_err());
    }

    #[test]
    fn test_unbalanced_parens_right() {
        assert!(parse_test("))").is_err());
        assert!(parse_test(")))").is_err());
        assert!(parse_test("(+ 1 2))").is_err());
    }

    #[test]
    fn test_mismatched_structures() {
        assert!(parse_test("(+ 1 2 (").is_err());
        assert!(parse_test("(+ 1 2) )").is_err());
        assert!(parse_test("((+ 1) 2))").is_err());
    }

    #[test]
    fn test_quote_without_expression() {
        let tokens = vec![Token::Quote];
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn test_error_message_content() {
        match parse_test("(+ 1 2") {
            Err(RispyError::ParseError(msg)) => {
                assert!(msg.contains("Expected ')"));
            }
            _ => panic!("Expected ParseError with specific message"),
        }

        match parse_test("+ 1 2)") {
            Err(RispyError::ParseError(msg)) => {
                assert!(msg.contains("Unexpected ')'"));
            }
            _ => panic!("Expected ParseError with specific message"),
        }
    }

    #[test]
    fn test_complex_lisp_expressions() {
        // Function definition
        let func_def = "(define square (lambda (x) (* x x)))";
        let result = parse_test(func_def).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::Symbol("define".to_string()));
            assert_eq!(list[1], Value::Symbol("square".to_string()));
        } else {
            panic!("Expected list");
        }

        // Conditional expression
        let cond_expr = "(if (> x 0) \"positive\" \"non-positive\")";
        let result = parse_test(cond_expr).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 4);
            assert_eq!(list[0], Value::Symbol("if".to_string()));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_many_single_expressions() {
        let input = "1 2 3 4 5 6 7 8 9 10";
        let tokens = tokenize(input).unwrap();
        let result = parse(tokens).unwrap();
        assert_eq!(result.len(), 10);
        for (i, expr) in result.iter().enumerate() {
            assert_eq!(*expr, Value::Number((i + 1) as f64));
        }
    }

    #[test]
    fn test_alternating_quotes_and_atoms() {
        let result = parse_test("('a b 'c d 'e)").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::List(vec![
                    Value::Symbol("quote".to_string()),
                    Value::Symbol("a".to_string())
                ]),
                Value::Symbol("b".to_string()),
                Value::List(vec![
                    Value::Symbol("quote".to_string()),
                    Value::Symbol("c".to_string())
                ]),
                Value::Symbol("d".to_string()),
                Value::List(vec![
                    Value::Symbol("quote".to_string()),
                    Value::Symbol("e".to_string())
                ])
            ])
        );
    }

    #[test]
    fn test_quoted_empty_list() {
        let result = parse_test("'()").unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![])
            ])
        );
    }

    #[test]
    fn test_multiple_expressions_with_quotes() {
        let input = "'a 'b '(1 2) '(x y z)";
        let tokens = tokenize(input).unwrap();
        let result = parse(tokens).unwrap();
        assert_eq!(result.len(), 4);

        assert_eq!(
            result[0],
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::Symbol("a".to_string())
            ])
        );

        assert_eq!(
            result[3],
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![
                    Value::Symbol("x".to_string()),
                    Value::Symbol("y".to_string()),
                    Value::Symbol("z".to_string())
                ])
            ])
        );
    }

    #[test]
    fn test_nested_function_calls() {
        let result = parse_test("(+ (* 2 3) (/ 8 4) (- 10 5))").unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 4);
            assert_eq!(list[0], Value::Symbol("+".to_string()));

            // Check nested structure
            if let Value::List(nested) = &list[1] {
                assert_eq!(nested[0], Value::Symbol("*".to_string()));
                assert_eq!(nested[1], Value::Number(2.0));
                assert_eq!(nested[2], Value::Number(3.0));
            } else {
                panic!("Expected nested list");
            }
        } else {
            panic!("Expected list");
        }
    }
}
