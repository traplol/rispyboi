use crate::error::RispyError;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Symbol(String),
    List(Vec<Value>),
    Function(Box<Function>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    Builtin(BuiltinFunction),
    UserDefined(Box<UserFunction>),
    Macro(Box<MacroFunction>),
}

#[derive(Debug, Clone)]
pub struct BuiltinFunction {
    pub name: String,
    pub arity: Arity,
    pub func: fn(&[Value]) -> Result<Value, RispyError>,
}

impl PartialEq for BuiltinFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arity == other.arity
    }
}

#[derive(Debug, Clone)]
pub struct UserFunction {
    pub name: Option<String>,
    pub params: Vec<String>,
    pub body: Box<Value>,
    pub closure: Rc<Environment>,
}

impl PartialEq for UserFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name 
            && self.params == other.params 
            && self.body == other.body
            // Skip comparing closure environments for equality
    }
}

#[derive(Debug, Clone)]
pub struct MacroFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Box<Value>,
    pub env: Rc<Environment>,
}

impl PartialEq for MacroFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name 
            && self.params == other.params 
            && self.body == other.body
            // Skip comparing environments for equality
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Arity {
    Exact(usize),
    AtLeast(usize),
    Range(usize, usize),
    Variadic,
}

#[derive(Debug)]
pub struct Environment {
    bindings: RefCell<HashMap<String, Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Rc<Self> {
        Rc::new(Environment {
            bindings: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    pub fn with_parent(parent: Rc<Environment>) -> Rc<Self> {
        Rc::new(Environment {
            bindings: RefCell::new(HashMap::new()),
            parent: Some(parent),
        })
    }

    pub fn define(&self, name: String, value: Value) {
        self.bindings.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.bindings.borrow().get(name) {
            Some(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn set(&self, name: &str, value: Value) -> Result<(), RispyError> {
        if self.bindings.borrow().contains_key(name) {
            self.bindings.borrow_mut().insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &self.parent {
            parent.set(name, value)
        } else {
            Err(RispyError::UndefinedSymbol(format!(
                "Cannot set undefined variable: {}",
                name
            )))
        }
    }
}

#[derive(Debug)]
pub struct MacroEnvironment {
    macros: RefCell<HashMap<String, MacroFunction>>,
    parent: Option<Rc<MacroEnvironment>>,
}

impl MacroEnvironment {
    pub fn new() -> Rc<Self> {
        Rc::new(MacroEnvironment {
            macros: RefCell::new(HashMap::new()),
            parent: None,
        })
    }

    pub fn with_parent(parent: Rc<MacroEnvironment>) -> Rc<Self> {
        Rc::new(MacroEnvironment {
            macros: RefCell::new(HashMap::new()),
            parent: Some(parent),
        })
    }

    pub fn define(&self, name: String, macro_func: MacroFunction) {
        self.macros.borrow_mut().insert(name, macro_func);
    }

    pub fn get(&self, name: &str) -> Option<MacroFunction> {
        if let Some(macro_func) = self.macros.borrow().get(name) {
            Some(macro_func.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            None
        }
    }

    pub fn has_macro(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    pub fn declare_placeholder(&self, name: String) {
        // Create a placeholder macro for forward references
        let placeholder = MacroFunction {
            name: name.clone(),
            params: vec![],
            body: Box::new(Value::Nil),
            env: Environment::new(), // Empty environment for placeholder
        };
        self.macros.borrow_mut().insert(name, placeholder);
    }
}

impl Value {
    pub fn nil() -> Value {
        Value::Nil
    }

    pub fn bool(b: bool) -> Value {
        Value::Bool(b)
    }

    pub fn number(n: f64) -> Value {
        Value::Number(n)
    }

    pub fn string(s: impl Into<String>) -> Value {
        Value::String(s.into())
    }

    pub fn symbol(s: impl Into<String>) -> Value {
        Value::Symbol(s.into())
    }

    pub fn list(values: Vec<Value>) -> Value {
        Value::List(values)
    }

    pub fn car(&self) -> Result<&Value, RispyError> {
        match self {
            Value::List(list) => list.first().ok_or_else(|| {
                RispyError::RuntimeError("car: cannot get car of empty list".to_string())
            }),
            _ => Err(RispyError::TypeError(format!(
                "car: expected list, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn cdr(&self) -> Result<Value, RispyError> {
        match self {
            Value::List(list) => {
                if list.is_empty() {
                    Err(RispyError::RuntimeError(
                        "cdr: cannot get cdr of empty list".to_string(),
                    ))
                } else {
                    Ok(Value::List(list[1..].to_vec()))
                }
            }
            _ => Err(RispyError::TypeError(format!(
                "cdr: expected list, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn cons(self, tail: Value) -> Value {
        match tail {
            Value::List(mut list) => {
                list.insert(0, self);
                Value::List(list)
            }
            Value::Nil => Value::List(vec![self]),
            _ => Value::List(vec![self, tail]),
        }
    }

    pub fn length(&self) -> Result<usize, RispyError> {
        match self {
            Value::List(list) => Ok(list.len()),
            Value::Nil => Ok(0),
            _ => Err(RispyError::TypeError(format!(
                "length: expected list, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn is_empty_list(&self) -> bool {
        match self {
            Value::List(list) => list.is_empty(),
            Value::Nil => true,
            _ => false,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Nil => "nil",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::List(_) => "list",
            Value::Function(_) => "function",
        }
    }

    pub fn expect_number(&self) -> Result<f64, RispyError> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(RispyError::TypeError(format!(
                "Expected number, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn expect_string(&self) -> Result<&str, RispyError> {
        match self {
            Value::String(s) => Ok(s),
            _ => Err(RispyError::TypeError(format!(
                "Expected string, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn expect_symbol(&self) -> Result<&str, RispyError> {
        match self {
            Value::Symbol(s) => Ok(s),
            _ => Err(RispyError::TypeError(format!(
                "Expected symbol, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn expect_list(&self) -> Result<&[Value], RispyError> {
        match self {
            Value::List(list) => Ok(list),
            _ => Err(RispyError::TypeError(format!(
                "Expected list, got {}",
                self.type_name()
            ))),
        }
    }

    pub fn expect_function(&self) -> Result<&Function, RispyError> {
        match self {
            Value::Function(func) => Ok(func),
            _ => Err(RispyError::TypeError(format!(
                "Expected function, got {}",
                self.type_name()
            ))),
        }
    }
}

pub fn is_nil(value: &Value) -> bool {
    matches!(value, Value::Nil)
}

pub fn is_bool(value: &Value) -> bool {
    matches!(value, Value::Bool(_))
}

pub fn is_number(value: &Value) -> bool {
    matches!(value, Value::Number(_))
}

pub fn is_string(value: &Value) -> bool {
    matches!(value, Value::String(_))
}

pub fn is_symbol(value: &Value) -> bool {
    matches!(value, Value::Symbol(_))
}

pub fn is_list(value: &Value) -> bool {
    matches!(value, Value::List(_))
}

pub fn is_function(value: &Value) -> bool {
    matches!(value, Value::Function(_))
}

pub fn is_atom(value: &Value) -> bool {
    !is_list(value)
}

pub fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Nil => false,
        Value::Bool(false) => false,
        _ => true,
    }
}

impl Function {
    pub fn name(&self) -> &str {
        match self {
            Function::Builtin(f) => &f.name,
            Function::UserDefined(f) => f.name.as_deref().unwrap_or("<anonymous>"),
            Function::Macro(f) => &f.name,
        }
    }

    pub fn arity(&self) -> Arity {
        match self {
            Function::Builtin(f) => f.arity.clone(),
            Function::UserDefined(f) => Arity::Exact(f.params.len()),
            Function::Macro(f) => Arity::Exact(f.params.len()),
        }
    }
}

pub fn check_arity(args: &[Value], expected: &Arity) -> Result<(), RispyError> {
    let arg_count = args.len();
    let valid = match expected {
        Arity::Exact(n) => arg_count == *n,
        Arity::AtLeast(n) => arg_count >= *n,
        Arity::Range(min, max) => arg_count >= *min && arg_count <= *max,
        Arity::Variadic => true,
    };

    if !valid {
        let expected_str = match expected {
            Arity::Exact(n) => format!("{}", n),
            Arity::AtLeast(n) => format!("at least {}", n),
            Arity::Range(min, max) => format!("{} to {}", min, max),
            Arity::Variadic => "any number of".to_string(),
        };
        Err(RispyError::ArityError(format!(
            "Expected {} arguments, got {}",
            expected_str, arg_count
        )))
    } else {
        Ok(())
    }
}

fn escape_string(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => "\\\"".to_string(),
            '\\' => "\\\\".to_string(),
            '\n' => "\\n".to_string(),
            '\t' => "\\t".to_string(),
            '\r' => "\\r".to_string(),
            c => c.to_string(),
        })
        .collect()
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Bool(b) => write!(f, "#{}", if *b { "t" } else { "f" }),
            Value::Number(n) => {
                if n.fract() == 0.0 && n.is_finite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "\"{}\"", escape_string(s)),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::List(list) => {
                write!(f, "(")?;
                for (i, value) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, ")")
            }
            Value::Function(func) => write!(f, "#<function:{}>", func.name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_constructors() {
        assert_eq!(Value::nil(), Value::Nil);
        assert_eq!(Value::bool(true), Value::Bool(true));
        assert_eq!(Value::number(42.0), Value::Number(42.0));
        assert_eq!(Value::string("hello"), Value::String("hello".to_string()));
        assert_eq!(Value::symbol("foo"), Value::Symbol("foo".to_string()));
        assert_eq!(
            Value::list(vec![Value::number(1.0), Value::number(2.0)]),
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );
    }

    #[test]
    fn test_type_predicates() {
        assert!(is_nil(&Value::Nil));
        assert!(is_bool(&Value::Bool(true)));
        assert!(is_number(&Value::Number(42.0)));
        assert!(is_string(&Value::String("hello".to_string())));
        assert!(is_symbol(&Value::Symbol("foo".to_string())));
        assert!(is_list(&Value::List(vec![])));

        assert!(is_atom(&Value::Number(42.0)));
        assert!(!is_atom(&Value::List(vec![])));
    }

    #[test]
    fn test_truthiness() {
        assert!(!is_truthy(&Value::Nil));
        assert!(!is_truthy(&Value::Bool(false)));
        assert!(is_truthy(&Value::Bool(true)));
        assert!(is_truthy(&Value::Number(0.0)));
        assert!(is_truthy(&Value::String("".to_string())));
        assert!(is_truthy(&Value::List(vec![])));
    }

    #[test]
    fn test_list_operations() {
        let list = Value::list(vec![
            Value::number(1.0),
            Value::number(2.0),
            Value::number(3.0),
        ]);

        assert_eq!(list.car().unwrap(), &Value::Number(1.0));
        assert_eq!(
            list.cdr().unwrap(),
            Value::List(vec![Value::Number(2.0), Value::Number(3.0)])
        );
        assert_eq!(list.length().unwrap(), 3);
        assert!(!list.is_empty_list());

        let empty_list = Value::list(vec![]);
        assert!(empty_list.is_empty_list());
        assert_eq!(empty_list.length().unwrap(), 0);
        assert!(empty_list.car().is_err());
        assert!(empty_list.cdr().is_err());
    }

    #[test]
    fn test_cons() {
        let value = Value::number(1.0);
        let tail = Value::list(vec![Value::number(2.0), Value::number(3.0)]);
        let result = value.cons(tail);

        assert_eq!(
            result,
            Value::List(vec![
                Value::Number(1.0),
                Value::Number(2.0),
                Value::Number(3.0)
            ])
        );

        let value2 = Value::number(1.0);
        let nil_tail = Value::Nil;
        let result2 = value2.cons(nil_tail);
        assert_eq!(result2, Value::List(vec![Value::Number(1.0)]));
    }

    #[test]
    fn test_expect_methods() {
        let num = Value::number(42.0);
        assert_eq!(num.expect_number().unwrap(), 42.0);
        assert!(num.expect_string().is_err());

        let string = Value::string("hello");
        assert_eq!(string.expect_string().unwrap(), "hello");
        assert!(string.expect_number().is_err());

        let symbol = Value::symbol("foo");
        assert_eq!(symbol.expect_symbol().unwrap(), "foo");
        assert!(symbol.expect_list().is_err());

        let list = Value::list(vec![Value::number(1.0)]);
        assert_eq!(list.expect_list().unwrap(), &[Value::Number(1.0)]);
        assert!(list.expect_symbol().is_err());
    }

    #[test]
    fn test_type_names() {
        assert_eq!(Value::Nil.type_name(), "nil");
        assert_eq!(Value::Bool(true).type_name(), "boolean");
        assert_eq!(Value::Number(42.0).type_name(), "number");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::Symbol("foo".to_string()).type_name(), "symbol");
        assert_eq!(Value::List(vec![]).type_name(), "list");
    }

    #[test]
    fn test_display_formatting() {
        assert_eq!(format!("{}", Value::Nil), "nil");
        assert_eq!(format!("{}", Value::Bool(true)), "#t");
        assert_eq!(format!("{}", Value::Bool(false)), "#f");
        assert_eq!(format!("{}", Value::Number(42.0)), "42");
        assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
        assert_eq!(
            format!("{}", Value::String("hello".to_string())),
            "\"hello\""
        );
        assert_eq!(format!("{}", Value::Symbol("foo".to_string())), "foo");
        assert_eq!(
            format!(
                "{}",
                Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
            ),
            "(1 2)"
        );
        assert_eq!(format!("{}", Value::List(vec![])), "()");
    }

    #[test]
    fn test_string_escaping() {
        let value = Value::String("hello\nworld\t\"quoted\"\\backslash".to_string());
        assert_eq!(
            format!("{}", value),
            "\"hello\\nworld\\t\\\"quoted\\\"\\\\backslash\""
        );
    }

    #[test]
    fn test_arity_checking() {
        assert!(check_arity(&[], &Arity::Exact(0)).is_ok());
        assert!(check_arity(&[Value::Nil], &Arity::Exact(1)).is_ok());
        assert!(check_arity(&[Value::Nil], &Arity::Exact(0)).is_err());

        assert!(check_arity(&[Value::Nil, Value::Nil], &Arity::AtLeast(2)).is_ok());
        assert!(check_arity(&[Value::Nil, Value::Nil, Value::Nil], &Arity::AtLeast(2)).is_ok());
        assert!(check_arity(&[Value::Nil], &Arity::AtLeast(2)).is_err());

        assert!(check_arity(&[Value::Nil], &Arity::Range(1, 3)).is_ok());
        assert!(check_arity(&[Value::Nil, Value::Nil], &Arity::Range(1, 3)).is_ok());
        assert!(check_arity(&[], &Arity::Range(1, 3)).is_err());
        assert!(
            check_arity(
                &[Value::Nil, Value::Nil, Value::Nil, Value::Nil],
                &Arity::Range(1, 3)
            )
            .is_err()
        );

        assert!(check_arity(&[], &Arity::Variadic).is_ok());
        let many_nils: Vec<Value> = (0..100).map(|_| Value::Nil).collect();
        assert!(check_arity(&many_nils, &Arity::Variadic).is_ok());
    }

    #[test]
    fn test_environment() {
        let env = Environment::new();
        env.define("x".to_string(), Value::number(42.0));

        assert_eq!(env.get("x"), Some(Value::Number(42.0)));
        assert_eq!(env.get("y"), None);

        let child_env = Environment::with_parent(env);
        child_env.define("y".to_string(), Value::number(24.0));

        assert_eq!(child_env.get("x"), Some(Value::Number(42.0)));
        assert_eq!(child_env.get("y"), Some(Value::Number(24.0)));
    }

    #[test]
    fn test_function_arity() {
        let user_func = Function::UserDefined(Box::new(UserFunction {
            name: Some("test".to_string()),
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Value::Nil),
            closure: Environment::new(),
        }));

        assert_eq!(user_func.arity(), Arity::Exact(2));
        assert_eq!(user_func.name(), "test");
    }

    #[test]
    fn test_value_equality() {
        // Same values should be equal
        assert_eq!(Value::Number(42.0), Value::Number(42.0));
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
        assert_eq!(Value::Bool(true), Value::Bool(true));
        assert_eq!(Value::Nil, Value::Nil);

        // Different values should not be equal
        assert_ne!(Value::Number(42.0), Value::Number(43.0));
        assert_ne!(
            Value::String("hello".to_string()),
            Value::String("world".to_string())
        );
        assert_ne!(Value::Bool(true), Value::Bool(false));
        assert_ne!(Value::Nil, Value::Number(0.0));
    }

    #[test]
    fn test_list_equality() {
        let list1 = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let list2 = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let list3 = Value::List(vec![Value::Number(2.0), Value::Number(1.0)]);
        let empty1 = Value::List(vec![]);
        let empty2 = Value::List(vec![]);

        assert_eq!(list1, list2);
        assert_ne!(list1, list3);
        assert_eq!(empty1, empty2);
        assert_ne!(list1, empty1);
    }

    #[test]
    fn test_nested_list_equality() {
        let nested1 = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ]);
        let nested2 = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(4.0)]),
        ]);
        let nested3 = Value::List(vec![
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)]),
            Value::List(vec![Value::Number(3.0), Value::Number(5.0)]),
        ]);

        assert_eq!(nested1, nested2);
        assert_ne!(nested1, nested3);
    }

    #[test]
    fn test_cons_with_different_types() {
        // Cons with number and list
        let result1 = Value::Number(1.0).cons(Value::List(vec![Value::Number(2.0)]));
        assert_eq!(
            result1,
            Value::List(vec![Value::Number(1.0), Value::Number(2.0)])
        );

        // Cons with string and nil
        let result2 = Value::String("hello".to_string()).cons(Value::Nil);
        assert_eq!(
            result2,
            Value::List(vec![Value::String("hello".to_string())])
        );

        // Cons with symbol and non-list
        let result3 = Value::Symbol("x".to_string()).cons(Value::Number(42.0));
        assert_eq!(
            result3,
            Value::List(vec![Value::Symbol("x".to_string()), Value::Number(42.0)])
        );
    }

    #[test]
    fn test_car_cdr_edge_cases() {
        // Single element list
        let single = Value::List(vec![Value::Number(42.0)]);
        assert_eq!(single.car().unwrap(), &Value::Number(42.0));
        assert_eq!(single.cdr().unwrap(), Value::List(vec![]));

        // Two element list
        let pair = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        assert_eq!(pair.car().unwrap(), &Value::Number(1.0));
        assert_eq!(pair.cdr().unwrap(), Value::List(vec![Value::Number(2.0)]));

        // Non-list types should error
        assert!(Value::Number(42.0).car().is_err());
        assert!(Value::String("hello".to_string()).cdr().is_err());
    }

    #[test]
    fn test_length_edge_cases() {
        assert_eq!(Value::Nil.length().unwrap(), 0);
        assert_eq!(Value::List(vec![]).length().unwrap(), 0);
        assert_eq!(Value::List(vec![Value::Nil]).length().unwrap(), 1);

        let long_list = Value::List((0..1000).map(|i| Value::Number(i as f64)).collect());
        assert_eq!(long_list.length().unwrap(), 1000);

        // Non-list types should error
        assert!(Value::Number(42.0).length().is_err());
    }

    #[test]
    fn test_is_empty_list_comprehensive() {
        assert!(Value::List(vec![]).is_empty_list());
        assert!(Value::Nil.is_empty_list());
        assert!(!Value::List(vec![Value::Nil]).is_empty_list());
        assert!(!Value::Number(0.0).is_empty_list());
        assert!(!Value::String("".to_string()).is_empty_list());
    }

    #[test]
    fn test_all_type_predicates() {
        let values = vec![
            Value::Nil,
            Value::Bool(true),
            Value::Number(42.0),
            Value::String("hello".to_string()),
            Value::Symbol("foo".to_string()),
            Value::List(vec![]),
        ];

        for (i, value) in values.iter().enumerate() {
            assert_eq!(is_nil(value), i == 0);
            assert_eq!(is_bool(value), i == 1);
            assert_eq!(is_number(value), i == 2);
            assert_eq!(is_string(value), i == 3);
            assert_eq!(is_symbol(value), i == 4);
            assert_eq!(is_list(value), i == 5);
            assert_eq!(is_atom(value), i != 5);
        }
    }

    #[test]
    fn test_truthiness_comprehensive() {
        // Falsy values
        assert!(!is_truthy(&Value::Nil));
        assert!(!is_truthy(&Value::Bool(false)));

        // Truthy values
        assert!(is_truthy(&Value::Bool(true)));
        assert!(is_truthy(&Value::Number(0.0)));
        assert!(is_truthy(&Value::Number(-1.0)));
        assert!(is_truthy(&Value::String("".to_string())));
        assert!(is_truthy(&Value::String("hello".to_string())));
        assert!(is_truthy(&Value::Symbol("foo".to_string())));
        assert!(is_truthy(&Value::List(vec![])));
        assert!(is_truthy(&Value::List(vec![Value::Nil])));
    }

    #[test]
    fn test_arity_edge_cases() {
        // Test all arity types
        assert!(check_arity(&[], &Arity::Exact(0)).is_ok());
        assert!(check_arity(&[Value::Nil], &Arity::Exact(0)).is_err());

        assert!(check_arity(&[], &Arity::AtLeast(0)).is_ok());
        assert!(check_arity(&[Value::Nil], &Arity::AtLeast(0)).is_ok());
        assert!(check_arity(&[], &Arity::AtLeast(1)).is_err());

        assert!(check_arity(&[], &Arity::Range(0, 2)).is_ok());
        assert!(check_arity(&[Value::Nil], &Arity::Range(0, 2)).is_ok());
        assert!(check_arity(&[Value::Nil, Value::Nil], &Arity::Range(0, 2)).is_ok());
        assert!(check_arity(&[Value::Nil, Value::Nil, Value::Nil], &Arity::Range(0, 2)).is_err());

        assert!(check_arity(&[], &Arity::Variadic).is_ok());
        let many_nils: Vec<Value> = (0..1000).map(|_| Value::Nil).collect();
        assert!(check_arity(&many_nils, &Arity::Variadic).is_ok());
    }

    #[test]
    fn test_environment_scoping() {
        let parent = Environment::new();
        parent.define("x".to_string(), Value::Number(42.0));
        parent.define("y".to_string(), Value::String("parent".to_string()));

        let child = Environment::with_parent(parent);
        child.define("y".to_string(), Value::String("child".to_string()));
        child.define("z".to_string(), Value::Bool(true));

        // Child should shadow parent's y
        assert_eq!(child.get("y"), Some(Value::String("child".to_string())));

        // Child should inherit parent's x
        assert_eq!(child.get("x"), Some(Value::Number(42.0)));

        // Child's own variable
        assert_eq!(child.get("z"), Some(Value::Bool(true)));

        // Undefined variable
        assert_eq!(child.get("undefined"), None);
    }

    #[test]
    fn test_environment_deep_nesting() {
        let env1 = Environment::new();
        env1.define("level".to_string(), Value::Number(1.0));

        let env2 = Environment::with_parent(env1);
        env2.define("level".to_string(), Value::Number(2.0));

        let env3 = Environment::with_parent(env2);
        env3.define("level".to_string(), Value::Number(3.0));

        let env4 = Environment::with_parent(env3);

        // Should get the most recent binding
        assert_eq!(env4.get("level"), Some(Value::Number(3.0)));
    }

    #[test]
    fn test_expect_methods_comprehensive() {
        let number = Value::Number(3.14);
        let string = Value::String("hello".to_string());
        let symbol = Value::Symbol("foo".to_string());
        let list = Value::List(vec![Value::Number(1.0), Value::Number(2.0)]);
        let nil = Value::Nil;

        // Positive cases
        assert_eq!(number.expect_number().unwrap(), 3.14);
        assert_eq!(string.expect_string().unwrap(), "hello");
        assert_eq!(symbol.expect_symbol().unwrap(), "foo");
        assert_eq!(
            list.expect_list().unwrap(),
            &[Value::Number(1.0), Value::Number(2.0)]
        );

        // Negative cases
        assert!(string.expect_number().is_err());
        assert!(number.expect_string().is_err());
        assert!(list.expect_symbol().is_err());
        assert!(symbol.expect_list().is_err());
        assert!(nil.expect_number().is_err());
    }

    #[test]
    fn test_display_comprehensive() {
        // Test display of all value types
        assert_eq!(format!("{}", Value::Nil), "nil");
        assert_eq!(format!("{}", Value::Bool(true)), "#t");
        assert_eq!(format!("{}", Value::Bool(false)), "#f");

        // Numbers
        assert_eq!(format!("{}", Value::Number(42.0)), "42");
        assert_eq!(format!("{}", Value::Number(-42.0)), "-42");
        assert_eq!(format!("{}", Value::Number(3.14)), "3.14");
        assert_eq!(format!("{}", Value::Number(0.0)), "0");

        // Strings with various escapes
        assert_eq!(
            format!("{}", Value::String("simple".to_string())),
            "\"simple\""
        );
        assert_eq!(
            format!("{}", Value::String("with\nline\tbreaks".to_string())),
            "\"with\\nline\\tbreaks\""
        );

        // Symbols
        assert_eq!(format!("{}", Value::Symbol("simple".to_string())), "simple");
        assert_eq!(
            format!("{}", Value::Symbol("kebab-case".to_string())),
            "kebab-case"
        );

        // Lists
        assert_eq!(format!("{}", Value::List(vec![])), "()");
        assert_eq!(format!("{}", Value::List(vec![Value::Number(1.0)])), "(1)");
        assert_eq!(
            format!(
                "{}",
                Value::List(vec![
                    Value::Number(1.0),
                    Value::String("hello".to_string()),
                    Value::Bool(true)
                ])
            ),
            "(1 \"hello\" #t)"
        );
    }

    #[test]
    fn test_display_nested_structures() {
        let nested = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::List(vec![
                Value::Symbol("*".to_string()),
                Value::Number(2.0),
                Value::Number(3.0),
            ]),
            Value::Number(4.0),
        ]);

        assert_eq!(format!("{}", nested), "(+ (* 2 3) 4)");
    }

    #[test]
    fn test_type_names_comprehensive() {
        assert_eq!(Value::Nil.type_name(), "nil");
        assert_eq!(Value::Bool(true).type_name(), "boolean");
        assert_eq!(Value::Bool(false).type_name(), "boolean");
        assert_eq!(Value::Number(42.0).type_name(), "number");
        assert_eq!(Value::Number(-3.14).type_name(), "number");
        assert_eq!(Value::String("".to_string()).type_name(), "string");
        assert_eq!(Value::String("hello".to_string()).type_name(), "string");
        assert_eq!(Value::Symbol("foo".to_string()).type_name(), "symbol");
        assert_eq!(Value::Symbol("+".to_string()).type_name(), "symbol");
        assert_eq!(Value::List(vec![]).type_name(), "list");
        assert_eq!(Value::List(vec![Value::Nil]).type_name(), "list");
    }

    #[test]
    fn test_value_cloning() {
        let original = Value::List(vec![
            Value::Number(1.0),
            Value::String("hello".to_string()),
            Value::List(vec![Value::Bool(true)]),
        ]);

        let cloned = original.clone();
        assert_eq!(original, cloned);

        // Ensure deep clone
        if let (Value::List(orig_list), Value::List(cloned_list)) = (&original, &cloned) {
            assert_eq!(orig_list.len(), cloned_list.len());
            for (orig_item, cloned_item) in orig_list.iter().zip(cloned_list.iter()) {
                assert_eq!(orig_item, cloned_item);
            }
        }
    }

    #[test]
    fn test_large_data_structures() {
        // Large list
        let large_list = Value::List((0..10000).map(|i| Value::Number(i as f64)).collect());
        assert_eq!(large_list.length().unwrap(), 10000);
        assert_eq!(large_list.car().unwrap(), &Value::Number(0.0));

        // Deep nesting
        let mut deeply_nested = Value::Number(42.0);
        for _ in 0..100 {
            deeply_nested = Value::List(vec![deeply_nested]);
        }

        // Should not crash
        assert_eq!(deeply_nested.type_name(), "list");
    }
}
