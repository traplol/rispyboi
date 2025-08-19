use rispyboi::*;

#[test]
fn test_pipeline_numbers() {
    let test_cases = vec!["42", "-3.14", "+123", "0", "0.0"];

    for source in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");

        // Should be a number
        assert!(matches!(result, value::Value::Number(_)));
    }
}

#[test]
fn test_pipeline_strings() {
    let test_cases = vec![
        "\"hello\"",
        "\"\"",
        "\"with\\nescapes\"",
        "\"multiple words here\"",
    ];

    for source in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");

        // Should be a string
        assert!(matches!(result, value::Value::String(_)));
    }
}

#[test]
fn test_pipeline_booleans_and_nil() {
    let test_cases = vec![
        ("#t", value::Value::Bool(true)),
        ("#f", value::Value::Bool(false)),
        ("true", value::Value::Bool(true)),
        ("false", value::Value::Bool(false)),
        ("nil", value::Value::Nil),
    ];

    for (source, expected) in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");
        assert_eq!(result, expected);
    }
}

#[test]
fn test_pipeline_quotes() {
    let test_cases = vec![
        ("'x", value::Value::Symbol("x".to_string())),
        ("'42", value::Value::Number(42.0)),
        ("'\"hello\"", value::Value::String("hello".to_string())),
        ("'()", value::Value::List(vec![])),
        (
            "'(1 2 3)",
            value::Value::List(vec![
                value::Value::Number(1.0),
                value::Value::Number(2.0),
                value::Value::Number(3.0),
            ]),
        ),
    ];

    for (source, expected) in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");
        assert_eq!(result, expected);
    }
}

#[test]
fn test_pipeline_symbols_with_environment() {
    let env = eval::create_global_environment();

    // Define some variables
    env.define("x".to_string(), value::Value::Number(42.0));
    env.define(
        "greeting".to_string(),
        value::Value::String("hello".to_string()),
    );
    env.define("flag".to_string(), value::Value::Bool(true));

    let test_cases = vec![
        ("x", value::Value::Number(42.0)),
        ("greeting", value::Value::String("hello".to_string())),
        ("flag", value::Value::Bool(true)),
    ];

    for (source, expected) in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");
        assert_eq!(result, expected);
    }
}

#[test]
fn test_pipeline_multiple_expressions() {
    let source = "42 \"hello\" #t nil 'x";
    let tokens = lexer::tokenize(source).expect("Lexing should succeed");
    let ast = parser::parse(tokens).expect("Parsing should succeed");
    let env = eval::create_global_environment();

    assert_eq!(ast.len(), 5);

    let expected = vec![
        value::Value::Number(42.0),
        value::Value::String("hello".to_string()),
        value::Value::Bool(true),
        value::Value::Nil,
        value::Value::Symbol("x".to_string()),
    ];

    for (i, expr) in ast.iter().enumerate() {
        let result = eval::eval(expr, &env).expect("Evaluation should succeed");
        assert_eq!(result, expected[i]);
    }
}

#[test]
fn test_pipeline_complex_quotes() {
    let test_cases = vec![
        "''x",
        "'(+ 1 2)",
        "'(if #t \"yes\" \"no\")",
        "'(define x 42)",
    ];

    for source in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");

        // All should evaluate to some value without error
        assert!(matches!(
            result,
            value::Value::Symbol(_) | value::Value::List(_)
        ));
    }
}

#[test]
fn test_pipeline_error_propagation() {
    let error_cases = vec![
        // Lexer errors
        ("\"unterminated string", "lex"),
        // Parser errors
        ("(", "parse"),
        (")", "parse"),
        ("(+ 1 2", "parse"),
        // Eval errors
        ("undefined-variable", "eval"),
    ];

    for (source, error_stage) in error_cases {
        match error_stage {
            "lex" => {
                assert!(
                    lexer::tokenize(source).is_err(),
                    "Should fail at lexing: {}",
                    source
                );
            }
            "parse" => {
                if let Ok(tokens) = lexer::tokenize(source) {
                    assert!(
                        parser::parse(tokens).is_err(),
                        "Should fail at parsing: {}",
                        source
                    );
                }
            }
            "eval" => {
                if let Ok(tokens) = lexer::tokenize(source) {
                    if let Ok(ast) = parser::parse(tokens) {
                        let env = eval::create_global_environment();
                        for expr in ast {
                            assert!(
                                eval::eval(&expr, &env).is_err(),
                                "Should fail at evaluation: {}",
                                source
                            );
                        }
                    }
                }
            }
            _ => panic!("Unknown error stage: {}", error_stage),
        }
    }
}

#[test]
fn test_pipeline_whitespace_and_comments() {
    let sources = vec![
        "  42  ",
        "\t42\t",
        "\n42\n",
        "; comment\n42",
        "42 ; inline comment",
        "; comment 1\n; comment 2\n42",
        "  ; comment  \n  42  ; another comment  ",
    ];

    for source in sources {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");
        assert_eq!(result, value::Value::Number(42.0));
    }
}

#[test]
fn test_pipeline_nested_structures() {
    let sources = vec!["'(1 (2 3) 4)", "'((()))", "'(a (b (c d) e) f)"];

    for source in sources {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");

        // Should be a list
        assert!(matches!(result, value::Value::List(_)));
    }
}

#[test]
fn test_pipeline_large_inputs() {
    // Large number list
    let numbers: Vec<String> = (1..=100).map(|i| i.to_string()).collect();
    let large_quote = format!("'({})", numbers.join(" "));

    let tokens = lexer::tokenize(&large_quote).expect("Lexing should succeed");
    let ast = parser::parse(tokens).expect("Parsing should succeed");
    let env = eval::create_global_environment();

    assert_eq!(ast.len(), 1);
    let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");

    if let value::Value::List(list) = result {
        assert_eq!(list.len(), 100);
        assert_eq!(list[0], value::Value::Number(1.0));
        assert_eq!(list[99], value::Value::Number(100.0));
    } else {
        panic!("Expected list result");
    }
}

#[test]
fn test_pipeline_mixed_expressions() {
    let source = r#"
        ; Multiple expressions with various types
        42
        "hello world"
        #t
        'symbol
        '(1 2 3)
        nil
        ''nested-quote
    "#;

    let tokens = lexer::tokenize(source).expect("Lexing should succeed");
    let ast = parser::parse(tokens).expect("Parsing should succeed");
    let env = eval::create_global_environment();

    assert_eq!(ast.len(), 7);

    let results: Vec<_> = ast
        .iter()
        .map(|expr| eval::eval(expr, &env).expect("Evaluation should succeed"))
        .collect();

    assert_eq!(results[0], value::Value::Number(42.0));
    assert_eq!(results[1], value::Value::String("hello world".to_string()));
    assert_eq!(results[2], value::Value::Bool(true));
    assert_eq!(results[3], value::Value::Symbol("symbol".to_string()));
    assert!(matches!(results[4], value::Value::List(_)));
    assert_eq!(results[5], value::Value::Nil);
    assert!(matches!(results[6], value::Value::List(_)));
}

#[test]
fn test_pipeline_quote_edge_cases() {
    let test_cases = vec![
        ("'nil", value::Value::Nil),
        ("'#t", value::Value::Bool(true)),
        ("'#f", value::Value::Bool(false)),
        ("'42", value::Value::Number(42.0)),
        ("'\"string\"", value::Value::String("string".to_string())),
    ];

    for (source, expected) in test_cases {
        let tokens = lexer::tokenize(source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");
        let env = eval::create_global_environment();

        assert_eq!(ast.len(), 1);
        let result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");
        assert_eq!(result, expected);
    }
}

#[test]
fn test_pipeline_environment_persistence() {
    let env = eval::create_global_environment();

    // Define variables through direct environment manipulation
    env.define("x".to_string(), value::Value::Number(1.0));
    env.define("y".to_string(), value::Value::Number(2.0));

    // Test multiple evaluations with the same environment
    let expressions = vec!["x", "y", "'(x y)", "x"];

    for expr_source in expressions {
        let tokens = lexer::tokenize(expr_source).expect("Lexing should succeed");
        let ast = parser::parse(tokens).expect("Parsing should succeed");

        assert_eq!(ast.len(), 1);
        let _result = eval::eval(&ast[0], &env).expect("Evaluation should succeed");

        // Environment should persist between evaluations
        assert_eq!(env.get("x"), Some(value::Value::Number(1.0)));
        assert_eq!(env.get("y"), Some(value::Value::Number(2.0)));
    }
}

#[test]
fn test_pipeline_comprehensive_syntax() {
    // Test all supported syntax elements in one go
    let source = r#"
        ; Comments are ignored
        42          ; number
        -3.14       ; negative number  
        "hello"     ; string
        #t          ; boolean true
        #f          ; boolean false
        nil         ; nil value
        'symbol     ; quoted symbol
        '()         ; quoted empty list
        '(1 2 3)    ; quoted list with numbers
        '(a b c)    ; quoted list with symbols
        ''x         ; nested quotes
    "#;

    let tokens = lexer::tokenize(source).expect("Lexing should succeed");
    let ast = parser::parse(tokens).expect("Parsing should succeed");
    let env = eval::create_global_environment();

    // Should parse into multiple expressions
    assert!(ast.len() >= 10);

    // All should evaluate successfully
    for expr in ast {
        let _result = eval::eval(&expr, &env).expect("All expressions should evaluate");
    }
}
