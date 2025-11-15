use crate::ast::*;
use crate::builtins;
use crate::env::Environment;
use crate::error::{EvalError, Result};
use crate::value::Value;
use std::collections::HashMap;

pub struct Evaluator {
    env: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Add builtin functions
        env.set("print".to_string(), Value::Builtin(builtins::builtin_print));
        env.set("len".to_string(), Value::Builtin(builtins::builtin_len));
        env.set("first".to_string(), Value::Builtin(builtins::builtin_first));
        env.set("last".to_string(), Value::Builtin(builtins::builtin_last));
        env.set("rest".to_string(), Value::Builtin(builtins::builtin_rest));
        env.set("push".to_string(), Value::Builtin(builtins::builtin_push));

        Evaluator { env }
    }

    pub fn eval_program(&mut self, program: Vec<Stmt>) -> Result<Value> {
        let mut result = Value::Null;

        for stmt in program {
            result = self.eval_statement(stmt)?;

            // Handle early return
            if let Value::Return(val) = result {
                return Ok(*val);
            }
        }

        Ok(result)
    }

    fn eval_statement(&mut self, stmt: Stmt) -> Result<Value> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.eval_expression(value)?;
                self.env.set(name, val);
                Ok(Value::Null)
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expression(value)?;
                self.env.set(name, val);
                Ok(Value::Null)
            }
            Stmt::Return(expr) => {
                let val = self.eval_expression(expr)?;
                Ok(Value::Return(Box::new(val)))
            }
            Stmt::Expression(expr) => self.eval_expression(expr),
            Stmt::While { condition, body } => {
                loop {
                    let cond_val = self.eval_expression(condition.clone())?;
                    if !self.is_truthy(&cond_val) {
                        break;
                    }

                    for stmt in &body {
                        let result = self.eval_statement(stmt.clone())?;
                        // Handle return in loop
                        if matches!(result, Value::Return(_)) {
                            return Ok(result);
                        }
                    }
                }
                Ok(Value::Null)
            }
        }
    }

    fn eval_expression(&mut self, expr: Expr) -> Result<Value> {
        match expr {
            Expr::Integer(n) => Ok(Value::Integer(n)),
            Expr::Boolean(b) => Ok(Value::Boolean(b)),
            Expr::String(s) => Ok(Value::String(s)),
            Expr::Identifier(name) => self
                .env
                .get(&name)
                .ok_or_else(|| EvalError::UndefinedVariable(name)),
            Expr::Array(elements) => {
                let values: Result<Vec<_>> = elements
                    .into_iter()
                    .map(|e| self.eval_expression(e))
                    .collect();
                Ok(Value::Array(values?))
            }
            Expr::Hash(pairs) => {
                let mut map = HashMap::new();
                for (key_expr, value_expr) in pairs {
                    let key_val = self.eval_expression(key_expr)?;
                    let key = key_val.to_hash_key()?;
                    let value = self.eval_expression(value_expr)?;
                    map.insert(key, value);
                }
                Ok(Value::Hash(map))
            }
            Expr::Prefix { operator, right } => {
                let right_val = self.eval_expression(*right)?;
                self.eval_prefix_expression(operator, right_val)
            }
            Expr::Infix {
                left,
                operator,
                right,
            } => {
                let left_val = self.eval_expression(*left)?;
                let right_val = self.eval_expression(*right)?;
                self.eval_infix_expression(operator, left_val, right_val)
            }
            Expr::If {
                condition,
                consequence,
                alternative,
            } => {
                let cond = self.eval_expression(*condition)?;

                if self.is_truthy(&cond) {
                    self.eval_block_statement(consequence)
                } else if let Some(alt) = alternative {
                    self.eval_block_statement(alt)
                } else {
                    Ok(Value::Null)
                }
            }
            Expr::Function { parameters, body } => Ok(Value::Function {
                parameters,
                body,
                env: self.env.clone(),
            }),
            Expr::Call {
                function,
                arguments,
            } => {
                let func = self.eval_expression(*function)?;
                let args: Result<Vec<_>> = arguments
                    .into_iter()
                    .map(|a| self.eval_expression(a))
                    .collect();
                self.apply_function(func, args?)
            }
            Expr::Index { left, index } => {
                let left_val = self.eval_expression(*left)?;
                let index_val = self.eval_expression(*index)?;
                self.eval_index_expression(left_val, index_val)
            }
        }
    }

    fn eval_prefix_expression(&self, operator: PrefixOp, right: Value) -> Result<Value> {
        match operator {
            PrefixOp::Bang => Ok(Value::Boolean(!self.is_truthy(&right))),
            PrefixOp::Minus => match right {
                Value::Integer(n) => Ok(Value::Integer(-n)),
                _ => Err(EvalError::TypeMismatch),
            },
        }
    }

    fn eval_infix_expression(
        &self,
        operator: InfixOp,
        left: Value,
        right: Value,
    ) -> Result<Value> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => match operator {
                InfixOp::Plus => Ok(Value::Integer(l + r)),
                InfixOp::Minus => Ok(Value::Integer(l - r)),
                InfixOp::Multiply => Ok(Value::Integer(l * r)),
                InfixOp::Divide => {
                    if r == 0 {
                        Err(EvalError::DivisionByZero)
                    } else {
                        Ok(Value::Integer(l / r))
                    }
                }
                InfixOp::Equal => Ok(Value::Boolean(l == r)),
                InfixOp::NotEqual => Ok(Value::Boolean(l != r)),
                InfixOp::LessThan => Ok(Value::Boolean(l < r)),
                InfixOp::GreaterThan => Ok(Value::Boolean(l > r)),
                InfixOp::LessThanEqual => Ok(Value::Boolean(l <= r)),
                InfixOp::GreaterThanEqual => Ok(Value::Boolean(l >= r)),
                _ => Err(EvalError::InvalidOperation),
            },
            (Value::Boolean(l), Value::Boolean(r)) => match operator {
                InfixOp::And => Ok(Value::Boolean(l && r)),
                InfixOp::Or => Ok(Value::Boolean(l || r)),
                InfixOp::Equal => Ok(Value::Boolean(l == r)),
                InfixOp::NotEqual => Ok(Value::Boolean(l != r)),
                _ => Err(EvalError::InvalidOperation),
            },
            (Value::String(l), Value::String(r)) => match operator {
                InfixOp::Plus => Ok(Value::String(format!("{}{}", l, r))),
                InfixOp::Equal => Ok(Value::Boolean(l == r)),
                InfixOp::NotEqual => Ok(Value::Boolean(l != r)),
                _ => Err(EvalError::InvalidOperation),
            },
            _ => Err(EvalError::TypeMismatch),
        }
    }

    fn eval_index_expression(&self, left: Value, index: Value) -> Result<Value> {
        match (left, index) {
            (Value::Array(arr), Value::Integer(idx)) => {
                if idx < 0 {
                    return Err(EvalError::IndexOutOfBounds);
                }
                arr.get(idx as usize)
                    .cloned()
                    .ok_or(EvalError::IndexOutOfBounds)
            }
            (Value::Hash(map), index_val) => {
                let key = index_val.to_hash_key()?;
                Ok(map.get(&key).cloned().unwrap_or(Value::Null))
            }
            _ => Err(EvalError::TypeMismatch),
        }
    }

    fn apply_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value> {
        match func {
            Value::Function {
                parameters,
                body,
                env,
            } => {
                if parameters.len() != args.len() {
                    return Err(EvalError::WrongArgumentCount);
                }

                // Create environment chain: params -> closure -> calling env
                // This allows recursive functions to find themselves in the calling environment
                let closure_with_caller = Environment::with_outer(self.env.clone());
                let mut extended_env = Environment::with_outer(closure_with_caller);

                // Also include the function's original closure
                for (key, val) in env.store.iter() {
                    extended_env.set(key.clone(), val.clone());
                }

                // Bind arguments to parameters
                for (param, arg) in parameters.iter().zip(args.iter()) {
                    extended_env.set(param.clone(), arg.clone());
                }

                // Evaluate function body with new environment
                let prev_env = std::mem::replace(&mut self.env, extended_env);
                let result = self.eval_block_statement(body);
                self.env = prev_env;

                // Unwrap return value
                match result? {
                    Value::Return(val) => Ok(*val),
                    val => Ok(val),
                }
            }
            Value::Builtin(func) => func(args),
            _ => Err(EvalError::NotAFunction),
        }
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Null => false,
            Value::Boolean(false) => false,
            _ => true,
        }
    }

    fn eval_block_statement(&mut self, stmts: Vec<Stmt>) -> Result<Value> {
        let mut result = Value::Null;

        for stmt in stmts {
            result = self.eval_statement(stmt)?;

            if matches!(result, Value::Return(_)) {
                return Ok(result);
            }
        }

        Ok(result)
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn eval(input: &str) -> Result<Value> {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program().unwrap();
        let mut evaluator = Evaluator::new();
        evaluator.eval_program(program)
    }

    #[test]
    fn test_integer_arithmetic() {
        assert_eq!(eval("5 + 10 * 2").unwrap(), Value::Integer(25));
        assert_eq!(eval("(5 + 10) * 2").unwrap(), Value::Integer(30));
        assert_eq!(eval("20 - 5 * 2").unwrap(), Value::Integer(10));
    }

    #[test]
    fn test_boolean_logic() {
        assert_eq!(eval("true && false").unwrap(), Value::Boolean(false));
        assert_eq!(eval("true || false").unwrap(), Value::Boolean(true));
        assert_eq!(eval("!true").unwrap(), Value::Boolean(false));
    }

    #[test]
    fn test_comparison() {
        assert_eq!(eval("5 > 3").unwrap(), Value::Boolean(true));
        assert_eq!(eval("5 < 3").unwrap(), Value::Boolean(false));
        assert_eq!(eval("5 == 5").unwrap(), Value::Boolean(true));
        assert_eq!(eval("5 != 3").unwrap(), Value::Boolean(true));
    }

    #[test]
    fn test_let_binding() {
        assert_eq!(eval("let x = 5; x").unwrap(), Value::Integer(5));
        assert_eq!(eval("let x = 5; let y = 10; x + y").unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_if_expression() {
        assert_eq!(eval("if (true) { 10 }").unwrap(), Value::Integer(10));
        assert_eq!(eval("if (false) { 10 } else { 20 }").unwrap(), Value::Integer(20));
    }

    #[test]
    fn test_function_call() {
        let input = "let add = fn(a, b) { a + b }; add(5, 10)";
        assert_eq!(eval(input).unwrap(), Value::Integer(15));
    }

    #[test]
    fn test_closure() {
        let input = "
            let newAdder = fn(x) {
                fn(y) { x + y }
            };
            let addTwo = newAdder(2);
            addTwo(3)
        ";
        assert_eq!(eval(input).unwrap(), Value::Integer(5));
    }

    #[test]
    fn test_fibonacci() {
        let input = "
            let fib = fn(n) {
                if (n <= 1) {
                    return n;
                } else {
                    return fib(n - 1) + fib(n - 2);
                }
            };
            fib(10)
        ";
        assert_eq!(eval(input).unwrap(), Value::Integer(55));
    }

    #[test]
    fn test_array() {
        assert_eq!(
            eval("[1, 2, 3][0]").unwrap(),
            Value::Integer(1)
        );
        assert_eq!(
            eval("[1, 2, 3][2]").unwrap(),
            Value::Integer(3)
        );
    }

    #[test]
    fn test_hash() {
        let input = r#"
            let person = {"name": "Alice", "age": 30};
            person["name"]
        "#;
        assert_eq!(eval(input).unwrap(), Value::String("Alice".to_string()));
    }

    #[test]
    fn test_while_loop() {
        let input = "
            let i = 0;
            let sum = 0;
            while (i < 5) {
                sum = sum + i;
                i = i + 1;
            }
            sum
        ";
        assert_eq!(eval(input).unwrap(), Value::Integer(10));
    }
}
