use crate::error::{DbError, Result};
use crate::types::{Column, DataType, Value};

#[derive(Debug, PartialEq)]
pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        values: Vec<Value>,
    },
    Select {
        table: String,
        columns: Vec<String>, // "*" for all
        where_clause: Option<WhereClause>,
    },
}

#[derive(Debug, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub nullable: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhereClause {
    pub column: String,
    pub operator: Operator,
    pub value: Value,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
}

/// Simple SQL parser (hand-written, no parser generator)
pub struct Parser {
    tokens: Vec<String>,
    pos: usize,
}

impl Parser {
    pub fn new(sql: &str) -> Self {
        let tokens = tokenize(sql);
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Result<Statement> {
        let first = self.current().ok_or_else(|| DbError::ParseError("Empty query".to_string()))?;

        match first.to_uppercase().as_str() {
            "CREATE" => self.parse_create_table(),
            "INSERT" => self.parse_insert(),
            "SELECT" => self.parse_select(),
            _ => Err(DbError::ParseError(format!("Unknown statement: {}", first))),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement> {
        self.expect("CREATE")?;
        self.expect("TABLE")?;

        let name = self.consume()?.to_string();

        self.expect("(")?;

        let mut columns = Vec::new();

        loop {
            let col_name = self.consume()?.to_string();
            let type_str = self.consume()?.to_uppercase();

            let data_type = match type_str.as_str() {
                "INTEGER" | "INT" => DataType::Integer,
                "TEXT" | "VARCHAR" => DataType::Text,
                "BOOLEAN" | "BOOL" => DataType::Boolean,
                _ => return Err(DbError::ParseError(format!("Unknown type: {}", type_str))),
            };

            let mut primary_key = false;
            let mut nullable = true;

            // Check for PRIMARY KEY or NOT NULL
            while let Some(token) = self.peek() {
                match token.to_uppercase().as_str() {
                    "PRIMARY" => {
                        self.consume()?;
                        self.expect("KEY")?;
                        primary_key = true;
                        nullable = false;
                    }
                    "NOT" => {
                        self.consume()?;
                        self.expect("NULL")?;
                        nullable = false;
                    }
                    "," | ")" => break,
                    _ => break,
                }
            }

            columns.push(ColumnDef {
                name: col_name,
                data_type,
                primary_key,
                nullable,
            });

            if let Some(",") = self.peek().map(|s| s.as_str()) {
                self.consume()?;
            } else {
                break;
            }
        }

        self.expect(")")?;

        Ok(Statement::CreateTable { name, columns })
    }

    fn parse_insert(&mut self) -> Result<Statement> {
        self.expect("INSERT")?;
        self.expect("INTO")?;

        let table = self.consume()?.to_string();

        self.expect("VALUES")?;
        self.expect("(")?;

        let mut values = Vec::new();

        loop {
            let token = self.consume()?;
            let value = parse_value(token)?;
            values.push(value);

            if let Some(",") = self.peek().map(|s| s.as_str()) {
                self.consume()?;
            } else {
                break;
            }
        }

        self.expect(")")?;

        Ok(Statement::Insert { table, values })
    }

    fn parse_select(&mut self) -> Result<Statement> {
        self.expect("SELECT")?;

        let mut columns = Vec::new();

        // Parse column list
        loop {
            let col = self.consume()?.to_string();
            columns.push(col);

            if let Some(",") = self.peek().map(|s| s.as_str()) {
                self.consume()?;
            } else {
                break;
            }
        }

        self.expect("FROM")?;

        let table = self.consume()?.to_string();

        // Optional WHERE clause
        let where_clause = if self.peek().map(|s| s.to_uppercase()) == Some("WHERE".to_string()) {
            self.consume()?; // WHERE

            let column = self.consume()?.to_string();
            let op_str = self.consume()?.to_string();
            let value_str = self.consume()?.to_string();

            let operator = match op_str.as_str() {
                "=" => Operator::Equals,
                "!=" | "<>" => Operator::NotEquals,
                ">" => Operator::GreaterThan,
                "<" => Operator::LessThan,
                _ => return Err(DbError::ParseError(format!("Unknown operator: {}", op_str))),
            };

            let value = parse_value(&value_str)?;

            Some(WhereClause {
                column,
                operator,
                value,
            })
        } else {
            None
        };

        Ok(Statement::Select {
            table,
            columns,
            where_clause,
        })
    }

    fn current(&self) -> Option<&String> {
        self.tokens.get(self.pos)
    }

    fn peek(&self) -> Option<&String> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Result<&String> {
        let token = self
            .tokens
            .get(self.pos)
            .ok_or_else(|| DbError::ParseError("Unexpected end of input".to_string()))?;
        self.pos += 1;
        Ok(token)
    }

    fn expect(&mut self, expected: &str) -> Result<()> {
        let token = self.consume()?;
        if token.to_uppercase() != expected.to_uppercase() {
            return Err(DbError::ParseError(format!(
                "Expected '{}', got '{}'",
                expected, token
            )));
        }
        Ok(())
    }
}

fn tokenize(sql: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_string = false;

    for ch in sql.chars() {
        match ch {
            '\'' if !in_string => {
                in_string = true;
            }
            '\'' if in_string => {
                in_string = false;
                tokens.push(format!("'{}'", current));
                current.clear();
            }
            ' ' | '\t' | '\n' if !in_string => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            '(' | ')' | ',' if !in_string => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
                tokens.push(ch.to_string());
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn parse_value(token: &str) -> Result<Value> {
    if token.starts_with('\'') && token.ends_with('\'') {
        // String literal
        let s = token[1..token.len() - 1].to_string();
        Ok(Value::Text(s))
    } else if token.to_uppercase() == "TRUE" {
        Ok(Value::Boolean(true))
    } else if token.to_uppercase() == "FALSE" {
        Ok(Value::Boolean(false))
    } else if token.to_uppercase() == "NULL" {
        Ok(Value::Null)
    } else if let Ok(i) = token.parse::<i64>() {
        Ok(Value::Integer(i))
    } else {
        Err(DbError::ParseError(format!("Cannot parse value: {}", token)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_table() {
        let sql = "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL, active BOOLEAN)";
        let mut parser = Parser::new(sql);
        let stmt = parser.parse().unwrap();

        match stmt {
            Statement::CreateTable { name, columns } => {
                assert_eq!(name, "users");
                assert_eq!(columns.len(), 3);
                assert_eq!(columns[0].name, "id");
                assert!(columns[0].primary_key);
                assert_eq!(columns[1].name, "name");
                assert!(!columns[1].nullable);
            }
            _ => panic!("Wrong statement type"),
        }
    }

    #[test]
    fn test_parse_insert() {
        let sql = "INSERT INTO users VALUES (1, 'Alice', TRUE)";
        let mut parser = Parser::new(sql);
        let stmt = parser.parse().unwrap();

        match stmt {
            Statement::Insert { table, values } => {
                assert_eq!(table, "users");
                assert_eq!(values.len(), 3);
                assert_eq!(values[0], Value::Integer(1));
                assert_eq!(values[1], Value::Text("Alice".to_string()));
                assert_eq!(values[2], Value::Boolean(true));
            }
            _ => panic!("Wrong statement type"),
        }
    }

    #[test]
    fn test_parse_select() {
        let sql = "SELECT * FROM users";
        let mut parser = Parser::new(sql);
        let stmt = parser.parse().unwrap();

        match stmt {
            Statement::Select {
                table,
                columns,
                where_clause,
            } => {
                assert_eq!(table, "users");
                assert_eq!(columns, vec!["*"]);
                assert!(where_clause.is_none());
            }
            _ => panic!("Wrong statement type"),
        }
    }

    #[test]
    fn test_parse_select_with_where() {
        let sql = "SELECT name FROM users WHERE id = 1";
        let mut parser = Parser::new(sql);
        let stmt = parser.parse().unwrap();

        match stmt {
            Statement::Select {
                where_clause: Some(clause),
                ..
            } => {
                assert_eq!(clause.column, "id");
                assert_eq!(clause.operator, Operator::Equals);
                assert_eq!(clause.value, Value::Integer(1));
            }
            _ => panic!("Wrong statement"),
        }
    }
}
