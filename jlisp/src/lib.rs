//! # JLisp
//!
//! A Lisp where all data is JSON and all functions are JMESPath.
//!
//! ## Examples
//!
//! ```
//! use jlisp::Jlisp;
//! use serde_json::json;
//!
//! let mut jlisp = Jlisp::new();
//!
//! // Simple arithmetic using JMESPath functions
//! let result = jlisp.eval(&json!(["add", 1, 2])).unwrap();
//! assert_eq!(result, json!(3.0));
//!
//! // Nested expressions
//! let result = jlisp.eval(&json!(["add", ["multiply", 2, 3], 4])).unwrap();
//! assert_eq!(result, json!(10.0));
//! ```

use jmespath::{Rcvar, Variable};
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JlispError {
    #[error("Unknown function: {0}")]
    UnknownFunction(String),
    #[error("Invalid expression: {0}")]
    InvalidExpression(String),
    #[error("JMESPath error: {0}")]
    JmespathError(String),
    #[error("Type error: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },
    #[error("Arity error: {name} expects {expected} args, got {got}")]
    ArityError {
        name: String,
        expected: usize,
        got: usize,
    },
}

pub type Result<T> = std::result::Result<T, JlispError>;

/// Special forms that are handled by the evaluator directly
#[derive(Debug, Clone, PartialEq)]
enum SpecialForm {
    Quote,     // Don't evaluate the argument
    If,        // Conditional
    Def,       // Define a variable/function
    Let,       // Local bindings (JLisp style with @.key access)
    LetNative, // Native JEP-011 let expression with $var syntax
    Lambda,    // Anonymous function (stored as JMESPath expression)
    Do,        // Sequence of expressions
    Jmes,      // Raw JMESPath expression
}

impl SpecialForm {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "quote" => Some(Self::Quote),
            "if" => Some(Self::If),
            "def" => Some(Self::Def),
            "let" => Some(Self::Let),
            "let$" => Some(Self::LetNative),
            "lambda" | "fn" => Some(Self::Lambda),
            "do" => Some(Self::Do),
            "jmes" | "$" => Some(Self::Jmes),
            _ => None,
        }
    }
}

/// A user-defined function (just a JMESPath expression)
#[derive(Debug, Clone)]
struct UserFn {
    params: Vec<String>,
    body: Value,
}

pub struct Jlisp {
    runtime: jmespath::Runtime,
    env: HashMap<String, Value>,
    user_fns: HashMap<String, UserFn>,
}

impl Default for Jlisp {
    fn default() -> Self {
        Self::new()
    }
}

impl Jlisp {
    pub fn new() -> Self {
        let mut runtime = jmespath::Runtime::new();
        runtime.register_builtin_functions();
        jmespath_extensions::register_all(&mut runtime);

        Self {
            runtime,
            env: HashMap::new(),
            user_fns: HashMap::new(),
        }
    }

    /// Evaluate a JLisp expression
    pub fn eval(&mut self, expr: &Value) -> Result<Value> {
        self.eval_with_context(expr, &Value::Null)
    }

    /// Evaluate with a context value (accessible as @)
    pub fn eval_with_context(&mut self, expr: &Value, ctx: &Value) -> Result<Value> {
        match expr {
            // Atoms evaluate to themselves
            Value::Null | Value::Bool(_) | Value::Number(_) => Ok(expr.clone()),

            // Strings: if it starts with @, evaluate as JMESPath, otherwise return as-is
            Value::String(s) => {
                if s.starts_with('@') || s.starts_with('$') {
                    self.eval_jmespath(s, ctx)
                } else if let Some(val) = self.env.get(s) {
                    Ok(val.clone())
                } else {
                    Ok(expr.clone())
                }
            }

            // Objects are evaluated with their values evaluated
            Value::Object(map) => {
                let mut result = serde_json::Map::new();
                for (k, v) in map {
                    result.insert(k.clone(), self.eval_with_context(v, ctx)?);
                }
                Ok(Value::Object(result))
            }

            // Arrays: if first element is a string, it's a function call [fn, arg1, arg2, ...]
            // Otherwise, it's a literal array that gets evaluated element-wise
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Ok(Value::Array(vec![]));
                }

                // Check if first element is a string (function call) or not (literal array)
                let fn_name = match &arr[0] {
                    Value::String(s) => s.clone(),
                    _ => {
                        // Not a function call - evaluate as literal array
                        let mut result = Vec::new();
                        for item in arr {
                            result.push(self.eval_with_context(item, ctx)?);
                        }
                        return Ok(Value::Array(result));
                    }
                };

                let args = &arr[1..];

                // Check for special forms first
                if let Some(special) = SpecialForm::from_str(&fn_name) {
                    return self.eval_special_form(special, args, ctx);
                }

                // Check for user-defined functions
                if let Some(user_fn) = self.user_fns.get(&fn_name).cloned() {
                    return self.eval_user_fn(&user_fn, args, ctx);
                }

                // Otherwise, it's a JMESPath function call
                self.eval_jmespath_fn(&fn_name, args, ctx)
            }
        }
    }

    fn eval_special_form(
        &mut self,
        form: SpecialForm,
        args: &[Value],
        ctx: &Value,
    ) -> Result<Value> {
        match form {
            SpecialForm::Quote => {
                if args.len() != 1 {
                    return Err(JlispError::ArityError {
                        name: "quote".to_string(),
                        expected: 1,
                        got: args.len(),
                    });
                }
                Ok(args[0].clone())
            }

            SpecialForm::If => {
                if args.len() < 2 || args.len() > 3 {
                    return Err(JlispError::ArityError {
                        name: "if".to_string(),
                        expected: 3,
                        got: args.len(),
                    });
                }
                let cond = self.eval_with_context(&args[0], ctx)?;
                let is_truthy = match &cond {
                    Value::Null => false,
                    Value::Bool(b) => *b,
                    Value::Array(a) => !a.is_empty(),
                    Value::Object(o) => !o.is_empty(),
                    Value::String(s) => !s.is_empty(),
                    Value::Number(_) => true,
                };
                if is_truthy {
                    self.eval_with_context(&args[1], ctx)
                } else if args.len() == 3 {
                    self.eval_with_context(&args[2], ctx)
                } else {
                    Ok(Value::Null)
                }
            }

            SpecialForm::Def => {
                if args.len() < 2 {
                    return Err(JlispError::ArityError {
                        name: "def".to_string(),
                        expected: 2,
                        got: args.len(),
                    });
                }
                let name = args[0]
                    .as_str()
                    .ok_or_else(|| JlispError::TypeError {
                        expected: "string".to_string(),
                        got: format!("{:?}", args[0]),
                    })?
                    .to_string();

                // If third arg exists, it's a function definition: ["def", "name", ["params"], body]
                if args.len() >= 3 {
                    let params = args[1]
                        .as_array()
                        .ok_or_else(|| JlispError::TypeError {
                            expected: "array of parameter names".to_string(),
                            got: format!("{:?}", args[1]),
                        })?
                        .iter()
                        .map(|v| {
                            v.as_str()
                                .map(|s| s.to_string())
                                .ok_or_else(|| JlispError::TypeError {
                                    expected: "string".to_string(),
                                    got: format!("{:?}", v),
                                })
                        })
                        .collect::<Result<Vec<_>>>()?;

                    let body = args[2].clone();
                    self.user_fns.insert(name.clone(), UserFn { params, body });
                    Ok(Value::String(name))
                } else {
                    // Simple variable definition
                    let value = self.eval_with_context(&args[1], ctx)?;
                    self.env.insert(name.clone(), value.clone());
                    Ok(value)
                }
            }

            SpecialForm::Let => {
                if args.len() != 2 {
                    return Err(JlispError::ArityError {
                        name: "let".to_string(),
                        expected: 2,
                        got: args.len(),
                    });
                }
                let bindings = args[0].as_object().ok_or_else(|| JlispError::TypeError {
                    expected: "object".to_string(),
                    got: format!("{:?}", args[0]),
                })?;

                // Create new context with bindings
                let mut new_ctx = ctx.clone();
                if let Value::Object(ref mut map) = new_ctx {
                    for (k, v) in bindings {
                        let evaluated = self.eval_with_context(v, ctx)?;
                        map.insert(k.clone(), evaluated);
                    }
                } else {
                    let mut map = serde_json::Map::new();
                    for (k, v) in bindings {
                        let evaluated = self.eval_with_context(v, ctx)?;
                        map.insert(k.clone(), evaluated);
                    }
                    new_ctx = Value::Object(map);
                }

                self.eval_with_context(&args[1], &new_ctx)
            }

            SpecialForm::Lambda => {
                // Return a representation of the lambda
                Ok(Value::Object(serde_json::Map::from_iter([
                    ("__lambda".to_string(), Value::Bool(true)),
                    (
                        "params".to_string(),
                        args.get(0).cloned().unwrap_or(Value::Array(vec![])),
                    ),
                    (
                        "body".to_string(),
                        args.get(1).cloned().unwrap_or(Value::Null),
                    ),
                ])))
            }

            SpecialForm::Do => {
                let mut result = Value::Null;
                for arg in args {
                    result = self.eval_with_context(arg, ctx)?;
                }
                Ok(result)
            }

            SpecialForm::Jmes => {
                // Raw JMESPath expression: ["jmes", "let $x = `5` in add($x, `1`)"]
                // or ["$", "expression"]
                if args.is_empty() {
                    return Err(JlispError::ArityError {
                        name: "jmes".to_string(),
                        expected: 1,
                        got: 0,
                    });
                }
                let expr = args[0].as_str().ok_or_else(|| JlispError::TypeError {
                    expected: "string".to_string(),
                    got: format!("{:?}", args[0]),
                })?;
                self.eval_jmespath(expr, ctx)
            }

            SpecialForm::LetNative => {
                // Native JEP-011 let: ["let$", {"x": 5, "y": 10}, "add($x, $y)"]
                // Builds: let $x = `5`, $y = `10` in add($x, $y)
                if args.len() != 2 {
                    return Err(JlispError::ArityError {
                        name: "let$".to_string(),
                        expected: 2,
                        got: args.len(),
                    });
                }
                let bindings = args[0].as_object().ok_or_else(|| JlispError::TypeError {
                    expected: "object".to_string(),
                    got: format!("{:?}", args[0]),
                })?;
                let body = args[1].as_str().ok_or_else(|| JlispError::TypeError {
                    expected: "string (JMESPath expression)".to_string(),
                    got: format!("{:?}", args[1]),
                })?;

                // Evaluate bindings and build let expression
                let mut binding_strs = Vec::new();
                for (name, value) in bindings {
                    let evaluated = self.eval_with_context(value, ctx)?;
                    binding_strs.push(format!("${} = `{}`", name, evaluated));
                }

                let expr = format!("let {} in {}", binding_strs.join(", "), body);
                self.eval_jmespath(&expr, ctx)
            }
        }
    }

    fn eval_user_fn(&mut self, user_fn: &UserFn, args: &[Value], ctx: &Value) -> Result<Value> {
        if args.len() != user_fn.params.len() {
            return Err(JlispError::ArityError {
                name: "user function".to_string(),
                expected: user_fn.params.len(),
                got: args.len(),
            });
        }

        // Build context with evaluated arguments
        let mut fn_ctx = serde_json::Map::new();
        for (param, arg) in user_fn.params.iter().zip(args.iter()) {
            let evaluated = self.eval_with_context(arg, ctx)?;
            fn_ctx.insert(param.clone(), evaluated);
        }

        let body = user_fn.body.clone();
        self.eval_with_context(&body, &Value::Object(fn_ctx))
    }

    fn eval_jmespath_fn(&mut self, fn_name: &str, args: &[Value], ctx: &Value) -> Result<Value> {
        // Evaluate all arguments first
        let mut eval_args: Vec<Value> = Vec::new();
        for arg in args {
            eval_args.push(self.eval_with_context(arg, ctx)?);
        }

        // Build a JMESPath expression that calls the function
        let args_json: Vec<String> = eval_args
            .iter()
            .map(|v| {
                // Use backticks for literal JSON values in JMESPath
                format!("`{}`", v)
            })
            .collect();

        let expr_str = format!("{}({})", fn_name, args_json.join(", "));

        self.eval_jmespath(&expr_str, ctx)
    }

    fn eval_jmespath(&self, expr: &str, ctx: &Value) -> Result<Value> {
        let compiled = self
            .runtime
            .compile(expr)
            .map_err(|e| JlispError::JmespathError(e.to_string()))?;

        let var = json_to_variable(ctx);

        let result = compiled
            .search(&var)
            .map_err(|e| JlispError::JmespathError(e.to_string()))?;

        Ok(variable_to_json(&result))
    }
}

fn json_to_variable(value: &Value) -> Rcvar {
    Rc::new(match value {
        Value::Null => Variable::Null,
        Value::Bool(b) => Variable::Bool(*b),
        Value::Number(n) => Variable::Number(n.clone()),
        Value::String(s) => Variable::String(s.clone()),
        Value::Array(arr) => Variable::Array(arr.iter().map(json_to_variable).collect()),
        Value::Object(map) => Variable::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), json_to_variable(v)))
                .collect(),
        ),
    })
}

fn variable_to_json(var: &Variable) -> Value {
    match var {
        Variable::Null => Value::Null,
        Variable::Bool(b) => Value::Bool(*b),
        Variable::Number(n) => Value::Number(n.clone()),
        Variable::String(s) => Value::String(s.clone()),
        Variable::Array(arr) => Value::Array(arr.iter().map(|v| variable_to_json(v)).collect()),
        Variable::Object(map) => Value::Object(
            map.iter()
                .map(|(k, v)| (k.clone(), variable_to_json(v)))
                .collect(),
        ),
        Variable::Expref(_) => Value::Null, // Can't represent expression refs in JSON
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_atoms() {
        let mut jlisp = Jlisp::new();
        assert_eq!(jlisp.eval(&json!(42)).unwrap(), json!(42));
        assert_eq!(jlisp.eval(&json!(true)).unwrap(), json!(true));
        assert_eq!(jlisp.eval(&json!(null)).unwrap(), json!(null));
        assert_eq!(jlisp.eval(&json!("hello")).unwrap(), json!("hello"));
    }

    #[test]
    fn test_arithmetic() {
        let mut jlisp = Jlisp::new();
        assert_eq!(jlisp.eval(&json!(["add", 1, 2])).unwrap(), json!(3.0));
        assert_eq!(jlisp.eval(&json!(["multiply", 3, 4])).unwrap(), json!(12.0));
        assert_eq!(jlisp.eval(&json!(["subtract", 10, 3])).unwrap(), json!(7.0));
        assert_eq!(jlisp.eval(&json!(["divide", 20, 4])).unwrap(), json!(5.0));
    }

    #[test]
    fn test_nested() {
        let mut jlisp = Jlisp::new();
        // (2 * 3) + 4 = 10
        let result = jlisp.eval(&json!(["add", ["multiply", 2, 3], 4])).unwrap();
        assert_eq!(result, json!(10.0));
    }

    #[test]
    fn test_if() {
        let mut jlisp = Jlisp::new();
        assert_eq!(
            jlisp.eval(&json!(["if", true, "yes", "no"])).unwrap(),
            json!("yes")
        );
        assert_eq!(
            jlisp.eval(&json!(["if", false, "yes", "no"])).unwrap(),
            json!("no")
        );
        assert_eq!(
            jlisp.eval(&json!(["if", null, "yes", "no"])).unwrap(),
            json!("no")
        );
    }

    #[test]
    fn test_quote() {
        let mut jlisp = Jlisp::new();
        let result = jlisp.eval(&json!(["quote", ["add", 1, 2]])).unwrap();
        assert_eq!(result, json!(["add", 1, 2]));
    }

    #[test]
    fn test_def_variable() {
        let mut jlisp = Jlisp::new();
        jlisp.eval(&json!(["def", "x", 42])).unwrap();
        assert_eq!(jlisp.eval(&json!("x")).unwrap(), json!(42));
    }

    #[test]
    fn test_def_function() {
        let mut jlisp = Jlisp::new();
        // Define double(n) = n * 2
        jlisp
            .eval(&json!(["def", "double", ["n"], ["multiply", "@.n", 2]]))
            .unwrap();
        let result = jlisp.eval(&json!(["double", 5])).unwrap();
        assert_eq!(result, json!(10.0));
    }

    #[test]
    fn test_let() {
        let mut jlisp = Jlisp::new();
        let result = jlisp
            .eval(&json!(["let", {"x": 5, "y": 10}, ["add", "@.x", "@.y"]]))
            .unwrap();
        assert_eq!(result, json!(15.0));
    }

    #[test]
    fn test_do() {
        let mut jlisp = Jlisp::new();
        let result = jlisp
            .eval(&json!([
                "do",
                ["def", "a", 1],
                ["def", "b", 2],
                ["add", "a", "b"]
            ]))
            .unwrap();
        assert_eq!(result, json!(3.0));
    }

    #[test]
    fn test_array_functions() {
        let mut jlisp = Jlisp::new();
        assert_eq!(
            jlisp.eval(&json!(["length", [1, 2, 3, 4, 5]])).unwrap(),
            json!(5)
        );
        assert_eq!(
            jlisp.eval(&json!(["sum", [1, 2, 3, 4, 5]])).unwrap(),
            json!(15.0)
        );
        assert_eq!(
            jlisp.eval(&json!(["avg", [10, 20, 30]])).unwrap(),
            json!(20.0)
        );
    }

    #[test]
    fn test_string_functions() {
        let mut jlisp = Jlisp::new();
        assert_eq!(
            jlisp.eval(&json!(["upper", "hello"])).unwrap(),
            json!("HELLO")
        );
        assert_eq!(
            jlisp.eval(&json!(["lower", "WORLD"])).unwrap(),
            json!("world")
        );
    }

    #[test]
    fn test_jmespath_context() {
        let mut jlisp = Jlisp::new();
        let ctx = json!({"name": "Alice", "age": 30});
        let result = jlisp.eval_with_context(&json!("@.name"), &ctx).unwrap();
        assert_eq!(result, json!("Alice"));
    }

    #[test]
    fn test_jmes_raw() {
        let mut jlisp = Jlisp::new();
        // Raw JMESPath with native let
        let result = jlisp
            .eval(&json!(["$", "let $x = `5` in add($x, `10`)"]))
            .unwrap();
        assert_eq!(result, json!(15.0));

        // Also works with "jmes" alias
        let result = jlisp
            .eval(&json!([
                "jmes",
                "let $a = `2`, $b = `3` in multiply($a, $b)"
            ]))
            .unwrap();
        assert_eq!(result, json!(6.0));
    }

    #[test]
    fn test_let_native() {
        let mut jlisp = Jlisp::new();
        // Native JEP-011 let with object bindings
        let result = jlisp
            .eval(&json!(["let$", {"x": 5, "y": 10}, "add($x, $y)"]))
            .unwrap();
        assert_eq!(result, json!(15.0));

        // With nested JLisp expressions in bindings
        let result = jlisp
            .eval(&json!(["let$", {"doubled": ["multiply", 5, 2]}, "add($doubled, `1`)"]))
            .unwrap();
        assert_eq!(result, json!(11.0));
    }
}
