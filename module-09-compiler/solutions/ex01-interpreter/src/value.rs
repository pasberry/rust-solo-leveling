use crate::ast::Stmt;
use crate::env::Environment;
use crate::error::{EvalError, Result};
use std::collections::HashMap;
use std::fmt;

pub type BuiltinFn = fn(Vec<Value>) -> Result<Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    String(String),
    Array(Vec<Value>),
    Hash(HashMap<HashKey, Value>),
    Function {
        parameters: Vec<String>,
        body: Vec<Stmt>,
        env: Environment,
    },
    Builtin(BuiltinFn),
    Return(Box<Value>),
    Null,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Hash(a), Value::Hash(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Value {
    pub fn to_hash_key(&self) -> Result<HashKey> {
        match self {
            Value::Integer(n) => Ok(HashKey::Integer(*n)),
            Value::Boolean(b) => Ok(HashKey::Boolean(*b)),
            Value::String(s) => Ok(HashKey::String(s.clone())),
            _ => Err(EvalError::TypeMismatch),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            Value::Hash(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| {
                        let key_str = match k {
                            HashKey::Integer(n) => n.to_string(),
                            HashKey::Boolean(b) => b.to_string(),
                            HashKey::String(s) => format!("\"{}\"", s),
                        };
                        format!("{}: {}", key_str, v)
                    })
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            Value::Function { .. } => write!(f, "[function]"),
            Value::Builtin(_) => write!(f, "[builtin function]"),
            Value::Return(val) => write!(f, "{}", val),
            Value::Null => write!(f, "null"),
        }
    }
}
