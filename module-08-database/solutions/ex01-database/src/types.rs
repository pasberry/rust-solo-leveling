use crate::error::{DbError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported data types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Text,
    Boolean,
}

/// A value in the database
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Value {
    Null,
    Boolean(bool),
    Integer(i64),
    Text(String),
}

impl Value {
    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Value::Integer(_) => Some(DataType::Integer),
            Value::Text(_) => Some(DataType::Text),
            Value::Boolean(_) => Some(DataType::Boolean),
            Value::Null => None,
        }
    }

    pub fn as_integer(&self) -> Result<i64> {
        match self {
            Value::Integer(i) => Ok(*i),
            _ => Err(DbError::TypeMismatch {
                expected: "Integer".to_string(),
                actual: format!("{:?}", self),
            }),
        }
    }

    pub fn as_text(&self) -> Result<&str> {
        match self {
            Value::Text(s) => Ok(s),
            _ => Err(DbError::TypeMismatch {
                expected: "Text".to_string(),
                actual: format!("{:?}", self),
            }),
        }
    }

    pub fn as_bool(&self) -> Result<bool> {
        match self {
            Value::Boolean(b) => Ok(*b),
            _ => Err(DbError::TypeMismatch {
                expected: "Boolean".to_string(),
                actual: format!("{:?}", self),
            }),
        }
    }
}

/// Column definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub primary_key: bool,
}

/// Table schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub columns: Vec<Column>,
}

impl Schema {
    pub fn new(name: String, columns: Vec<Column>) -> Self {
        Schema { name, columns }
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    pub fn primary_key_index(&self) -> Option<usize> {
        self.columns.iter().position(|c| c.primary_key)
    }

    pub fn validate_row(&self, row: &Row) -> Result<()> {
        if row.values.len() != self.columns.len() {
            return Err(DbError::ConstraintViolation(format!(
                "Expected {} columns, got {}",
                self.columns.len(),
                row.values.len()
            )));
        }

        for (i, (col, value)) in self.columns.iter().zip(&row.values).enumerate() {
            // Check NULL constraint
            if !col.nullable && matches!(value, Value::Null) {
                return Err(DbError::ConstraintViolation(format!(
                    "Column '{}' cannot be NULL",
                    col.name
                )));
            }

            // Check type match (if not NULL)
            if let Some(value_type) = value.data_type() {
                if value_type != col.data_type {
                    return Err(DbError::TypeMismatch {
                        expected: format!("{:?}", col.data_type),
                        actual: format!("{:?}", value_type),
                    });
                }
            }
        }

        Ok(())
    }
}

/// A row of data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Row { values }
    }

    pub fn get(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    pub fn to_map(&self, schema: &Schema) -> HashMap<String, Value> {
        self.values
            .iter()
            .zip(&schema.columns)
            .map(|(v, c)| (c.name.clone(), v.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_types() {
        let int_val = Value::Integer(42);
        assert_eq!(int_val.as_integer().unwrap(), 42);

        let text_val = Value::Text("hello".to_string());
        assert_eq!(text_val.as_text().unwrap(), "hello");

        let bool_val = Value::Boolean(true);
        assert!(bool_val.as_bool().unwrap());
    }

    #[test]
    fn test_schema_validation() {
        let schema = Schema::new(
            "users".to_string(),
            vec![
                Column {
                    name: "id".to_string(),
                    data_type: DataType::Integer,
                    nullable: false,
                    primary_key: true,
                },
                Column {
                    name: "name".to_string(),
                    data_type: DataType::Text,
                    nullable: false,
                    primary_key: false,
                },
            ],
        );

        // Valid row
        let row = Row::new(vec![Value::Integer(1), Value::Text("Alice".to_string())]);
        assert!(schema.validate_row(&row).is_ok());

        // NULL in non-nullable column
        let row = Row::new(vec![Value::Integer(1), Value::Null]);
        assert!(schema.validate_row(&row).is_err());

        // Type mismatch
        let row = Row::new(vec![Value::Integer(1), Value::Integer(2)]);
        assert!(schema.validate_row(&row).is_err());
    }
}
