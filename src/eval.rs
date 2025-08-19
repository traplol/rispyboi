use crate::builtins;
use crate::error::RispyError;
use crate::macros::MacroExpander;
use crate::value::{Environment, Function, MacroEnvironment, MacroFunction, Value};
use std::rc::Rc;

pub fn eval(expr: &Value, env: &Rc<Environment>) -> Result<Value, RispyError> {
    // For backward compatibility, use eval_with_macros with empty macro environment
    let macro_env = MacroEnvironment::new();
    let macro_expander = MacroExpander::new(macro_env);
    eval_with_macros(expr, env, &macro_expander)
}

pub fn eval_with_macros(
    expr: &Value,
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    // Phase 1: Macro expansion
    let expanded = macro_expander.expand(expr)?;
    
    // Phase 2: Runtime evaluation
    eval_runtime(&expanded, env, macro_expander)
}

fn eval_runtime(
    expr: &Value,
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    match expr {
        // Self-evaluating forms
        Value::Nil | Value::Bool(_) | Value::Number(_) | Value::String(_) => Ok(expr.clone()),

        // Symbol lookup
        Value::Symbol(name) => eval_symbol(name, env),

        // List evaluation (function application or special form)
        Value::List(list) => eval_list_runtime(list, env, macro_expander),

        // Functions are values but shouldn't be directly evaluated
        Value::Function(_) => Ok(expr.clone()),
    }
}

fn eval_symbol(name: &str, env: &Rc<Environment>) -> Result<Value, RispyError> {
    env.get(name)
        .ok_or_else(|| RispyError::UndefinedSymbol(format!("Undefined variable: {}", name)))
}

fn eval_list_runtime(
    list: &[Value],
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if list.is_empty() {
        return Ok(Value::List(vec![])); // Empty list evaluates to itself
    }

    let first = &list[0];

    // Check for special forms first
    if let Value::Symbol(name) = first {
        match name.as_str() {
            "quote" => eval_quote(&list[1..], env),
            "if" => eval_if(&list[1..], env, macro_expander),
            "define" => eval_define(&list[1..], env, macro_expander),
            "define-function" => eval_define_function(&list[1..], env, macro_expander),
            "define-macro" => eval_define_macro(&list[1..], macro_expander),
            "set!" => eval_set(&list[1..], env, macro_expander),
            _ => {
                // Try function application
                eval_function_application(list, env, macro_expander)
            }
        }
    } else {
        // Evaluate first element in case it's a function expression
        let func = eval_runtime(first, env, macro_expander)?;
        if let Value::Function(_) = func {
            let mut new_list = vec![func];
            new_list.extend_from_slice(&list[1..]);
            eval_function_application(&new_list, env, macro_expander)
        } else {
            Err(RispyError::TypeError(
                "First element of list must be a function or symbol".to_string(),
            ))
        }
    }
}

fn eval_function_application(
    list: &[Value],
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if list.is_empty() {
        return Err(RispyError::RuntimeError(
            "Cannot apply empty list".to_string(),
        ));
    }

    let function_expr = &list[0];
    let args = &list[1..];

    // Evaluate arguments
    let mut evaluated_args = Vec::new();
    for arg in args {
        evaluated_args.push(eval_runtime(arg, env, macro_expander)?);
    }

    // Get the function
    let function = match function_expr {
        Value::Symbol(name) => {
            // First check the environment for user-defined functions
            if let Some(value) = env.get(name) {
                value.clone()
            } else {
                // Check built-in functions
                let builtins = builtins::create_builtin_registry();
                if let Some(builtin_func) = builtins.get(name) {
                    Value::Function(Box::new(builtin_func.clone()))
                } else {
                    return Err(RispyError::UndefinedSymbol(format!(
                        "Undefined function: {}",
                        name
                    )));
                }
            }
        }
        Value::Function(_) => function_expr.clone(),
        _ => {
            return Err(RispyError::TypeError(
                "Cannot apply non-function value".to_string(),
            ));
        }
    };

    // Apply the function
    match function {
        Value::Function(func) => match *func {
            Function::Builtin(builtin) => {
                // Call the builtin function
                (builtin.func)(&evaluated_args)
            }
            Function::UserDefined(user_func) => {
                // Check arity
                if evaluated_args.len() != user_func.params.len() {
                    return Err(RispyError::ArityError(format!(
                        "Function {} expects {} arguments, got {}",
                        user_func.name.as_deref().unwrap_or("<anonymous>"),
                        user_func.params.len(),
                        evaluated_args.len()
                    )));
                }

                // Create new environment with current environment as parent (for live references)
                let call_env = Environment::with_parent(env.clone());

                // Bind parameters to arguments
                for (param, arg) in user_func.params.iter().zip(evaluated_args.iter()) {
                    call_env.define(param.clone(), arg.clone());
                }

                // Evaluate the function body in the new environment
                eval_runtime(&user_func.body, &call_env, macro_expander)
            }
            Function::Macro(_) => {
                // TODO: Implement macro expansion
                Err(RispyError::RuntimeError(
                    "Macros not yet implemented".to_string(),
                ))
            }
        },
        _ => Err(RispyError::TypeError("Value is not a function".to_string())),
    }
}

fn eval_quote(args: &[Value], _env: &Rc<Environment>) -> Result<Value, RispyError> {
    if args.len() != 1 {
        return Err(RispyError::ArityError(
            "quote expects exactly 1 argument".to_string(),
        ));
    }

    // Return the argument without evaluation
    Ok(args[0].clone())
}

fn eval_if(
    args: &[Value],
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if args.len() != 3 {
        return Err(RispyError::ArityError(
            "if expects exactly 3 arguments (condition then else)".to_string(),
        ));
    }

    let condition = eval_runtime(&args[0], env, macro_expander)?;

    // Check if condition is truthy (anything except nil and #f is true)
    let is_truthy = match condition {
        Value::Nil => false,
        Value::Bool(false) => false,
        _ => true,
    };

    if is_truthy {
        eval_runtime(&args[1], env, macro_expander) // then branch
    } else {
        eval_runtime(&args[2], env, macro_expander) // else branch
    }
}

fn eval_define(
    args: &[Value],
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if args.len() != 2 {
        return Err(RispyError::ArityError(
            "define expects exactly 2 arguments (name value)".to_string(),
        ));
    }

    // First argument must be a symbol
    let name = match &args[0] {
        Value::Symbol(name) => name.clone(),
        _ => {
            return Err(RispyError::TypeError(
                "define: first argument must be a symbol".to_string(),
            ));
        }
    };

    // Evaluate the value
    let value = eval_runtime(&args[1], env, macro_expander)?;

    // Define the variable in the environment
    env.define(name, value);

    // Return nil (define doesn't return a value)
    Ok(Value::Nil)
}

fn eval_set(
    args: &[Value],
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if args.len() != 2 {
        return Err(RispyError::ArityError(
            "set! expects exactly 2 arguments (name value)".to_string(),
        ));
    }

    // First argument must be a symbol
    let name = match &args[0] {
        Value::Symbol(name) => name.clone(),
        _ => {
            return Err(RispyError::TypeError(
                "set!: first argument must be a symbol".to_string(),
            ));
        }
    };

    // Evaluate the value
    let value = eval_runtime(&args[1], env, macro_expander)?;

    // Set the variable in the environment (define if it doesn't exist)
    match env.set(&name, value.clone()) {
        Ok(()) => {}, // Variable existed and was updated
        Err(_) => {
            // Variable doesn't exist, so define it
            env.define(name, value.clone());
        }
    }

    // Return the new value
    Ok(value)
}

fn eval_define_function(
    args: &[Value],
    env: &Rc<Environment>,
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if args.len() != 3 {
        return Err(RispyError::ArityError(
            "define-function expects exactly 3 arguments (name params body)".to_string(),
        ));
    }

    // First argument must be a symbol (function name)
    let name = match &args[0] {
        Value::Symbol(name) => name.clone(),
        _ => {
            return Err(RispyError::TypeError(
                "define-function: first argument must be a symbol".to_string(),
            ));
        }
    };

    // Second argument must be a list of parameter symbols
    let params = match &args[1] {
        Value::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    Value::Symbol(param_name) => params.push(param_name.clone()),
                    _ => {
                        return Err(RispyError::TypeError(
                            "define-function: parameters must be symbols".to_string(),
                        ));
                    }
                }
            }
            params
        }
        _ => {
            return Err(RispyError::TypeError(
                "define-function: second argument must be a list of parameters".to_string(),
            ));
        }
    };

    // Third argument is the body (not evaluated here, stored for later)
    let body = args[2].clone();

    // Create a user-defined function with current environment
    let mut user_function = crate::value::UserFunction {
        name: Some(name.clone()),
        params,
        body: Box::new(body),
        closure: env.clone(), // Capture current environment as closure
    };

    let function_value = Value::Function(Box::new(Function::UserDefined(Box::new(
        user_function.clone(),
    ))));

    // Define the function in the environment first
    env.define(name.clone(), function_value.clone());

    // Update the closure to include the function itself (for recursion)
    user_function.closure = env.clone();
    let updated_function_value =
        Value::Function(Box::new(Function::UserDefined(Box::new(user_function))));

    // Redefine with updated closure
    env.define(name, updated_function_value);

    // Return nil
    Ok(Value::Nil)
}

fn eval_define_macro(
    args: &[Value],
    macro_expander: &MacroExpander,
) -> Result<Value, RispyError> {
    if args.len() != 3 {
        return Err(RispyError::ArityError(
            "define-macro expects exactly 3 arguments (name params body)".to_string(),
        ));
    }

    // First argument must be a symbol (macro name)
    let name = match &args[0] {
        Value::Symbol(name) => name.clone(),
        _ => {
            return Err(RispyError::TypeError(
                "define-macro: first argument must be a symbol".to_string(),
            ));
        }
    };

    // Second argument must be a list of parameter symbols
    let params = match &args[1] {
        Value::List(param_list) => {
            let mut params = Vec::new();
            for param in param_list {
                match param {
                    Value::Symbol(param_name) => params.push(param_name.clone()),
                    _ => {
                        return Err(RispyError::TypeError(
                            "define-macro: parameters must be symbols".to_string(),
                        ));
                    }
                }
            }
            params
        }
        _ => {
            return Err(RispyError::TypeError(
                "define-macro: second argument must be a list of parameters".to_string(),
            ));
        }
    };

    // Third argument is the body (template)
    let body = args[2].clone();

    // Create a macro function
    let macro_function = MacroFunction {
        name: name.clone(),
        params,
        body: Box::new(body),
        env: Environment::new(), // Macros have their own scope
    };

    // Define the macro in the macro environment
    macro_expander.define_macro(name, macro_function);

    // Return nil
    Ok(Value::Nil)
}

pub fn create_global_environment() -> Rc<Environment> {
    let env = Environment::new();

    // Add built-in functions to global environment
    let builtins = builtins::create_builtin_registry();
    for (name, function) in builtins {
        env.define(name, Value::Function(Box::new(function)));
    }

    env
}

pub fn create_global_environment_with_macros() -> (Rc<Environment>, MacroExpander) {
    let env = create_global_environment();
    let macro_env = MacroEnvironment::new();
    let macro_expander = MacroExpander::new(macro_env);
    (env, macro_expander)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_single;

    #[test]
    fn test_eval_self_evaluating() {
        let env = Environment::new();

        // Numbers
        assert_eq!(
            eval(&Value::Number(42.0), &env).unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(
            eval(&Value::Number(-3.14), &env).unwrap(),
            Value::Number(-3.14)
        );

        // Strings
        assert_eq!(
            eval(&Value::String("hello".to_string()), &env).unwrap(),
            Value::String("hello".to_string())
        );

        // Booleans
        assert_eq!(
            eval(&Value::Bool(true), &env).unwrap(),
            Value::Bool(true)
        );
        assert_eq!(
            eval(&Value::Bool(false), &env).unwrap(),
            Value::Bool(false)
        );

        // Nil
        assert_eq!(eval(&Value::Nil, &env).unwrap(), Value::Nil);
    }

    #[test]
    fn test_eval_empty_list() {
        let env = Environment::new();
        let empty_list = Value::List(vec![]);
        assert_eq!(eval(&empty_list, &env).unwrap(), empty_list);
    }

    #[test]
    fn test_eval_quote() {
        let env = Environment::new();

        // Quote a symbol
        let quoted_symbol = parse_single("'x").unwrap();
        let result = eval(&quoted_symbol, &env).unwrap();
        assert_eq!(result, Value::Symbol("x".to_string()));

        // Quote a list
        let quoted_list = parse_single("'(1 2 3)").unwrap();
        let result = eval(&quoted_list, &env).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );
    }

    #[test]
    fn test_eval_undefined_symbol() {
        let env = Environment::new();
        let symbol = Value::Symbol("undefined".to_string());
        let result = eval(&symbol, &env);
        assert!(matches!(result, Err(RispyError::UndefinedSymbol(_))));
    }

    #[test]
    fn test_eval_symbol_lookup() {
        let env = Environment::new();
        env.define("x".to_string(), Value::Number(42.0));

        let symbol = Value::Symbol("x".to_string());
        let result = eval(&symbol, &env).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_quote_arity_error() {
        let env = Environment::new();

        // Too many arguments to quote
        let expr = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);

        let result = eval(&expr, &env);
        assert!(matches!(result, Err(RispyError::ArityError(_))));

        // Too few arguments to quote
        let expr2 = Value::List(vec![Value::Symbol("quote".to_string())]);

        let result2 = eval(&expr2, &env);
        assert!(matches!(result2, Err(RispyError::ArityError(_))));
    }

    #[test]
    fn test_eval_unknown_function() {
        let env = Environment::new();

        let expr = Value::List(vec![
            Value::Symbol("unknown-function".to_string()),
            Value::Number(1.0),
        ]);

        let result = eval(&expr, &env);
        assert!(matches!(result, Err(RispyError::UndefinedSymbol(_))));
    }

    #[test]
    fn test_eval_functions_are_values() {
        let env = Environment::new();
        let func = Value::Function(Box::new(crate::value::Function::UserDefined(Box::new(
            crate::value::UserFunction {
                name: Some("test".to_string()),
                params: vec!["x".to_string()],
                body: Box::new(Value::Symbol("x".to_string())),
                closure: Environment::new(),
            },
        ))));

        let result = eval(&func, &env).unwrap();
        assert_eq!(result, func);
    }

    #[test]
    fn test_eval_multiple_expressions() {
        let env = Environment::new();
        env.define("x".to_string(), Value::Number(10.0));

        let exprs = vec![
            Value::Number(42.0),
            Value::Symbol("x".to_string()),
            Value::String("hello".to_string()),
        ];

        for (i, expr) in exprs.iter().enumerate() {
            let result = eval(expr, &env).unwrap();
            match i {
                0 => assert_eq!(result, Value::Number(42.0)),
                1 => assert_eq!(result, Value::Number(10.0)),
                2 => assert_eq!(result, Value::String("hello".to_string())),
                _ => panic!("Unexpected iteration"),
            }
        }
    }

    #[test]
    fn test_eval_nested_quotes() {
        let env = Environment::new();

        // ''x should become (quote (quote x))
        let nested_quote = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::Symbol("x".to_string()),
            ]),
        ]);

        let result = eval(&nested_quote, &env).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::Symbol("x".to_string())
            ])
        );
    }

    #[test]
    fn test_eval_quote_with_expressions() {
        let env = Environment::new();

        // Quote a list that would normally be evaluated
        let quoted_expr = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0),
            ]),
        ]);

        let result = eval(&quoted_expr, &env).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0)
            ])
        );
    }

    #[test]
    fn test_eval_symbol_case_sensitivity() {
        let env = Environment::new();
        env.define("var".to_string(), Value::Number(1.0));
        env.define("VAR".to_string(), Value::Number(2.0));
        env.define("Var".to_string(), Value::Number(3.0));

        assert_eq!(
            eval(&Value::Symbol("var".to_string()), &env).unwrap(),
            Value::Number(1.0)
        );
        assert_eq!(
            eval(&Value::Symbol("VAR".to_string()), &env).unwrap(),
            Value::Number(2.0)
        );
        assert_eq!(
            eval(&Value::Symbol("Var".to_string()), &env).unwrap(),
            Value::Number(3.0)
        );
    }

    #[test]
    fn test_eval_environment_isolation() {
        let env1 = Environment::new();
        let env2 = Environment::new();

        env1.define("x".to_string(), Value::Number(1.0));
        env2.define("x".to_string(), Value::Number(2.0));

        let symbol = Value::Symbol("x".to_string());

        assert_eq!(eval(&symbol, &env1).unwrap(), Value::Number(1.0));
        assert_eq!(eval(&symbol, &env2).unwrap(), Value::Number(2.0));
    }

    #[test]
    fn test_eval_all_self_evaluating_types() {
        let env = Environment::new();

        let test_cases = vec![
            (Value::Nil, Value::Nil),
            (Value::Bool(true), Value::Bool(true)),
            (Value::Bool(false), Value::Bool(false)),
            (Value::Number(0.0), Value::Number(0.0)),
            (Value::Number(-42.5), Value::Number(-42.5)),
            (Value::String("".to_string()), Value::String("".to_string())),
            (
                Value::String("test string".to_string()),
                Value::String("test string".to_string()),
            ),
        ];

        for (input, expected) in test_cases {
            let result = eval(&input, &env).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_eval_empty_environment() {
        let env = Environment::new();

        // Self-evaluating forms should work even in empty environment
        assert_eq!(
            eval(&Value::Number(42.0), &env).unwrap(),
            Value::Number(42.0)
        );
        assert_eq!(eval(&Value::Nil, &env).unwrap(), Value::Nil);

        // Symbol lookup should fail
        assert!(eval(&Value::Symbol("undefined".to_string()), &env).is_err());
    }

    #[test]
    fn test_eval_error_types() {
        let env = Environment::new();

        // Undefined symbol
        match eval(&Value::Symbol("undefined".to_string()), &env) {
            Err(RispyError::UndefinedSymbol(msg)) => {
                assert!(msg.contains("undefined"));
            }
            _ => panic!("Expected UndefinedSymbol error"),
        }

        // Quote with wrong arity
        let bad_quote = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);

        match eval(&bad_quote, &env) {
            Err(RispyError::ArityError(msg)) => {
                assert!(msg.contains("quote"));
                assert!(msg.contains("1 argument"));
            }
            _ => panic!("Expected ArityError"),
        }

        // Non-symbol in function position
        let bad_call = Value::List(vec![Value::Number(42.0), Value::Number(1.0)]);

        match eval(&bad_call, &env) {
            Err(RispyError::TypeError(msg)) => {
                assert!(msg.contains("symbol"));
            }
            _ => panic!("Expected TypeError"),
        }
    }

    #[test]
    fn test_eval_quote_edge_cases() {
        let env = Environment::new();

        // Quote nil
        let quote_nil = Value::List(vec![Value::Symbol("quote".to_string()), Value::Nil]);
        assert_eq!(eval(&quote_nil, &env).unwrap(), Value::Nil);

        // Quote empty list
        let quote_empty = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::List(vec![]),
        ]);
        assert_eq!(eval(&quote_empty, &env).unwrap(), Value::List(vec![]));

        // Quote complex nested structure
        let complex = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::List(vec![
                Value::Symbol("if".to_string()),
                Value::List(vec![
                    Value::Symbol("=".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Number(0.0),
                ]),
                Value::String("zero".to_string()),
                Value::String("non-zero".to_string()),
            ]),
        ]);

        let result = eval(&complex, &env).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list[0], Value::Symbol("if".to_string()));
            assert_eq!(list.len(), 4);
        } else {
            panic!("Expected list result");
        }
    }

    #[test]
    fn test_eval_large_expressions() {
        let env = Environment::new();

        // Large quoted list
        let large_list = Value::List(vec![
            Value::Symbol("quote".to_string()),
            Value::List((0..1000).map(|i| Value::Number(i as f64)).collect()),
        ]);

        let result = eval(&large_list, &env).unwrap();
        if let Value::List(list) = result {
            assert_eq!(list.len(), 1000);
            assert_eq!(list[0], Value::Number(0.0));
            assert_eq!(list[999], Value::Number(999.0));
        } else {
            panic!("Expected list result");
        }
    }

    #[test]
    fn test_eval_deeply_nested_quotes() {
        let env = Environment::new();

        // Create deeply nested quote structure
        let mut nested = Value::Symbol("x".to_string());
        for _ in 0..10 {
            nested = Value::List(vec![Value::Symbol("quote".to_string()), nested]);
        }

        // Should not crash and should reduce nesting by one level
        let result = eval(&nested, &env).unwrap();

        // Result should be one level less nested
        if let Value::List(list) = result {
            assert_eq!(list[0], Value::Symbol("quote".to_string()));
        } else {
            panic!("Expected list result");
        }
    }

    #[test]
    fn test_eval_with_integration() {
        // Test complete pipeline: parse then eval
        let env = Environment::new();
        env.define("x".to_string(), Value::Number(42.0));

        let test_cases = vec![
            ("42", Value::Number(42.0)),
            ("\"hello\"", Value::String("hello".to_string())),
            ("#t", Value::Bool(true)),
            ("#f", Value::Bool(false)),
            ("nil", Value::Nil),
            ("x", Value::Number(42.0)),
            ("'x", Value::Symbol("x".to_string())),
            (
                "'(1 2 3)",
                Value::List(vec![
                    Value::Number(1.0),
                    Value::Number(2.0),
                    Value::Number(3.0),
                ]),
            ),
        ];

        for (source, expected) in test_cases {
            let expr = parse_single(source).unwrap();
            let result = eval(&expr, &env).unwrap();
            assert_eq!(result, expected, "Failed for source: {}", source);
        }
    }

    #[test]
    fn test_define_macro_basic() {
        let (env, macro_expander) = create_global_environment_with_macros();
        
        // Define a simple identity macro
        let define_input = "(define-macro identity (x) x)";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval_with_macros(&ast[0], &env, &macro_expander).unwrap();
        assert_eq!(result, Value::Nil); // define-macro returns nil
        
        // Use the macro - should expand identity to just the argument
        let use_input = "(identity 42)";
        let tokens = crate::lexer::tokenize(use_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval_with_macros(&ast[0], &env, &macro_expander).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_define_macro_with_template() {
        let (env, macro_expander) = create_global_environment_with_macros();
        
        // Define a simple template macro without quasiquote for now
        // (define-macro when (test body) (if test body nil))
        let define_input = "(define-macro when (test body) (if test body nil))";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval_with_macros(&ast[0], &env, &macro_expander).unwrap();
        assert_eq!(result, Value::Nil);
        
        // Use the when macro
        let use_input = "(when #t (+ 1 2))";
        let tokens = crate::lexer::tokenize(use_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval_with_macros(&ast[0], &env, &macro_expander).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }

    #[test]
    fn test_macro_expansion_integration() {
        let (env, macro_expander) = create_global_environment_with_macros();
        
        // Test that macros are expanded before evaluation
        // This is an integration test for the two-phase evaluation
        
        // Define a simple macro without quasiquote for now
        let define_input = "(define-macro double (x) (* 2 x))";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval_with_macros(&ast[0], &env, &macro_expander).unwrap();
        
        // Use the macro in a larger expression
        let use_input = "(+ (double 3) (double 4))";
        let tokens = crate::lexer::tokenize(use_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval_with_macros(&ast[0], &env, &macro_expander).unwrap();
        assert_eq!(result, Value::Number(14.0)); // (* 2 3) + (* 2 4) = 6 + 8 = 14
    }

    #[test]
    fn test_environment_modification_persistence() {
        let env = Environment::new();

        // Define a variable
        env.define("x".to_string(), Value::Number(1.0));

        // Evaluate some expressions
        eval(&Value::Number(42.0), &env).unwrap();
        eval(&Value::String("hello".to_string()), &env).unwrap();

        // Variable should still be there
        assert_eq!(
            eval(&Value::Symbol("x".to_string()), &env).unwrap(),
            Value::Number(1.0)
        );

        // Add another variable
        env.define("y".to_string(), Value::Bool(true));

        // Both should be accessible
        assert_eq!(
            eval(&Value::Symbol("x".to_string()), &env).unwrap(),
            Value::Number(1.0)
        );
        assert_eq!(
            eval(&Value::Symbol("y".to_string()), &env).unwrap(),
            Value::Bool(true)
        );
    }

    #[test]
    fn test_eval_arithmetic_functions() {
        let env = create_global_environment();

        // Test addition
        let add_expr = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let result = eval(&add_expr, &env).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test subtraction
        let sub_expr = Value::List(vec![
            Value::Symbol("-".to_string()),
            Value::Number(10.0),
            Value::Number(4.0),
        ]);
        let result = eval(&sub_expr, &env).unwrap();
        assert_eq!(result, Value::Number(6.0));

        // Test multiplication
        let mul_expr = Value::List(vec![
            Value::Symbol("*".to_string()),
            Value::Number(3.0),
            Value::Number(4.0),
        ]);
        let result = eval(&mul_expr, &env).unwrap();
        assert_eq!(result, Value::Number(12.0));

        // Test division
        let div_expr = Value::List(vec![
            Value::Symbol("/".to_string()),
            Value::Number(15.0),
            Value::Number(3.0),
        ]);
        let result = eval(&div_expr, &env).unwrap();
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_eval_comparison_functions() {
        let env = create_global_environment();

        // Test equality
        let eq_expr = Value::List(vec![
            Value::Symbol("=".to_string()),
            Value::Number(5.0),
            Value::Number(5.0),
        ]);
        let result = eval(&eq_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(true));

        let neq_expr = Value::List(vec![
            Value::Symbol("=".to_string()),
            Value::Number(5.0),
            Value::Number(6.0),
        ]);
        let result = eval(&neq_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test less than
        let lt_expr = Value::List(vec![
            Value::Symbol("<".to_string()),
            Value::Number(3.0),
            Value::Number(5.0),
        ]);
        let result = eval(&lt_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(true));

        let not_lt_expr = Value::List(vec![
            Value::Symbol("<".to_string()),
            Value::Number(5.0),
            Value::Number(3.0),
        ]);
        let result = eval(&not_lt_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_list_functions() {
        let env = create_global_environment();

        // Test car
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let car_expr = Value::List(vec![
            Value::Symbol("car".to_string()),
            Value::List(vec![Value::Symbol("quote".to_string()), list.clone()]),
        ]);
        let result = eval(&car_expr, &env).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test cdr
        let cdr_expr = Value::List(vec![
            Value::Symbol("cdr".to_string()),
            Value::List(vec![Value::Symbol("quote".to_string()), list]),
        ]);
        let result = eval(&cdr_expr, &env).unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)])
        );

        // Test cons
        let cons_expr = Value::List(vec![
            Value::Symbol("cons".to_string()),
            Value::Number(1.0),
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
            ]),
        ]);
        let result = eval(&cons_expr, &env).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );
    }

    #[test]
    fn test_eval_type_predicates() {
        let env = create_global_environment();

        // Test atom?
        let atom_expr = Value::List(vec![
            Value::Symbol("atom?".to_string()),
            Value::Number(42.0),
        ]);
        let result = eval(&atom_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(true));

        let not_atom_expr = Value::List(vec![
            Value::Symbol("atom?".to_string()),
            Value::List(vec![
                Value::Symbol("quote".to_string()),
                Value::List(vec![Value::Number(1.0)]),
            ]),
        ]);
        let result = eval(&not_atom_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(false));

        // Test null?
        let null_expr = Value::List(vec![Value::Symbol("null?".to_string()), Value::Nil]);
        let result = eval(&null_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(true));

        let not_null_expr = Value::List(vec![
            Value::Symbol("null?".to_string()),
            Value::Number(42.0),
        ]);
        let result = eval(&not_null_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(false));
    }

    #[test]
    fn test_eval_nested_function_calls() {
        let env = create_global_environment();

        // Test (+ (* 2 3) 4) = 10
        let nested_expr = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::Number(4.0),
        ]);
        let result = eval(&nested_expr, &env).unwrap();
        assert_eq!(result, Value::Number(10.0));

        // Test (< (+ 1 2) (* 2 3)) = true (3 < 6)
        let complex_expr = Value::List(vec![
            Value::Symbol("<".to_string()),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(1.0),
                Value::Number(2.0),
            ]),
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
        ]);
        let result = eval(&complex_expr, &env).unwrap();
        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_eval_function_application_errors() {
        let env = create_global_environment();

        // Test undefined function
        let undefined_expr = Value::List(vec![
            Value::Symbol("undefined-function".to_string()),
            Value::Number(1.0),
        ]);
        let result = eval(&undefined_expr, &env);
        assert!(matches!(result, Err(RispyError::UndefinedSymbol(_))));

        // Test wrong arity
        let wrong_arity_expr =
            Value::List(vec![Value::Symbol("+".to_string()), Value::Number(1.0)]);
        let result = eval(&wrong_arity_expr, &env);
        assert!(matches!(result, Err(RispyError::ArityError(_))));

        // Test division by zero
        let div_zero_expr = Value::List(vec![
            Value::Symbol("/".to_string()),
            Value::Number(5.0),
            Value::Number(0.0),
        ]);
        let result = eval(&div_zero_expr, &env);
        assert!(matches!(result, Err(RispyError::RuntimeError(_))));

        // Test wrong type for arithmetic
        let wrong_type_expr = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Number(1.0),
            Value::String("hello".to_string()),
        ]);
        let result = eval(&wrong_type_expr, &env);
        assert!(matches!(result, Err(RispyError::TypeError(_))));
    }

    #[test]
    fn test_eval_complex_integration() {
        let env = create_global_environment();

        // Test complete parse and eval pipeline
        let test_cases = vec![
            ("(+ 1 2)", Value::Number(3.0)),
            ("(* 3 4)", Value::Number(12.0)),
            ("(- 10 3)", Value::Number(7.0)),
            ("(/ 12 4)", Value::Number(3.0)),
            ("(= 5 5)", Value::Bool(true)),
            ("(< 3 5)", Value::Bool(true)),
            ("(atom? 42)", Value::Bool(true)),
            ("(null? nil)", Value::Bool(true)),
            ("(+ (* 2 3) (/ 8 4))", Value::Number(8.0)), // 6 + 2 = 8
        ];

        for (source, expected) in test_cases {
            let expr = parse_single(source).unwrap();
            let result = eval(&expr, &env).unwrap();
            assert_eq!(result, expected, "Failed for source: {}", source);
        }
    }

    #[test]
    fn test_eval_if_special_form() {
        let env = create_global_environment();

        // Test true condition
        let if_true = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(true),
            Value::Number(42.0),
            Value::Number(0.0),
        ]);
        let result = eval(&if_true, &env).unwrap();
        assert_eq!(result, Value::Number(42.0));

        // Test false condition
        let if_false = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(false),
            Value::Number(42.0),
            Value::Number(24.0),
        ]);
        let result = eval(&if_false, &env).unwrap();
        assert_eq!(result, Value::Number(24.0));

        // Test nil condition (should be false)
        let if_nil = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Nil,
            Value::String("true".to_string()),
            Value::String("false".to_string()),
        ]);
        let result = eval(&if_nil, &env).unwrap();
        assert_eq!(result, Value::String("false".to_string()));

        // Test truthy values (non-nil, non-false)
        let if_truthy = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Number(0.0), // 0 is truthy in Lisp
            Value::String("zero is truthy".to_string()),
            Value::String("unreachable".to_string()),
        ]);
        let result = eval(&if_truthy, &env).unwrap();
        assert_eq!(result, Value::String("zero is truthy".to_string()));

        // Test with expression in condition
        let if_expr = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::List(vec![
                Value::Symbol("=".to_string()),
                Value::Number(2.0),
                Value::Number(2.0),
            ]),
            Value::String("equal".to_string()),
            Value::String("not equal".to_string()),
        ]);
        let result = eval(&if_expr, &env).unwrap();
        assert_eq!(result, Value::String("equal".to_string()));
    }

    #[test]
    fn test_eval_define_special_form() {
        let env = create_global_environment();

        // Test simple variable definition
        let define_expr = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
            Value::Number(42.0),
        ]);
        let result = eval(&define_expr, &env).unwrap();
        assert_eq!(result, Value::Nil); // define returns nil

        // Test that variable is accessible
        let lookup_expr = Value::Symbol("x".to_string());
        let result = eval(&lookup_expr, &env).unwrap();
        assert_eq!(result, Value::Number(42.0));

        // Test defining with expression
        let define_expr2 = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("y".to_string()),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Number(10.0),
                Value::Number(20.0),
            ]),
        ]);
        eval(&define_expr2, &env).unwrap();

        let lookup_y = Value::Symbol("y".to_string());
        let result = eval(&lookup_y, &env).unwrap();
        assert_eq!(result, Value::Number(30.0));

        // Test redefining variable
        let redefine_expr = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
            Value::String("redefined".to_string()),
        ]);
        eval(&redefine_expr, &env).unwrap();

        let lookup_x = Value::Symbol("x".to_string());
        let result = eval(&lookup_x, &env).unwrap();
        assert_eq!(result, Value::String("redefined".to_string()));
    }

    #[test]
    fn test_eval_define_function_special_form() {
        let env = create_global_environment();

        // Test simple function definition
        let define_func = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("square".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Symbol("x".to_string()),
                Value::Symbol("x".to_string()),
            ]),
        ]);
        let result = eval(&define_func, &env).unwrap();
        assert_eq!(result, Value::Nil); // define-function returns nil

        // Test calling the function
        let call_func = Value::List(vec![
            Value::Symbol("square".to_string()),
            Value::Number(5.0),
        ]);
        let result = eval(&call_func, &env).unwrap();
        assert_eq!(result, Value::Number(25.0));

        // Test function with multiple parameters
        let define_add = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("add".to_string()),
            Value::List(vec![
                Value::Symbol("a".to_string()),
                Value::Symbol("b".to_string()),
            ]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("a".to_string()),
                Value::Symbol("b".to_string()),
            ]),
        ]);
        eval(&define_add, &env).unwrap();

        let call_add = Value::List(vec![
            Value::Symbol("add".to_string()),
            Value::Number(3.0),
            Value::Number(7.0),
        ]);
        let result = eval(&call_add, &env).unwrap();
        assert_eq!(result, Value::Number(10.0));

        // Test function with no parameters
        let define_const = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("get-answer".to_string()),
            Value::List(vec![]), // No parameters
            Value::Number(42.0),
        ]);
        eval(&define_const, &env).unwrap();

        let call_const = Value::List(vec![Value::Symbol("get-answer".to_string())]);
        let result = eval(&call_const, &env).unwrap();
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_eval_user_function_scoping() {
        let env = create_global_environment();

        // Define a global variable
        let define_global = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("global-var".to_string()),
            Value::Number(100.0),
        ]);
        eval(&define_global, &env).unwrap();

        // Define a function that uses the global variable
        let define_func = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("use-global".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("+".to_string()),
                Value::Symbol("x".to_string()),
                Value::Symbol("global-var".to_string()),
            ]),
        ]);
        eval(&define_func, &env).unwrap();

        // Call the function
        let call_func = Value::List(vec![
            Value::Symbol("use-global".to_string()),
            Value::Number(5.0),
        ]);
        let result = eval(&call_func, &env).unwrap();
        assert_eq!(result, Value::Number(105.0)); // 5 + 100

        // Test parameter shadowing
        let define_shadow = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("shadow-test".to_string()),
            Value::List(vec![Value::Symbol("global-var".to_string())]), // Parameter shadows global
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Symbol("global-var".to_string()), // Should use parameter, not global
                Value::Number(2.0),
            ]),
        ]);
        eval(&define_shadow, &env).unwrap();

        let call_shadow = Value::List(vec![
            Value::Symbol("shadow-test".to_string()),
            Value::Number(10.0),
        ]);
        let result = eval(&call_shadow, &env).unwrap();
        assert_eq!(result, Value::Number(20.0)); // 10 * 2, not 100 * 2
    }

    #[test]
    fn test_eval_special_forms_errors() {
        let env = create_global_environment();

        // Test if with wrong arity
        let bad_if = Value::List(vec![
            Value::Symbol("if".to_string()),
            Value::Bool(true),
            Value::Number(1.0),
        ]); // Missing else clause
        let result = eval(&bad_if, &env);
        assert!(matches!(result, Err(RispyError::ArityError(_))));

        // Test define with wrong arity
        let bad_define = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Symbol("x".to_string()),
        ]); // Missing value
        let result = eval(&bad_define, &env);
        assert!(matches!(result, Err(RispyError::ArityError(_))));

        // Test define with non-symbol name
        let bad_define2 = Value::List(vec![
            Value::Symbol("define".to_string()),
            Value::Number(42.0), // Should be symbol
            Value::Number(1.0),
        ]);
        let result = eval(&bad_define2, &env);
        assert!(matches!(result, Err(RispyError::TypeError(_))));

        // Test define-function with wrong arity
        let bad_def_func = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("func".to_string()),
            Value::List(vec![]), // Missing body
        ]);
        let result = eval(&bad_def_func, &env);
        assert!(matches!(result, Err(RispyError::ArityError(_))));

        // Test define-function with non-symbol parameter
        let bad_params = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("func".to_string()),
            Value::List(vec![Value::Number(42.0)]), // Parameter should be symbol
            Value::Number(1.0),
        ]);
        let result = eval(&bad_params, &env);
        assert!(matches!(result, Err(RispyError::TypeError(_))));

        // Test calling function with wrong arity
        let define_func = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("two-param".to_string()),
            Value::List(vec![
                Value::Symbol("a".to_string()),
                Value::Symbol("b".to_string()),
            ]),
            Value::Number(42.0),
        ]);
        eval(&define_func, &env).unwrap();

        let wrong_arity_call = Value::List(vec![
            Value::Symbol("two-param".to_string()),
            Value::Number(1.0), // Only one argument, need two
        ]);
        let result = eval(&wrong_arity_call, &env);
        assert!(matches!(result, Err(RispyError::ArityError(_))));
    }

    #[test]
    fn test_eval_nested_special_forms() {
        let env = create_global_environment();

        // Test if inside function
        let define_func = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("abs".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("if".to_string()),
                Value::List(vec![
                    Value::Symbol("<".to_string()),
                    Value::Symbol("x".to_string()),
                    Value::Number(0.0),
                ]),
                Value::List(vec![
                    Value::Symbol("-".to_string()),
                    Value::Number(0.0),
                    Value::Symbol("x".to_string()),
                ]),
                Value::Symbol("x".to_string()),
            ]),
        ]);
        eval(&define_func, &env).unwrap();

        // Test positive number
        let call_pos = Value::List(vec![Value::Symbol("abs".to_string()), Value::Number(5.0)]);
        let result = eval(&call_pos, &env).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test negative number
        let call_neg = Value::List(vec![Value::Symbol("abs".to_string()), Value::Number(-3.0)]);
        let result = eval(&call_neg, &env).unwrap();
        assert_eq!(result, Value::Number(3.0));

        // Test function that calls another function
        let define_double = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("double".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Number(2.0),
                Value::Symbol("x".to_string()),
            ]),
        ]);
        eval(&define_double, &env).unwrap();

        let define_quadruple = Value::List(vec![
            Value::Symbol("define-function".to_string()),
            Value::Symbol("quadruple".to_string()),
            Value::List(vec![Value::Symbol("x".to_string())]),
            Value::List(vec![
                Value::Symbol("double".to_string()),
                Value::List(vec![
                    Value::Symbol("double".to_string()),
                    Value::Symbol("x".to_string()),
                ]),
            ]),
        ]);
        eval(&define_quadruple, &env).unwrap();

        // Test quadruple(3) = 12
        let call_quadruple = Value::List(vec![
            Value::Symbol("quadruple".to_string()),
            Value::Number(3.0),
        ]);
        let result = eval(&call_quadruple, &env).unwrap();
        assert_eq!(result, Value::Number(12.0)); // double(double(3)) = double(6) = 12
    }

    #[test]
    fn test_recursion_factorial_simple() {
        let env = create_global_environment();
        
        // Define factorial function
        let define_input = "(define-function factorial (n) (if (= n 0) 1 (* n (factorial (- n 1)))))";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Test factorial(3) - should now work with shared references
        let call_input = "(factorial 3)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // factorial(3) = 3 * 2 * 1 = 6
        assert_eq!(result, Value::Number(6.0));
    }

    #[test]
    fn test_recursion_fibonacci() {
        let env = create_global_environment();
        
        // Define fibonacci function
        let define_input = "(define-function fib (n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Test fib(5) - should now work with shared references
        let call_input = "(fib 5)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // fib(5) = 5 (sequence: 0, 1, 1, 2, 3, 5)
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_closure_basic_capture() {
        let env = create_global_environment();
        
        // Define a variable in outer scope
        let define_var = "(define x 42)";
        let tokens = crate::lexer::tokenize(define_var).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define function that captures x
        let define_func = "(define-function get-x () x)";
        let tokens = crate::lexer::tokenize(define_func).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Call function - should return captured value
        let call_input = "(get-x)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        assert_eq!(result, Value::Number(42.0));
    }

    #[test]
    fn test_closure_parameter_shadowing() {
        let env = create_global_environment();
        
        // Define a variable in outer scope
        let define_var = "(define x 42)";
        let tokens = crate::lexer::tokenize(define_var).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define function with parameter that shadows outer x
        let define_func = "(define-function shadow-test (x) x)";
        let tokens = crate::lexer::tokenize(define_func).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Call function with different value - should use parameter, not outer x
        let call_input = "(shadow-test 100)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        assert_eq!(result, Value::Number(100.0));
    }

    #[test]
    fn test_closure_nested_functions() {
        let env = create_global_environment();
        
        // Define outer variable
        let define_var = "(define global-val 10)";
        let tokens = crate::lexer::tokenize(define_var).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define outer function that defines inner function
        let define_outer = "(define-function outer (x) x)";
        let tokens = crate::lexer::tokenize(define_outer).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define function that uses global-val
        let define_inner = "(define-function inner () global-val)";
        let tokens = crate::lexer::tokenize(define_inner).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Call inner function
        let call_input = "(inner)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        assert_eq!(result, Value::Number(10.0));
    }

    #[test]
    fn test_closure_modification_isolation() {
        let env = create_global_environment();
        
        // Define a variable
        let define_var = "(define counter 0)";
        let tokens = crate::lexer::tokenize(define_var).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define function that reads counter
        let define_func = "(define-function get-counter () counter)";
        let tokens = crate::lexer::tokenize(define_func).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Modify counter
        let modify_var = "(define counter 5)";
        let tokens = crate::lexer::tokenize(modify_var).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Call function - should see the new value (since we don't have true closures yet)
        let call_input = "(get-counter)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // This test documents current behavior - function sees global environment changes
        assert_eq!(result, Value::Number(5.0));
    }

    #[test]
    fn test_recursion_countdown() {
        let env = create_global_environment();
        
        // Define countdown function
        let define_input = "(define-function countdown (n) (if (= n 0) 0 (countdown (- n 1))))";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Test countdown(3) - should now work with shared references
        let call_input = "(countdown 3)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // countdown always returns 0 when it reaches the base case
        assert_eq!(result, Value::Number(0.0));
    }

    #[test]
    fn test_recursion_mutual_recursion() {
        let env = create_global_environment();
        
        // Define mutually recursive functions (even/odd)
        let define_even = "(define-function is-even (n) (if (= n 0) 1 (is-odd (- n 1))))";
        let tokens = crate::lexer::tokenize(define_even).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        let define_odd = "(define-function is-odd (n) (if (= n 0) 0 (is-even (- n 1))))";
        let tokens = crate::lexer::tokenize(define_odd).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Test is-even(4) - should now work with shared references
        let call_input = "(is-even 4)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // is-even(4) should return 1 (true) since 4 is even
        assert_eq!(result, Value::Number(1.0));
    }

    #[test]
    fn test_closure_function_as_value() {
        let env = create_global_environment();
        
        // Define a simple function
        let define_func = "(define-function add-one (x) (+ x 1))";
        let tokens = crate::lexer::tokenize(define_func).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Get the function as a value
        let func_value = env.get("add-one").unwrap().clone();
        
        // Verify it's a function
        assert!(matches!(func_value, Value::Function(_)));
        
        if let Value::Function(func) = func_value {
            // Check function properties
            assert_eq!(func.name(), "add-one");
            
            if let crate::value::Function::UserDefined(user_func) = func.as_ref() {
                assert_eq!(user_func.params, vec!["x".to_string()]);
                assert_eq!(user_func.name, Some("add-one".to_string()));
            }
        }
    }

    #[test]
    fn test_closure_captures_at_definition_time() {
        let env = create_global_environment();
        
        // Define variable
        let define_var = "(define captured-value 100)";
        let tokens = crate::lexer::tokenize(define_var).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define function that captures the variable
        let define_func = "(define-function capture-test () captured-value)";
        let tokens = crate::lexer::tokenize(define_func).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Verify the function captured the environment
        if let Some(Value::Function(func)) = env.get("capture-test") {
            if let crate::value::Function::UserDefined(user_func) = func.as_ref() {
                // Check that the closure captured the variable
                assert_eq!(user_func.closure.get("captured-value"), Some(Value::Number(100.0)));
                
                // Also check that built-in functions are available in closure
                assert!(user_func.closure.get("+").is_some());
                assert!(user_func.closure.get("-").is_some());
            }
        }
    }

    #[test]
    fn test_recursion_tail_call_simulation() {
        let env = create_global_environment();
        
        // Define a "tail recursive" function (though we don't have tail call optimization)
        let define_input = "(define-function sum-to-n (n acc) (if (= n 0) acc (sum-to-n (- n 1) (+ acc n))))";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Test sum-to-n(5, 0) - should now work with shared references
        let call_input = "(sum-to-n 5 0)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // sum-to-n(5, 0) = 5 + 4 + 3 + 2 + 1 = 15
        assert_eq!(result, Value::Number(15.0));
    }

    #[test]
    fn test_set_variable_mutation() {
        let env = create_global_environment();
        
        // First define a variable
        let define_input = "(define x 10)";
        let tokens = crate::lexer::tokenize(define_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Check initial value
        assert_eq!(env.get("x"), Some(Value::Number(10.0)));
        
        // Use set! to change the value
        let set_input = "(set! x 42)";
        let tokens = crate::lexer::tokenize(set_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // set! should return the new value
        assert_eq!(result, Value::Number(42.0));
        
        // Check that the variable was actually changed
        assert_eq!(env.get("x"), Some(Value::Number(42.0)));
    }

    #[test]
    fn test_set_undefined_variable_defines() {
        let env = create_global_environment();
        
        // set! on undefined variable should define it
        let set_input = "(set! undefined-var 42)";
        let tokens = crate::lexer::tokenize(set_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        // Should return the new value
        assert_eq!(result, Value::Number(42.0));
        
        // Variable should now exist in environment
        assert_eq!(env.get("undefined-var"), Some(Value::Number(42.0)));
    }

    #[test]
    fn test_set_wrong_arity_error() {
        let env = create_global_environment();
        
        // Try set! with wrong number of arguments
        let set_input = "(set! x)";
        let tokens = crate::lexer::tokenize(set_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env);
        
        // Should get arity error
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, crate::error::RispyError::ArityError(_)));
        }
    }

    #[test]
    fn test_set_non_symbol_error() {
        let env = create_global_environment();
        
        // Try set! with non-symbol first argument
        let set_input = "(set! 42 100)";
        let tokens = crate::lexer::tokenize(set_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env);
        
        // Should get type error
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(matches!(e, crate::error::RispyError::TypeError(_)));
        }
    }

    #[test]
    fn test_closure_higher_order_function() {
        let env = create_global_environment();
        
        // Define a function that takes another function (conceptually)
        let define_apply = "(define-function apply-twice (f x) (f (f x)))";
        let tokens = crate::lexer::tokenize(define_apply).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define a simple function to apply
        let define_increment = "(define-function increment (x) (+ x 1))";
        let tokens = crate::lexer::tokenize(define_increment).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // This would fail because we can't pass functions as values yet
        // But we can test that the functions exist
        assert!(env.get("apply-twice").is_some());
        assert!(env.get("increment").is_some());
    }

    #[test]
    fn test_closure_complex_capture() {
        let env = create_global_environment();
        
        // Define multiple variables in different scopes
        let define_a = "(define a 1)";
        let tokens = crate::lexer::tokenize(define_a).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        let define_b = "(define b 2)";
        let tokens = crate::lexer::tokenize(define_b).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        let define_c = "(define c 3)";
        let tokens = crate::lexer::tokenize(define_c).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Define function that uses all three
        let define_func = "(define-function sum-abc () (+ a (+ b c)))";
        let tokens = crate::lexer::tokenize(define_func).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        eval(&ast[0], &env).unwrap();
        
        // Test the function
        let call_input = "(sum-abc)";
        let tokens = crate::lexer::tokenize(call_input).unwrap();
        let ast = crate::parser::parse(tokens).unwrap();
        let result = eval(&ast[0], &env).unwrap();
        
        assert_eq!(result, Value::Number(6.0)); // 1 + 2 + 3 = 6
        
        // Verify the closure captured all variables
        if let Some(Value::Function(func)) = env.get("sum-abc") {
            if let crate::value::Function::UserDefined(user_func) = func.as_ref() {
                assert_eq!(user_func.closure.get("a"), Some(Value::Number(1.0)));
                assert_eq!(user_func.closure.get("b"), Some(Value::Number(2.0)));
                assert_eq!(user_func.closure.get("c"), Some(Value::Number(3.0)));
            }
        }
    }
}
