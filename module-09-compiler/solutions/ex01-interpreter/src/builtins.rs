use crate::error::{EvalError, Result};
use crate::value::Value;

pub fn builtin_print(args: Vec<Value>) -> Result<Value> {
    for arg in args {
        println!("{}", arg);
    }
    Ok(Value::Null)
}

pub fn builtin_len(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgumentCount);
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        _ => Err(EvalError::InvalidOperation),
    }
}

pub fn builtin_first(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgumentCount);
    }

    match &args[0] {
        Value::Array(arr) => Ok(arr.first().cloned().unwrap_or(Value::Null)),
        _ => Err(EvalError::TypeMismatch),
    }
}

pub fn builtin_last(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgumentCount);
    }

    match &args[0] {
        Value::Array(arr) => Ok(arr.last().cloned().unwrap_or(Value::Null)),
        _ => Err(EvalError::TypeMismatch),
    }
}

pub fn builtin_rest(args: Vec<Value>) -> Result<Value> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgumentCount);
    }

    match &args[0] {
        Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(Value::Array(arr[1..].to_vec()))
            }
        }
        _ => Err(EvalError::TypeMismatch),
    }
}

pub fn builtin_push(args: Vec<Value>) -> Result<Value> {
    if args.len() != 2 {
        return Err(EvalError::WrongArgumentCount);
    }

    match &args[0] {
        Value::Array(arr) => {
            let mut new_arr = arr.clone();
            new_arr.push(args[1].clone());
            Ok(Value::Array(new_arr))
        }
        _ => Err(EvalError::TypeMismatch),
    }
}
