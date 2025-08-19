use crate::error::RispyError;
use crate::value::{Arity, BuiltinFunction, Function, Value, check_arity};
use std::collections::HashMap;

pub fn create_builtin_registry() -> HashMap<String, Function> {
    let mut registry = HashMap::new();

    // Arithmetic operations
    registry.insert("+".to_string(), Function::Builtin(builtin_add()));
    registry.insert("-".to_string(), Function::Builtin(builtin_subtract()));
    registry.insert("*".to_string(), Function::Builtin(builtin_multiply()));
    registry.insert("/".to_string(), Function::Builtin(builtin_divide()));

    // Comparison operations
    registry.insert("=".to_string(), Function::Builtin(builtin_equal()));
    registry.insert("<".to_string(), Function::Builtin(builtin_less_than()));

    // List operations
    registry.insert("car".to_string(), Function::Builtin(builtin_car()));
    registry.insert("cdr".to_string(), Function::Builtin(builtin_cdr()));
    registry.insert("cons".to_string(), Function::Builtin(builtin_cons()));

    // Type predicates
    registry.insert("atom?".to_string(), Function::Builtin(builtin_atom_p()));
    registry.insert("null?".to_string(), Function::Builtin(builtin_null_p()));

    // I/O
    registry.insert("print".to_string(), Function::Builtin(builtin_print()));

    registry
}

// Arithmetic operations

fn builtin_add() -> BuiltinFunction {
    BuiltinFunction {
        name: "+".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            let a = args[0].expect_number()?;
            let b = args[1].expect_number()?;
            Ok(Value::Number(a + b))
        },
    }
}

fn builtin_subtract() -> BuiltinFunction {
    BuiltinFunction {
        name: "-".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            let a = args[0].expect_number()?;
            let b = args[1].expect_number()?;
            Ok(Value::Number(a - b))
        },
    }
}

fn builtin_multiply() -> BuiltinFunction {
    BuiltinFunction {
        name: "*".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            let a = args[0].expect_number()?;
            let b = args[1].expect_number()?;
            Ok(Value::Number(a * b))
        },
    }
}

fn builtin_divide() -> BuiltinFunction {
    BuiltinFunction {
        name: "/".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            let a = args[0].expect_number()?;
            let b = args[1].expect_number()?;
            if b == 0.0 {
                Err(RispyError::RuntimeError("Division by zero".to_string()))
            } else {
                Ok(Value::Number(a / b))
            }
        },
    }
}

// Comparison operations

fn builtin_equal() -> BuiltinFunction {
    BuiltinFunction {
        name: "=".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            Ok(Value::Bool(args[0] == args[1]))
        },
    }
}

fn builtin_less_than() -> BuiltinFunction {
    BuiltinFunction {
        name: "<".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            let a = args[0].expect_number()?;
            let b = args[1].expect_number()?;
            Ok(Value::Bool(a < b))
        },
    }
}

// List operations

fn builtin_car() -> BuiltinFunction {
    BuiltinFunction {
        name: "car".to_string(),
        arity: Arity::Exact(1),
        func: |args| {
            check_arity(args, &Arity::Exact(1))?;
            let car = args[0].car()?;
            Ok(car.clone())
        },
    }
}

fn builtin_cdr() -> BuiltinFunction {
    BuiltinFunction {
        name: "cdr".to_string(),
        arity: Arity::Exact(1),
        func: |args| {
            check_arity(args, &Arity::Exact(1))?;
            args[0].cdr()
        },
    }
}

fn builtin_cons() -> BuiltinFunction {
    BuiltinFunction {
        name: "cons".to_string(),
        arity: Arity::Exact(2),
        func: |args| {
            check_arity(args, &Arity::Exact(2))?;
            Ok(args[0].clone().cons(args[1].clone()))
        },
    }
}

// Type predicates

fn builtin_atom_p() -> BuiltinFunction {
    BuiltinFunction {
        name: "atom?".to_string(),
        arity: Arity::Exact(1),
        func: |args| {
            check_arity(args, &Arity::Exact(1))?;
            Ok(Value::Bool(crate::value::is_atom(&args[0])))
        },
    }
}

fn builtin_null_p() -> BuiltinFunction {
    BuiltinFunction {
        name: "null?".to_string(),
        arity: Arity::Exact(1),
        func: |args| {
            check_arity(args, &Arity::Exact(1))?;
            Ok(Value::Bool(args[0].is_empty_list()))
        },
    }
}

// I/O operations

fn builtin_print() -> BuiltinFunction {
    BuiltinFunction {
        name: "print".to_string(),
        arity: Arity::Exact(1),
        func: |args| {
            check_arity(args, &Arity::Exact(1))?;
            println!("{}", args[0]);
            Ok(Value::Nil)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_add() {
        let add = builtin_add();
        let args = vec![Value::Number(2.0), Value::Number(3.0)];
        let result = (add.func)(&args).unwrap();
        assert_eq!(result, Value::Number(5.0));

        // Test with wrong arity
        let args_wrong = vec![Value::Number(2.0)];
        assert!((add.func)(&args_wrong).is_err());

        // Test with wrong types
        let args_wrong_type = vec![Value::Number(2.0), Value::String("hello".to_string())];
        assert!((add.func)(&args_wrong_type).is_err());
    }

    #[test]
    fn test_builtin_subtract() {
        let sub = builtin_subtract();
        let args = vec![Value::Number(5.0), Value::Number(3.0)];
        let result = (sub.func)(&args).unwrap();
        assert_eq!(result, Value::Number(2.0));

        // Test negative result
        let args_neg = vec![Value::Number(3.0), Value::Number(5.0)];
        let result_neg = (sub.func)(&args_neg).unwrap();
        assert_eq!(result_neg, Value::Number(-2.0));
    }

    #[test]
    fn test_builtin_multiply() {
        let mul = builtin_multiply();
        let args = vec![Value::Number(4.0), Value::Number(3.0)];
        let result = (mul.func)(&args).unwrap();
        assert_eq!(result, Value::Number(12.0));

        // Test with zero
        let args_zero = vec![Value::Number(0.0), Value::Number(5.0)];
        let result_zero = (mul.func)(&args_zero).unwrap();
        assert_eq!(result_zero, Value::Number(0.0));
    }

    #[test]
    fn test_builtin_divide() {
        let div = builtin_divide();
        let args = vec![Value::Number(6.0), Value::Number(2.0)];
        let result = (div.func)(&args).unwrap();
        assert_eq!(result, Value::Number(3.0));

        // Test division by zero
        let args_zero = vec![Value::Number(5.0), Value::Number(0.0)];
        assert!((div.func)(&args_zero).is_err());

        // Test fractional result
        let args_frac = vec![Value::Number(5.0), Value::Number(2.0)];
        let result_frac = (div.func)(&args_frac).unwrap();
        assert_eq!(result_frac, Value::Number(2.5));
    }

    #[test]
    fn test_builtin_equal() {
        let eq = builtin_equal();

        // Test equal numbers
        let args_eq = vec![Value::Number(42.0), Value::Number(42.0)];
        let result_eq = (eq.func)(&args_eq).unwrap();
        assert_eq!(result_eq, Value::Bool(true));

        // Test unequal numbers
        let args_neq = vec![Value::Number(42.0), Value::Number(43.0)];
        let result_neq = (eq.func)(&args_neq).unwrap();
        assert_eq!(result_neq, Value::Bool(false));

        // Test different types
        let args_diff = vec![Value::Number(42.0), Value::String("42".to_string())];
        let result_diff = (eq.func)(&args_diff).unwrap();
        assert_eq!(result_diff, Value::Bool(false));

        // Test equal strings
        let args_str = vec![
            Value::String("hello".to_string()),
            Value::String("hello".to_string()),
        ];
        let result_str = (eq.func)(&args_str).unwrap();
        assert_eq!(result_str, Value::Bool(true));
    }

    #[test]
    fn test_builtin_less_than() {
        let lt = builtin_less_than();

        // Test true case
        let args_true = vec![Value::Number(3.0), Value::Number(5.0)];
        let result_true = (lt.func)(&args_true).unwrap();
        assert_eq!(result_true, Value::Bool(true));

        // Test false case
        let args_false = vec![Value::Number(5.0), Value::Number(3.0)];
        let result_false = (lt.func)(&args_false).unwrap();
        assert_eq!(result_false, Value::Bool(false));

        // Test equal case
        let args_equal = vec![Value::Number(5.0), Value::Number(5.0)];
        let result_equal = (lt.func)(&args_equal).unwrap();
        assert_eq!(result_equal, Value::Bool(false));

        // Test wrong type
        let args_wrong = vec![Value::String("a".to_string()), Value::Number(5.0)];
        assert!((lt.func)(&args_wrong).is_err());
    }

    #[test]
    fn test_builtin_car() {
        let car = builtin_car();

        // Test normal list
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let args = vec![list];
        let result = (car.func)(&args).unwrap();
        assert_eq!(result, Value::Number(1.0));

        // Test single element list
        let single_list = Value::List(vec![Value::Symbol("x".to_string())]);
        let args_single = vec![single_list];
        let result_single = (car.func)(&args_single).unwrap();
        assert_eq!(result_single, Value::Symbol("x".to_string()));

        // Test empty list
        let empty_list = Value::List(vec![]);
        let args_empty = vec![empty_list];
        assert!((car.func)(&args_empty).is_err());

        // Test non-list
        let non_list = Value::Number(42.0);
        let args_non_list = vec![non_list];
        assert!((car.func)(&args_non_list).is_err());
    }

    #[test]
    fn test_builtin_cdr() {
        let cdr = builtin_cdr();

        // Test normal list
        let list = Value::List(vec![
            Value::Number(1.0),
            Value::Number(2.0),
            Value::Number(3.0),
        ]);
        let args = vec![list];
        let result = (cdr.func)(&args).unwrap();
        assert_eq!(
            result,
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)])
        );

        // Test single element list
        let single_list = Value::List(vec![Value::Symbol("x".to_string())]);
        let args_single = vec![single_list];
        let result_single = (cdr.func)(&args_single).unwrap();
        assert_eq!(result_single, Value::List(vec![]));

        // Test empty list
        let empty_list = Value::List(vec![]);
        let args_empty = vec![empty_list];
        assert!((cdr.func)(&args_empty).is_err());
    }

    #[test]
    fn test_builtin_cons() {
        let cons = builtin_cons();

        // Test cons with list
        let args = vec![
            Value::Number(1.0),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)]),
        ];
        let result = (cons.func)(&args).unwrap();
        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );

        // Test cons with nil
        let args_nil = vec![Value::Number(1.0), Value::Nil];
        let result_nil = (cons.func)(&args_nil).unwrap();
        assert_eq!(result_nil, Value::List(vec![Value::Number(1.0)]));

        // Test cons with non-list
        let args_non_list = vec![Value::Number(1.0), Value::Number(2.0)];
        let result_non_list = (cons.func)(&args_non_list).unwrap();
        assert_eq!(
            result_non_list,
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );
    }

    #[test]
    fn test_builtin_atom_p() {
        let atom_p = builtin_atom_p();

        // Test atoms
        let atoms = vec![
            Value::Nil,
            Value::Bool(true),
            Value::Number(42.0),
            Value::String("hello".to_string()),
            Value::Symbol("foo".to_string()),
        ];

        for atom in atoms {
            let args = vec![atom];
            let result = (atom_p.func)(&args).unwrap();
            assert_eq!(result, Value::Bool(true));
        }

        // Test non-atom (list)
        let list = Value::List(vec![Value::Number(1.0)]);
        let args_list = vec![list];
        let result_list = (atom_p.func)(&args_list).unwrap();
        assert_eq!(result_list, Value::Bool(false));
    }

    #[test]
    fn test_builtin_null_p() {
        let null_p = builtin_null_p();

        // Test nil
        let args_nil = vec![Value::Nil];
        let result_nil = (null_p.func)(&args_nil).unwrap();
        assert_eq!(result_nil, Value::Bool(true));

        // Test empty list
        let args_empty = vec![Value::List(vec![])];
        let result_empty = (null_p.func)(&args_empty).unwrap();
        assert_eq!(result_empty, Value::Bool(true));

        // Test non-empty list
        let args_non_empty = vec![Value::List(vec![Value::Number(1.0)])];
        let result_non_empty = (null_p.func)(&args_non_empty).unwrap();
        assert_eq!(result_non_empty, Value::Bool(false));

        // Test other types
        let args_other = vec![Value::Number(0.0)];
        let result_other = (null_p.func)(&args_other).unwrap();
        assert_eq!(result_other, Value::Bool(false));
    }

    #[test]
    fn test_builtin_registry() {
        let registry = create_builtin_registry();

        // Test all expected functions are present
        let expected_functions = vec![
            "+", "-", "*", "/", "=", "<", "car", "cdr", "cons", "atom?", "null?", "print",
        ];

        for func_name in expected_functions {
            assert!(
                registry.contains_key(func_name),
                "Missing function: {}",
                func_name
            );
            if let Function::Builtin(builtin) = &registry[func_name] {
                assert_eq!(builtin.name, func_name);
            } else {
                panic!("Expected builtin function for {}", func_name);
            }
        }
    }

    #[test]
    fn test_arithmetic_edge_cases() {
        // Test negative numbers
        let add = builtin_add();
        let args_neg = vec![Value::Number(-5.0), Value::Number(3.0)];
        let result_neg = (add.func)(&args_neg).unwrap();
        assert_eq!(result_neg, Value::Number(-2.0));

        // Test floating point
        let mul = builtin_multiply();
        let args_float = vec![Value::Number(1.5), Value::Number(2.0)];
        let result_float = (mul.func)(&args_float).unwrap();
        assert_eq!(result_float, Value::Number(3.0));

        // Test very large numbers
        let add = builtin_add();
        let args_large = vec![Value::Number(1e10), Value::Number(1e10)];
        let result_large = (add.func)(&args_large).unwrap();
        assert_eq!(result_large, Value::Number(2e10));
    }

    #[test]
    fn test_comparison_edge_cases() {
        let eq = builtin_equal();

        // Test nil equality
        let args_nil = vec![Value::Nil, Value::Nil];
        let result_nil = (eq.func)(&args_nil).unwrap();
        assert_eq!(result_nil, Value::Bool(true));

        // Test list equality
        let list1 = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let list2 = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let args_lists = vec![list1, list2];
        let result_lists = (eq.func)(&args_lists).unwrap();
        assert_eq!(result_lists, Value::Bool(true));

        // Test floating point comparison
        let lt = builtin_less_than();
        let args_float = vec![Value::Number(1.5), Value::Number(1.6)];
        let result_float = (lt.func)(&args_float).unwrap();
        assert_eq!(result_float, Value::Bool(true));
    }

    #[test]
    fn test_list_operations_edge_cases() {
        // Test car/cdr with nested lists
        let nested = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::Number(3.0),
        ]);

        let car = builtin_car();
        let args_car = vec![nested.clone()];
        let result_car = (car.func)(&args_car).unwrap();
        assert_eq!(
            result_car,
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );

        let cdr = builtin_cdr();
        let args_cdr = vec![nested];
        let result_cdr = (cdr.func)(&args_cdr).unwrap();
        assert_eq!(result_cdr, Value::List(vec![Value::Number(3.0)]));

        // Test cons with mixed types
        let cons = builtin_cons();
        let args_mixed = vec![
            Value::String("hello".to_string()),
            Value::List(vec![Value::Bool(true)]),
        ];
        let result_mixed = (cons.func)(&args_mixed).unwrap();
        assert_eq!(
            result_mixed,
            Value::List(vec![Value::String("hello".to_string()), Value::Bool(true)])
        );
    }
}
