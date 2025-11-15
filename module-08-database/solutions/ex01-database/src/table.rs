use crate::error::{DbError, Result};
use crate::types::{Row, Schema, Value};
use std::collections::BTreeMap;

/// A table storing rows with a B-tree index on the primary key
pub struct Table {
    pub schema: Schema,
    // Using BTreeMap as a simplified B+ tree index (pk_value -> row_id)
    primary_index: BTreeMap<Value, usize>,
    // Actual row storage (row_id -> row)
    rows: Vec<Option<Row>>,
    // Next row ID
    next_id: usize,
}

impl Table {
    pub fn new(schema: Schema) -> Self {
        Table {
            schema,
            primary_index: BTreeMap::new(),
            rows: Vec::new(),
            next_id: 0,
        }
    }

    /// Insert a row
    pub fn insert(&mut self, row: Row) -> Result<usize> {
        // Validate row
        self.schema.validate_row(&row)?;

        // Get primary key value
        let pk_index = self
            .schema
            .primary_key_index()
            .ok_or_else(|| DbError::ConstraintViolation("No primary key defined".to_string()))?;

        let pk_value = row.values[pk_index].clone();

        // Check for duplicate primary key
        if self.primary_index.contains_key(&pk_value) {
            return Err(DbError::ConstraintViolation(format!(
                "Duplicate primary key: {:?}",
                pk_value
            )));
        }

        // Assign row ID
        let row_id = self.next_id;
        self.next_id += 1;

        // Insert into index
        self.primary_index.insert(pk_value, row_id);

        // Insert row
        if row_id >= self.rows.len() {
            self.rows.resize(row_id + 1, None);
        }
        self.rows[row_id] = Some(row);

        Ok(row_id)
    }

    /// Get a row by primary key
    pub fn get_by_pk(&self, pk: &Value) -> Option<&Row> {
        self.primary_index
            .get(pk)
            .and_then(|&row_id| self.rows.get(row_id))
            .and_then(|row| row.as_ref())
    }

    /// Delete a row by primary key
    pub fn delete_by_pk(&mut self, pk: &Value) -> Result<bool> {
        if let Some(&row_id) = self.primary_index.get(pk) {
            self.primary_index.remove(pk);
            if row_id < self.rows.len() {
                self.rows[row_id] = None;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Scan all rows
    pub fn scan(&self) -> Vec<&Row> {
        self.rows.iter().filter_map(|r| r.as_ref()).collect()
    }

    /// Scan rows matching a predicate
    pub fn scan_where<F>(&self, predicate: F) -> Vec<&Row>
    where
        F: Fn(&Row) -> bool,
    {
        self.rows
            .iter()
            .filter_map(|r| r.as_ref())
            .filter(|row| predicate(row))
            .collect()
    }

    /// Get table stats
    pub fn stats(&self) -> TableStats {
        let active_rows = self.rows.iter().filter(|r| r.is_some()).count();

        TableStats {
            total_rows: active_rows,
            index_entries: self.primary_index.len(),
            schema_columns: self.schema.columns.len(),
        }
    }
}

#[derive(Debug)]
pub struct TableStats {
    pub total_rows: usize,
    pub index_entries: usize,
    pub schema_columns: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Column, DataType};

    fn create_test_schema() -> Schema {
        Schema::new(
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
                Column {
                    name: "active".to_string(),
                    data_type: DataType::Boolean,
                    nullable: false,
                    primary_key: false,
                },
            ],
        )
    }

    #[test]
    fn test_insert_and_get() {
        let mut table = Table::new(create_test_schema());

        let row = Row::new(vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Boolean(true),
        ]);

        table.insert(row.clone()).unwrap();

        let retrieved = table.get_by_pk(&Value::Integer(1)).unwrap();
        assert_eq!(retrieved.values, row.values);
    }

    #[test]
    fn test_duplicate_primary_key() {
        let mut table = Table::new(create_test_schema());

        let row1 = Row::new(vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Boolean(true),
        ]);

        let row2 = Row::new(vec![
            Value::Integer(1),
            Value::Text("Bob".to_string()),
            Value::Boolean(false),
        ]);

        table.insert(row1).unwrap();
        let result = table.insert(row2);

        assert!(matches!(result, Err(DbError::ConstraintViolation(_))));
    }

    #[test]
    fn test_scan() {
        let mut table = Table::new(create_test_schema());

        for i in 1..=5 {
            table
                .insert(Row::new(vec![
                    Value::Integer(i),
                    Value::Text(format!("User{}", i)),
                    Value::Boolean(i % 2 == 0),
                ]))
                .unwrap();
        }

        let all = table.scan();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_scan_where() {
        let mut table = Table::new(create_test_schema());

        for i in 1..=5 {
            table
                .insert(Row::new(vec![
                    Value::Integer(i),
                    Value::Text(format!("User{}", i)),
                    Value::Boolean(i % 2 == 0),
                ]))
                .unwrap();
        }

        // Find active users
        let active = table.scan_where(|row| {
            matches!(row.values.get(2), Some(Value::Boolean(true)))
        });

        assert_eq!(active.len(), 2); // IDs 2 and 4
    }

    #[test]
    fn test_delete() {
        let mut table = Table::new(create_test_schema());

        let row = Row::new(vec![
            Value::Integer(1),
            Value::Text("Alice".to_string()),
            Value::Boolean(true),
        ]);

        table.insert(row).unwrap();
        assert!(table.delete_by_pk(&Value::Integer(1)).unwrap());
        assert!(!table.delete_by_pk(&Value::Integer(1)).unwrap());
        assert!(table.get_by_pk(&Value::Integer(1)).is_none());
    }
}
