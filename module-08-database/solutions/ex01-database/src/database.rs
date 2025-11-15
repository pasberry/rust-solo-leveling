use crate::error::{DbError, Result};
use crate::parser::{Operator, Parser, Statement};
use crate::table::Table;
use crate::types::{Column, Row, Schema, Value};
use std::collections::HashMap;

/// Query result
#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

impl QueryResult {
    pub fn empty() -> Self {
        QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
        }
    }

    pub fn rows_affected(count: usize) -> Self {
        QueryResult {
            columns: vec!["rows_affected".to_string()],
            rows: vec![vec![Value::Integer(count as i64)]],
        }
    }
}

/// Main database
pub struct Database {
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            tables: HashMap::new(),
        }
    }

    /// Execute a SQL statement
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        let mut parser = Parser::new(sql);
        let statement = parser.parse()?;

        match statement {
            Statement::CreateTable { name, columns } => self.create_table(name, columns),
            Statement::Insert { table, values } => self.insert(table, values),
            Statement::Select {
                table,
                columns,
                where_clause,
            } => self.select(table, columns, where_clause),
        }
    }

    fn create_table(&mut self, name: String, column_defs: Vec<crate::parser::ColumnDef>) -> Result<QueryResult> {
        if self.tables.contains_key(&name) {
            return Err(DbError::TableAlreadyExists(name));
        }

        let columns: Vec<Column> = column_defs
            .into_iter()
            .map(|def| Column {
                name: def.name,
                data_type: def.data_type,
                nullable: def.nullable,
                primary_key: def.primary_key,
            })
            .collect();

        let schema = Schema::new(name.clone(), columns);
        let table = Table::new(schema);

        self.tables.insert(name, table);

        Ok(QueryResult::empty())
    }

    fn insert(&mut self, table_name: String, values: Vec<Value>) -> Result<QueryResult> {
        let table = self
            .tables
            .get_mut(&table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.clone()))?;

        let row = Row::new(values);
        table.insert(row)?;

        Ok(QueryResult::rows_affected(1))
    }

    fn select(
        &self,
        table_name: String,
        column_names: Vec<String>,
        where_clause: Option<crate::parser::WhereClause>,
    ) -> Result<QueryResult> {
        let table = self
            .tables
            .get(&table_name)
            .ok_or_else(|| DbError::TableNotFound(table_name.clone()))?;

        // Get rows (with optional filter)
        let rows: Vec<&Row> = if let Some(clause) = where_clause {
            let col_index = table
                .schema
                .column_index(&clause.column)
                .ok_or_else(|| DbError::ColumnNotFound(clause.column.clone()))?;

            table.scan_where(|row| {
                if let Some(value) = row.get(col_index) {
                    matches_predicate(value, &clause.operator, &clause.value)
                } else {
                    false
                }
            })
        } else {
            table.scan()
        };

        // Determine columns to return
        let columns = if column_names.len() == 1 && column_names[0] == "*" {
            table.schema.columns.iter().map(|c| c.name.clone()).collect()
        } else {
            column_names.clone()
        };

        // Extract column indices
        let column_indices: Result<Vec<usize>> = columns
            .iter()
            .map(|name| {
                table
                    .schema
                    .column_index(name)
                    .ok_or_else(|| DbError::ColumnNotFound(name.clone()))
            })
            .collect();

        let column_indices = column_indices?;

        // Build result
        let result_rows: Vec<Vec<Value>> = rows
            .into_iter()
            .map(|row| {
                column_indices
                    .iter()
                    .map(|&idx| row.values[idx].clone())
                    .collect()
            })
            .collect();

        Ok(QueryResult {
            columns,
            rows: result_rows,
        })
    }

    /// List all tables
    pub fn list_tables(&self) -> Vec<&str> {
        self.tables.keys().map(|s| s.as_str()).collect()
    }

    /// Get table
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }
}

fn matches_predicate(value: &Value, operator: &Operator, target: &Value) -> bool {
    match operator {
        Operator::Equals => value == target,
        Operator::NotEquals => value != target,
        Operator::GreaterThan => {
            if let (Value::Integer(a), Value::Integer(b)) = (value, target) {
                a > b
            } else {
                false
            }
        }
        Operator::LessThan => {
            if let (Value::Integer(a), Value::Integer(b)) = (value, target) {
                a < b
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_table() {
        let mut db = Database::new();

        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .unwrap();

        assert_eq!(db.list_tables().len(), 1);
        assert!(db.get_table("users").is_some());
    }

    #[test]
    fn test_insert_and_select() {
        let mut db = Database::new();

        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, active BOOLEAN)")
            .unwrap();

        db.execute("INSERT INTO users VALUES (1, 'Alice', TRUE)")
            .unwrap();
        db.execute("INSERT INTO users VALUES (2, 'Bob', FALSE)")
            .unwrap();

        let result = db.execute("SELECT * FROM users").unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.columns, vec!["id", "name", "active"]);
    }

    #[test]
    fn test_select_with_where() {
        let mut db = Database::new();

        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
            .unwrap();

        db.execute("INSERT INTO users VALUES (1, 'Alice', 25)")
            .unwrap();
        db.execute("INSERT INTO users VALUES (2, 'Bob', 30)")
            .unwrap();
        db.execute("INSERT INTO users VALUES (3, 'Charlie', 35)")
            .unwrap();

        let result = db.execute("SELECT name FROM users WHERE age > 28").unwrap();

        assert_eq!(result.rows.len(), 2); // Bob and Charlie
        assert_eq!(result.columns, vec!["name"]);
    }

    #[test]
    fn test_select_specific_columns() {
        let mut db = Database::new();

        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER)")
            .unwrap();

        db.execute("INSERT INTO users VALUES (1, 'Alice', 25)")
            .unwrap();

        let result = db.execute("SELECT name, age FROM users").unwrap();

        assert_eq!(result.columns, vec!["name", "age"]);
        assert_eq!(result.rows[0].len(), 2);
    }

    #[test]
    fn test_duplicate_primary_key() {
        let mut db = Database::new();

        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .unwrap();

        db.execute("INSERT INTO users VALUES (1, 'Alice')")
            .unwrap();

        let result = db.execute("INSERT INTO users VALUES (1, 'Bob')");

        assert!(matches!(result, Err(DbError::ConstraintViolation(_))));
    }
}
