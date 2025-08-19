use crate::error::RispyError;
use crate::value::{MacroEnvironment, MacroFunction, Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// MacroExpander handles the expansion of macros in the AST
pub struct MacroExpander {
    macro_env: Rc<MacroEnvironment>,
    gensym_counter: AtomicUsize,
}

impl MacroExpander {
    pub fn new(macro_env: Rc<MacroEnvironment>) -> Self {
        MacroExpander {
            macro_env,
            gensym_counter: AtomicUsize::new(0),
        }
    }

    /// Generate a unique symbol for macro hygiene
    pub fn gensym(&self, prefix: &str) -> String {
        let id = self.gensym_counter.fetch_add(1, Ordering::SeqCst);
        format!("{}#{}", prefix, id)
    }

    /// Define a new macro in the macro environment
    pub fn define_macro(&self, name: String, macro_func: MacroFunction) {
        self.macro_env.define(name, macro_func);
    }

    /// Check if a value represents a macro call
    pub fn is_macro_call(&self, expr: &Value) -> bool {
        match expr {
            Value::List(list) if !list.is_empty() => {
                if let Value::Symbol(name) = &list[0] {
                    self.macro_env.has_macro(name)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// Expand all macros in an expression until no more expansions are possible
    pub fn expand(&self, expr: &Value) -> Result<Value, RispyError> {
        let mut current = expr.clone();
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 1000; // Prevent infinite expansion loops
        
        loop {
            if iteration_count >= MAX_ITERATIONS {
                return Err(RispyError::RuntimeError(
                    "Macro expansion exceeded maximum iterations (possible infinite loop)".to_string()
                ));
            }
            
            let expanded = self.expand_once(&current)?;
            if expanded == current {
                // No more expansions possible
                break;
            }
            current = expanded;
            iteration_count += 1;
        }
        
        Ok(current)
    }

    /// Perform one round of macro expansion
    pub fn expand_once(&self, expr: &Value) -> Result<Value, RispyError> {
        match expr {
            Value::List(list) if !list.is_empty() => {
                // Check if this is a macro call
                if let Value::Symbol(name) = &list[0] {
                    if let Some(macro_func) = self.macro_env.get(name) {
                        // This is a macro call - expand it
                        return self.expand_macro_call(&macro_func, &list[1..]);
                    }
                }
                
                // Not a macro call - recursively expand elements
                let mut expanded_list = Vec::new();
                for item in list {
                    expanded_list.push(self.expand_once(item)?);
                }
                Ok(Value::List(expanded_list))
            }
            // Non-list values don't contain macro calls
            _ => Ok(expr.clone()),
        }
    }

    /// Expand a single macro call
    fn expand_macro_call(
        &self,
        macro_func: &MacroFunction,
        args: &[Value],
    ) -> Result<Value, RispyError> {
        // Validate arity
        if args.len() != macro_func.params.len() {
            return Err(RispyError::ArityError(format!(
                "Macro {} expects {} arguments, got {}",
                macro_func.name,
                macro_func.params.len(),
                args.len()
            )));
        }

        // Create parameter bindings
        let mut bindings = HashMap::new();
        for (param, arg) in macro_func.params.iter().zip(args.iter()) {
            bindings.insert(param.clone(), arg.clone());
        }

        // Expand the macro body with parameter substitution
        self.expand_template(&macro_func.body, &bindings)
    }

    /// Expand a template with parameter substitution (quasiquote processing)
    fn expand_template(
        &self,
        template: &Value,
        bindings: &HashMap<String, Value>,
    ) -> Result<Value, RispyError> {
        match template {
            Value::Symbol(name) => {
                // Substitute macro parameters
                if let Some(value) = bindings.get(name) {
                    Ok(value.clone())
                } else {
                    Ok(template.clone())
                }
            }
            Value::List(list) if !list.is_empty() => {
                // Handle quasiquote, unquote, and unquote-splicing
                if let Value::Symbol(op) = &list[0] {
                    match op.as_str() {
                        "quasiquote" => {
                            if list.len() != 2 {
                                return Err(RispyError::RuntimeError(
                                    "quasiquote expects exactly 1 argument".to_string()
                                ));
                            }
                            self.eval_quasiquote(&list[1], bindings)
                        }
                        "unquote" => {
                            if list.len() != 2 {
                                return Err(RispyError::RuntimeError(
                                    "unquote expects exactly 1 argument".to_string()
                                ));
                            }
                            // Unquote should only appear within quasiquote
                            Err(RispyError::RuntimeError(
                                "unquote outside of quasiquote".to_string()
                            ))
                        }
                        "unquote-splicing" => {
                            // Unquote-splicing should only appear within quasiquote
                            Err(RispyError::RuntimeError(
                                "unquote-splicing outside of quasiquote".to_string()
                            ))
                        }
                        _ => {
                            // Regular list - recursively expand elements
                            let mut expanded_list = Vec::new();
                            for item in list {
                                expanded_list.push(self.expand_template(item, bindings)?);
                            }
                            Ok(Value::List(expanded_list))
                        }
                    }
                } else {
                    // Regular list - recursively expand elements
                    let mut expanded_list = Vec::new();
                    for item in list {
                        expanded_list.push(self.expand_template(item, bindings)?);
                    }
                    Ok(Value::List(expanded_list))
                }
            }
            // Other values pass through unchanged
            _ => Ok(template.clone()),
        }
    }

    /// Evaluate quasiquote expressions
    fn eval_quasiquote(
        &self,
        expr: &Value,
        bindings: &HashMap<String, Value>,
    ) -> Result<Value, RispyError> {
        match expr {
            Value::List(list) if !list.is_empty() => {
                if let Value::Symbol(op) = &list[0] {
                    match op.as_str() {
                        "unquote" => {
                            if list.len() != 2 {
                                return Err(RispyError::RuntimeError(
                                    "unquote expects exactly 1 argument".to_string()
                                ));
                            }
                            // Evaluate the unquoted expression
                            self.expand_template(&list[1], bindings)
                        }
                        "unquote-splicing" => {
                            return Err(RispyError::RuntimeError(
                                "unquote-splicing not valid in this position".to_string()
                            ));
                        }
                        _ => {
                            // Process list elements, handling splicing
                            self.eval_quasiquote_list(list, bindings)
                        }
                    }
                } else {
                    self.eval_quasiquote_list(list, bindings)
                }
            }
            Value::Symbol(name) => {
                // Check for parameter substitution
                if let Some(value) = bindings.get(name) {
                    Ok(value.clone())
                } else {
                    Ok(expr.clone())
                }
            }
            // Other values pass through unchanged
            _ => Ok(expr.clone()),
        }
    }

    /// Process a list within quasiquote, handling splicing
    fn eval_quasiquote_list(
        &self,
        list: &[Value],
        bindings: &HashMap<String, Value>,
    ) -> Result<Value, RispyError> {
        let mut result = Vec::new();
        
        for item in list {
            if let Value::List(item_list) = item {
                if let Some(Value::Symbol(op)) = item_list.first() {
                    if op == "unquote-splicing" {
                        if item_list.len() != 2 {
                            return Err(RispyError::RuntimeError(
                                "unquote-splicing expects exactly 1 argument".to_string()
                            ));
                        }
                        
                        // Evaluate the spliced expression
                        let spliced = self.expand_template(&item_list[1], bindings)?;
                        
                        // Splice the result into the list
                        match spliced {
                            Value::List(splice_list) => {
                                result.extend(splice_list);
                            }
                            _ => {
                                return Err(RispyError::RuntimeError(
                                    "unquote-splicing requires a list".to_string()
                                ));
                            }
                        }
                        continue;
                    }
                }
            }
            
            // Regular item - process with quasiquote
            result.push(self.eval_quasiquote(item, bindings)?);
        }
        
        Ok(Value::List(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Environment;

    fn create_test_macro_env() -> Rc<MacroEnvironment> {
        MacroEnvironment::new()
    }

    #[test]
    fn test_gensym_generation() {
        let macro_env = create_test_macro_env();
        let expander = MacroExpander::new(macro_env);
        
        let sym1 = expander.gensym("test");
        let sym2 = expander.gensym("test");
        
        assert_ne!(sym1, sym2);
        assert!(sym1.starts_with("test#"));
        assert!(sym2.starts_with("test#"));
    }

    #[test]
    fn test_is_macro_call_detection() {
        let macro_env = create_test_macro_env();
        let expander = MacroExpander::new(macro_env.clone());
        
        // Define a test macro
        let test_macro = MacroFunction {
            name: "test-macro".to_string(),
            params: vec!["x".to_string()],
            body: Box::new(Value::Symbol("x".to_string())),
            env: Environment::new(),
        };
        macro_env.define("test-macro".to_string(), test_macro);
        
        // Test macro call detection
        let macro_call = Value::List(vec![
            Value::Symbol("test-macro".to_string()),
            Value::Number(42.0),
        ]);
        assert!(expander.is_macro_call(&macro_call));
        
        // Test non-macro call
        let function_call = Value::List(vec![
            Value::Symbol("not-a-macro".to_string()),
            Value::Number(42.0),
        ]);
        assert!(!expander.is_macro_call(&function_call));
        
        // Test non-list
        assert!(!expander.is_macro_call(&Value::Number(42.0)));
    }

    #[test]
    fn test_simple_macro_expansion() {
        let macro_env = create_test_macro_env();
        let expander = MacroExpander::new(macro_env.clone());
        
        // Define a simple substitution macro: (define-macro identity (x) x)
        let identity_macro = MacroFunction {
            name: "identity".to_string(),
            params: vec!["x".to_string()],
            body: Box::new(Value::Symbol("x".to_string())),
            env: Environment::new(),
        };
        macro_env.define("identity".to_string(), identity_macro);
        
        // Test expansion: (identity 42) -> 42
        let macro_call = Value::List(vec![
            Value::Symbol("identity".to_string()),
            Value::Number(42.0),
        ]);
        
        let expanded = expander.expand(&macro_call).unwrap();
        assert_eq!(expanded, Value::Number(42.0));
    }

    #[test]
    fn test_macro_arity_error() {
        let macro_env = create_test_macro_env();
        let expander = MacroExpander::new(macro_env.clone());
        
        // Define a macro that expects 2 arguments
        let test_macro = MacroFunction {
            name: "binary-macro".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Value::Symbol("x".to_string())),
            env: Environment::new(),
        };
        macro_env.define("binary-macro".to_string(), test_macro);
        
        // Test with wrong number of arguments
        let wrong_arity_call = Value::List(vec![
            Value::Symbol("binary-macro".to_string()),
            Value::Number(42.0),
            // Missing second argument
        ]);
        
        let result = expander.expand(&wrong_arity_call);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), RispyError::ArityError(_)));
    }

    #[test]
    fn test_non_macro_passthrough() {
        let macro_env = create_test_macro_env();
        let expander = MacroExpander::new(macro_env);
        
        // Test that non-macro expressions pass through unchanged
        let expr = Value::List(vec![
            Value::Symbol("+".to_string()),
            Value::Number(1.0),
            Value::Number(2.0),
        ]);
        
        let expanded = expander.expand(&expr).unwrap();
        assert_eq!(expanded, expr);
    }
}