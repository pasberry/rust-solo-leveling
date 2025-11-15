# Module 09: Compiler/Interpreter

**Build a Programming Language Interpreter**

## Overview

Build a complete programming language with:
- Lexer (tokenizer)
- Parser (recursive descent or Pratt parsing)
- Abstract Syntax Tree (AST)
- Tree-walking interpreter OR bytecode VM
- Standard library functions
- REPL (Read-Eval-Print Loop)

**Duration**: 3-4 weeks (35-40 hours)

## What You'll Build

```rust
// Example language code your interpreter will run

let fibonacci = fn(n) {
    if n <= 1 {
        return n;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
};

let result = fibonacci(10);
print(result);  // 55

// Array operations
let numbers = [1, 2, 3, 4, 5];
let doubled = map(numbers, fn(x) { x * 2 });
print(doubled);  // [2, 4, 6, 8, 10]

// Hash maps
let person = {
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com"
};
print(person["name"]);  // Alice
```

## Architecture

```
Source Code (text)
        ↓
   ┌────────────┐
   │   Lexer    │ → Tokens
   └────────────┘
        ↓
   ┌────────────┐
   │   Parser   │ → AST (Abstract Syntax Tree)
   └────────────┘
        ↓
   ┌────────────┐
   │ Evaluator  │ → Result
   └────────────┘
        ↓
   Output/Side Effects
```

## Language Design

### Syntax Specification

```
// Comments
let x = 5;  // single-line comment

// Data types
let integer = 42;
let boolean = true;
let string = "Hello, world!";
let array = [1, 2, 3, 4, 5];
let hash = {"key": "value", "count": 42};

// Operators
let sum = 1 + 2;
let product = 3 * 4;
let comparison = 5 > 3;
let logical = true && false;

// Variables
let mutable_var = 10;
mutable_var = 20;

// Functions
let add = fn(a, b) { a + b };
let result = add(5, 10);

// Control flow
if x > 5 {
    print("x is large");
} else {
    print("x is small");
}

// Loops
let i = 0;
while i < 10 {
    print(i);
    i = i + 1;
}

// Higher-order functions
let apply = fn(f, x) { f(x) };
let double = fn(x) { x * 2 };
print(apply(double, 5));  // 10
```

## Key Components

### 1. Lexer (Tokenizer)

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    String(String),
    True,
    False,

    // Identifiers
    Ident(String),

    // Keywords
    Let,
    Fn,
    If,
    Else,
    Return,
    While,

    // Operators
    Assign,
    Plus,
    Minus,
    Star,
    Slash,
    Bang,

    // Comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    // Logical
    And,
    Or,

    // Delimiters
    Comma,
    Semicolon,
    Colon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,

    // Special
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();

        Lexer {
            input: chars,
            position: 0,
            current_char,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        match self.current_char {
            None => Token::Eof,
            Some(ch) => match ch {
                '+' => {
                    self.advance();
                    Token::Plus
                }
                '-' => {
                    self.advance();
                    Token::Minus
                }
                '*' => {
                    self.advance();
                    Token::Star
                }
                '/' => {
                    self.advance();
                    if self.current_char == Some('/') {
                        self.skip_comment();
                        self.next_token()
                    } else {
                        Token::Slash
                    }
                }
                '=' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::Eq
                    } else {
                        Token::Assign
                    }
                }
                '!' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::NotEq
                    } else {
                        Token::Bang
                    }
                }
                '<' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::LtEq
                    } else {
                        Token::Lt
                    }
                }
                '>' => {
                    self.advance();
                    if self.current_char == Some('=') {
                        self.advance();
                        Token::GtEq
                    } else {
                        Token::Gt
                    }
                }
                '&' if self.peek() == Some('&') => {
                    self.advance();
                    self.advance();
                    Token::And
                }
                '|' if self.peek() == Some('|') => {
                    self.advance();
                    self.advance();
                    Token::Or
                }
                '(' => { self.advance(); Token::LParen }
                ')' => { self.advance(); Token::RParen }
                '{' => { self.advance(); Token::LBrace }
                '}' => { self.advance(); Token::RBrace }
                '[' => { self.advance(); Token::LBracket }
                ']' => { self.advance(); Token::RBracket }
                ',' => { self.advance(); Token::Comma }
                ';' => { self.advance(); Token::Semicolon }
                ':' => { self.advance(); Token::Colon }
                '"' => self.read_string(),
                _ if ch.is_ascii_digit() => self.read_number(),
                _ if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
                _ => {
                    self.advance();
                    panic!("Unexpected character: {}", ch);
                }
            }
        }
    }

    fn advance(&mut self) {
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_comment(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == '\n' {
                break;
            }
            self.advance();
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start..self.position].iter().collect();
        Token::Integer(num_str.parse().unwrap())
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let ident: String = self.input[start..self.position].iter().collect();

        match ident.as_str() {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            "while" => Token::While,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Ident(ident),
        }
    }

    fn read_string(&mut self) -> Token {
        self.advance();  // Skip opening quote
        let start = self.position;

        while let Some(ch) = self.current_char {
            if ch == '"' {
                let s: String = self.input[start..self.position].iter().collect();
                self.advance();  // Skip closing quote
                return Token::String(s);
            }
            self.advance();
        }

        panic!("Unterminated string");
    }
}
```

### 2. Parser & AST

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Integer(i64),
    Boolean(bool),
    String(String),
    Identifier(String),
    Array(Vec<Expr>),
    Hash(Vec<(Expr, Expr)>),
    Index {
        left: Box<Expr>,
        index: Box<Expr>,
    },
    Prefix {
        operator: PrefixOp,
        right: Box<Expr>,
    },
    Infix {
        left: Box<Expr>,
        operator: InfixOp,
        right: Box<Expr>,
    },
    If {
        condition: Box<Expr>,
        consequence: Vec<Stmt>,
        alternative: Option<Vec<Stmt>>,
    },
    Function {
        parameters: Vec<String>,
        body: Vec<Stmt>,
    },
    Call {
        function: Box<Expr>,
        arguments: Vec<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Return(Expr),
    Expression(Expr),
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrefixOp {
    Minus,
    Bang,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InfixOp {
    Plus,
    Minus,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    And,
    Or,
}

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while self.current_token != Token::Eof {
            statements.push(self.parse_statement()?);
        }

        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match &self.current_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::While => self.parse_while_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(Token::Let)?;

        let name = match &self.current_token {
            Token::Ident(s) => s.clone(),
            _ => return Err(ParseError::ExpectedIdentifier),
        };
        self.advance();

        self.expect_token(Token::Assign)?;

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.current_token == Token::Semicolon {
            self.advance();
        }

        Ok(Stmt::Let { name, value })
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expr, ParseError> {
        let mut left = self.parse_prefix()?;

        while self.current_token != Token::Semicolon && precedence < self.current_precedence() {
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<Expr, ParseError> {
        match &self.current_token.clone() {
            Token::Integer(n) => {
                let expr = Expr::Integer(*n);
                self.advance();
                Ok(expr)
            }
            Token::True => {
                self.advance();
                Ok(Expr::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Boolean(false))
            }
            Token::String(s) => {
                let expr = Expr::String(s.clone());
                self.advance();
                Ok(expr)
            }
            Token::Ident(name) => {
                let expr = Expr::Identifier(name.clone());
                self.advance();
                Ok(expr)
            }
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::LParen => self.parse_grouped_expression(),
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_hash_literal(),
            Token::If => self.parse_if_expression(),
            Token::Fn => self.parse_function_literal(),
            _ => Err(ParseError::UnexpectedToken(self.current_token.clone())),
        }
    }

    fn parse_infix(&mut self, left: Expr) -> Result<Expr, ParseError> {
        match &self.current_token {
            Token::Plus | Token::Minus | Token::Star | Token::Slash |
            Token::Eq | Token::NotEq | Token::Lt | Token::Gt | Token::And | Token::Or => {
                self.parse_infix_expression(left)
            }
            Token::LParen => self.parse_call_expression(left),
            Token::LBracket => self.parse_index_expression(left),
            _ => Ok(left),
        }
    }

    fn parse_infix_expression(&mut self, left: Expr) -> Result<Expr, ParseError> {
        let operator = self.token_to_infix_op(&self.current_token)?;
        let precedence = self.current_precedence();
        self.advance();

        let right = self.parse_expression(precedence)?;

        Ok(Expr::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_call_expression(&mut self, function: Expr) -> Result<Expr, ParseError> {
        self.expect_token(Token::LParen)?;
        let arguments = self.parse_expression_list(Token::RParen)?;

        Ok(Expr::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_function_literal(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(Token::Fn)?;
        self.expect_token(Token::LParen)?;

        let parameters = self.parse_function_parameters()?;

        self.expect_token(Token::LBrace)?;
        let body = self.parse_block_statement()?;

        Ok(Expr::Function { parameters, body })
    }

    fn parse_if_expression(&mut self) -> Result<Expr, ParseError> {
        self.expect_token(Token::If)?;

        let condition = Box::new(self.parse_expression(Precedence::Lowest)?);

        self.expect_token(Token::LBrace)?;
        let consequence = self.parse_block_statement()?;

        let alternative = if self.current_token == Token::Else {
            self.advance();
            self.expect_token(Token::LBrace)?;
            Some(self.parse_block_statement()?)
        } else {
            None
        };

        Ok(Expr::If { condition, consequence, alternative })
    }

    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError> {
        if self.current_token == expected {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken(self.current_token.clone()))
        }
    }

    fn current_precedence(&self) -> Precedence {
        token_precedence(&self.current_token)
    }

    fn token_to_infix_op(&self, token: &Token) -> Result<InfixOp, ParseError> {
        match token {
            Token::Plus => Ok(InfixOp::Plus),
            Token::Minus => Ok(InfixOp::Minus),
            Token::Star => Ok(InfixOp::Multiply),
            Token::Slash => Ok(InfixOp::Divide),
            Token::Eq => Ok(InfixOp::Equal),
            Token::NotEq => Ok(InfixOp::NotEqual),
            Token::Lt => Ok(InfixOp::LessThan),
            Token::Gt => Ok(InfixOp::GreaterThan),
            Token::And => Ok(InfixOp::And),
            Token::Or => Ok(InfixOp::Or),
            _ => Err(ParseError::InvalidOperator),
        }
    }

    // Additional parsing methods omitted for brevity
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    Lowest,
    LogicalOr,
    LogicalAnd,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Or => Precedence::LogicalOr,
        Token::And => Precedence::LogicalAnd,
        Token::Eq | Token::NotEq => Precedence::Equals,
        Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Precedence::LessGreater,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Star | Token::Slash => Precedence::Product,
        Token::LParen => Precedence::Call,
        Token::LBracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}
```

### 3. Interpreter (Tree-Walking Evaluator)

```rust
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
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
    Null,
}

#[derive(Debug, Clone)]
pub struct Environment {
    store: HashMap<String, Value>,
    outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn with_outer(outer: Environment) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(Box::new(outer)),
        }
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.store.get(name).cloned().or_else(|| {
            self.outer.as_ref().and_then(|env| env.get(name))
        })
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.store.insert(name, value);
    }
}

pub struct Evaluator {
    env: Environment,
}

impl Evaluator {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Add builtin functions
        env.set("print".to_string(), Value::Builtin(builtin_print));
        env.set("len".to_string(), Value::Builtin(builtin_len));
        env.set("map".to_string(), Value::Builtin(builtin_map));

        Evaluator { env }
    }

    pub fn eval_program(&mut self, program: Vec<Stmt>) -> Result<Value, EvalError> {
        let mut result = Value::Null;

        for stmt in program {
            result = self.eval_statement(stmt)?;

            // Handle early return
            if matches!(result, Value::Return(_)) {
                return Ok(result);
            }
        }

        Ok(result)
    }

    fn eval_statement(&mut self, stmt: Stmt) -> Result<Value, EvalError> {
        match stmt {
            Stmt::Let { name, value } => {
                let val = self.eval_expression(value)?;
                self.env.set(name, val.clone());
                Ok(Value::Null)
            }
            Stmt::Return(expr) => {
                let val = self.eval_expression(expr)?;
                Ok(Value::Return(Box::new(val)))
            }
            Stmt::Expression(expr) => self.eval_expression(expr),
            Stmt::While { condition, body } => {
                while self.is_truthy(&self.eval_expression(condition.clone())?) {
                    for stmt in &body {
                        self.eval_statement(stmt.clone())?;
                    }
                }
                Ok(Value::Null)
            }
        }
    }

    fn eval_expression(&mut self, expr: Expr) -> Result<Value, EvalError> {
        match expr {
            Expr::Integer(n) => Ok(Value::Integer(n)),
            Expr::Boolean(b) => Ok(Value::Boolean(b)),
            Expr::String(s) => Ok(Value::String(s)),
            Expr::Identifier(name) => {
                self.env.get(&name).ok_or_else(|| EvalError::UndefinedVariable(name))
            }
            Expr::Array(elements) => {
                let values: Result<Vec<_>, _> = elements
                    .into_iter()
                    .map(|e| self.eval_expression(e))
                    .collect();
                Ok(Value::Array(values?))
            }
            Expr::Prefix { operator, right } => {
                let right_val = self.eval_expression(*right)?;
                self.eval_prefix_expression(operator, right_val)
            }
            Expr::Infix { left, operator, right } => {
                let left_val = self.eval_expression(*left)?;
                let right_val = self.eval_expression(*right)?;
                self.eval_infix_expression(operator, left_val, right_val)
            }
            Expr::If { condition, consequence, alternative } => {
                let cond = self.eval_expression(*condition)?;

                if self.is_truthy(&cond) {
                    self.eval_block_statement(consequence)
                } else if let Some(alt) = alternative {
                    self.eval_block_statement(alt)
                } else {
                    Ok(Value::Null)
                }
            }
            Expr::Function { parameters, body } => {
                Ok(Value::Function {
                    parameters,
                    body,
                    env: self.env.clone(),
                })
            }
            Expr::Call { function, arguments } => {
                let func = self.eval_expression(*function)?;
                let args: Result<Vec<_>, _> = arguments
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
            _ => todo!("Implement remaining expressions"),
        }
    }

    fn eval_infix_expression(
        &self,
        operator: InfixOp,
        left: Value,
        right: Value,
    ) -> Result<Value, EvalError> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => {
                Ok(match operator {
                    InfixOp::Plus => Value::Integer(l + r),
                    InfixOp::Minus => Value::Integer(l - r),
                    InfixOp::Multiply => Value::Integer(l * r),
                    InfixOp::Divide => Value::Integer(l / r),
                    InfixOp::Equal => Value::Boolean(l == r),
                    InfixOp::NotEqual => Value::Boolean(l != r),
                    InfixOp::LessThan => Value::Boolean(l < r),
                    InfixOp::GreaterThan => Value::Boolean(l > r),
                    _ => return Err(EvalError::InvalidOperation),
                })
            }
            (Value::Boolean(l), Value::Boolean(r)) => {
                Ok(match operator {
                    InfixOp::And => Value::Boolean(l && r),
                    InfixOp::Or => Value::Boolean(l || r),
                    InfixOp::Equal => Value::Boolean(l == r),
                    InfixOp::NotEqual => Value::Boolean(l != r),
                    _ => return Err(EvalError::InvalidOperation),
                })
            }
            (Value::String(l), Value::String(r)) => {
                Ok(match operator {
                    InfixOp::Plus => Value::String(format!("{}{}", l, r)),
                    InfixOp::Equal => Value::Boolean(l == r),
                    InfixOp::NotEqual => Value::Boolean(l != r),
                    _ => return Err(EvalError::InvalidOperation),
                })
            }
            _ => Err(EvalError::TypeMismatch),
        }
    }

    fn apply_function(&mut self, func: Value, args: Vec<Value>) -> Result<Value, EvalError> {
        match func {
            Value::Function { parameters, body, env } => {
                if parameters.len() != args.len() {
                    return Err(EvalError::WrongArgumentCount);
                }

                // Create new environment with function's closure
                let mut extended_env = Environment::with_outer(env);

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

    fn eval_block_statement(&mut self, stmts: Vec<Stmt>) -> Result<Value, EvalError> {
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

type BuiltinFn = fn(Vec<Value>) -> Result<Value, EvalError>;

fn builtin_print(args: Vec<Value>) -> Result<Value, EvalError> {
    for arg in args {
        println!("{}", value_to_string(&arg));
    }
    Ok(Value::Null)
}

fn builtin_len(args: Vec<Value>) -> Result<Value, EvalError> {
    if args.len() != 1 {
        return Err(EvalError::WrongArgumentCount);
    }

    match &args[0] {
        Value::String(s) => Ok(Value::Integer(s.len() as i64)),
        Value::Array(arr) => Ok(Value::Integer(arr.len() as i64)),
        _ => Err(EvalError::InvalidOperation),
    }
}

fn builtin_map(args: Vec<Value>) -> Result<Value, EvalError> {
    // Implementation of map function
    todo!()
}
```

### 4. REPL

```rust
use std::io::{self, Write};

pub fn run_repl() {
    let mut evaluator = Evaluator::new();

    println!("Welcome to the Monkey REPL!");
    println!("Type 'exit' to quit");

    loop {
        print!(">> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let input = input.trim();
        if input == "exit" {
            break;
        }

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        match parser.parse_program() {
            Ok(program) => {
                match evaluator.eval_program(program) {
                    Ok(value) => {
                        if !matches!(value, Value::Null) {
                            println!("{}", value_to_string(&value));
                        }
                    }
                    Err(e) => println!("Error: {:?}", e),
                }
            }
            Err(e) => println!("Parse error: {:?}", e),
        }
    }
}
```

## Implementation Roadmap

### Phase 1: Lexer (Days 1-3)
- Tokenize basic literals (integers, booleans, strings)
- Handle keywords and identifiers
- Operators and delimiters
- Comprehensive lexer tests

**Success criteria:**
- Tokenizes all language constructs
- Good error messages for invalid characters
- Handles edge cases (empty input, long strings)

### Phase 2: Parser & AST (Days 4-10)
- Define AST data structures
- Implement Pratt parser for expressions
- Parse statements (let, return, if, while)
- Parse function definitions and calls

**Success criteria:**
- Parses valid programs into AST
- Precedence and associativity correct
- Good error messages

### Phase 3: Basic Evaluator (Days 11-16)
- Evaluate literals and identifiers
- Arithmetic and comparison operators
- Variable bindings
- Control flow (if/else)

**Success criteria:**
- Can run simple programs
- Variables work correctly
- Conditionals evaluate properly

### Phase 4: Functions & Closures (Days 17-22)
- Function definitions and calls
- Lexical scoping
- Closures capture environment
- Recursion support

**Success criteria:**
- First-class functions work
- Closures capture variables
- Recursive functions (fibonacci, factorial)

### Phase 5: Data Structures (Days 23-27)
- Arrays with indexing
- Hash maps
- Built-in functions (len, map, filter)

**Success criteria:**
- Arrays and hashmaps work
- Standard library functions
- Higher-order functions

### Phase 6: REPL & Tooling (Days 28-32)
- Interactive REPL
- File execution mode
- Pretty-print AST
- Basic error messages

**Success criteria:**
- Usable interactive experience
- Can run script files
- Helpful errors

### Phase 7: Advanced Features (Days 33-40)
- While/for loops
- Break/continue statements
- String interpolation
- Module system (import/export)

## Performance Targets

- **Lexing**: >1MB/sec
- **Parsing**: >500KB/sec
- **Evaluation**: >10k ops/sec (simple arithmetic)
- **Startup time**: <50ms

## Success Criteria

- ✅ Complete lexer with all tokens
- ✅ Parser produces valid AST
- ✅ Tree-walking interpreter works
- ✅ Functions and closures
- ✅ Arrays and hash maps
- ✅ Standard library functions
- ✅ Interactive REPL
- ✅ Can run fibonacci, fizzbuzz, etc.

## Comparison with Production Languages

**What you'll implement:**
- Lexer and parser
- Tree-walking interpreter
- First-class functions
- Closures
- Basic data structures
- REPL

**What production languages have:**
- Bytecode compilation
- JIT compilation
- Garbage collection
- Module system
- Large standard library
- Type systems

**Learning focus:**
- Parsing techniques (Pratt parsing)
- AST design
- Evaluation strategies
- Environment and closures
- Language design decisions

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arithmetic() {
        let input = "5 + 10 * 2";
        let result = eval(input).unwrap();
        assert_eq!(result, Value::Integer(25));
    }

    #[test]
    fn test_function_call() {
        let input = "
            let add = fn(a, b) { a + b };
            add(5, 10)
        ";
        let result = eval(input).unwrap();
        assert_eq!(result, Value::Integer(15));
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
        let result = eval(input).unwrap();
        assert_eq!(result, Value::Integer(5));
    }

    #[test]
    fn test_fibonacci() {
        let input = "
            let fib = fn(n) {
                if n <= 1 {
                    return n;
                } else {
                    return fib(n - 1) + fib(n - 2);
                }
            };
            fib(10)
        ";
        let result = eval(input).unwrap();
        assert_eq!(result, Value::Integer(55));
    }
}
```

## Extensions & Variations

**After completing the core interpreter, try:**

1. **Bytecode VM**:
   - Compile to bytecode
   - Stack-based VM
   - Compare performance

2. **Type system**:
   - Static type checking
   - Type inference
   - Generics

3. **More features**:
   - Classes/objects
   - Exceptions
   - Async/await
   - Pattern matching

4. **Optimizations**:
   - Constant folding
   - Dead code elimination
   - Inline functions

5. **Tooling**:
   - Syntax highlighting
   - LSP server
   - Debugger

## Resources

**Books:**
- "Writing an Interpreter in Go" by Thorsten Ball (this module follows it closely)
- "Crafting Interpreters" by Robert Nystrom
- "Modern Compiler Implementation in ML" (Tiger book)

**Rust Crates:**
- `nom` - Parser combinators (alternative to hand-written parser)
- `logos` - Fast lexer generator
- `codespan-reporting` - Beautiful error messages

**Theory:**
- Pratt parsing explained
- Recursive descent parsing
- Abstract syntax trees
- Operational semantics

**Similar Projects:**
- The Monkey language (from "Writing an Interpreter in Go")
- Lox (from "Crafting Interpreters")
- Rhai (Rust scripting language)

## Next Module

[Module 10: Stock Trading System (Capstone) →](../module-10-trading-system/)
