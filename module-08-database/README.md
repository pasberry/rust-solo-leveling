# Module 08: SQLite-like Database

**Build a Relational Database with SQL Support**

## Overview

Build a simple relational database with:
- B+ tree storage engine
- Page-based file format
- SQL parser and query planner
- Transaction support (ACID)
- Index management
- Basic query optimization

**Duration**: 4-5 weeks (40-50 hours)

**Difficulty**: ⭐⭐⭐⭐⭐ (Most Complex Module)

## What You'll Build

```rust
// Usage
let db = Database::open("mydb.db")?;

// Create table
db.execute("
    CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        email TEXT UNIQUE,
        age INTEGER
    )
")?;

// Insert data
db.execute("INSERT INTO users (id, name, email, age) VALUES (1, 'Alice', 'alice@example.com', 30)")?;
db.execute("INSERT INTO users (id, name, email, age) VALUES (2, 'Bob', 'bob@example.com', 25)")?;

// Query data
let rows = db.query("SELECT name, age FROM users WHERE age > 25")?;
for row in rows {
    println!("{}: {}", row.get::<String>("name")?, row.get::<i64>("age")?);
}

// Transactions
let tx = db.begin_transaction()?;
tx.execute("UPDATE users SET age = age + 1 WHERE name = 'Alice'")?;
tx.commit()?;
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│              SQL Layer                          │
│  Parser → Analyzer → Planner → Executor        │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│           Storage Engine                        │
│  - B+ Tree Index                                │
│  - Page Manager                                 │
│  - Buffer Pool                                  │
│  - WAL (Write-Ahead Log)                        │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│           File System                           │
│  - Database file (pages)                        │
│  - WAL file (transactions)                      │
└─────────────────────────────────────────────────┘
```

## File Format

```
Database File Layout:
┌──────────────────┐
│  Header Page     │  Page 0: Metadata, schema version, page size
├──────────────────┤
│  Schema Page     │  Page 1: Table definitions, indexes
├──────────────────┤
│  B+ Tree Root    │  Page 2+: Table data (B+ tree nodes)
├──────────────────┤
│  ...             │
├──────────────────┤
│  Free Pages      │  Reclaimed pages for reuse
└──────────────────┘

Page Structure (4KB default):
┌─────────────────────────────────┐
│  Page Header (24 bytes)         │
│  - page_type: u8                │
│  - free_offset: u16             │
│  - cell_count: u16              │
│  - cell_offset: u16             │
│  - parent_page: u32             │
│  - right_sibling: u32           │
├─────────────────────────────────┤
│  Cell Pointer Array             │  Offsets to cells
├─────────────────────────────────┤
│  Free Space                     │
├─────────────────────────────────┤
│  Cells (variable size)          │  Actual row data
└─────────────────────────────────┘
```

## Key Components

### 1. Page Manager

```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};

const PAGE_SIZE: usize = 4096;

#[derive(Debug)]
pub struct PageManager {
    file: File,
    num_pages: u32,
}

#[derive(Debug, Clone)]
pub struct Page {
    pub id: u32,
    pub data: [u8; PAGE_SIZE],
}

impl PageManager {
    pub fn open(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let metadata = file.metadata()?;
        let num_pages = (metadata.len() / PAGE_SIZE as u64) as u32;

        Ok(PageManager { file, num_pages })
    }

    pub fn read_page(&mut self, page_id: u32) -> Result<Page> {
        if page_id >= self.num_pages {
            return Err(Error::InvalidPageId(page_id));
        }

        let offset = page_id as u64 * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        let mut data = [0u8; PAGE_SIZE];
        self.file.read_exact(&mut data)?;

        Ok(Page { id: page_id, data })
    }

    pub fn write_page(&mut self, page: &Page) -> Result<()> {
        let offset = page.id as u64 * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.data)?;
        self.file.sync_data()?;
        Ok(())
    }

    pub fn allocate_page(&mut self) -> Result<Page> {
        let page_id = self.num_pages;
        self.num_pages += 1;

        let page = Page {
            id: page_id,
            data: [0u8; PAGE_SIZE],
        };

        self.write_page(&page)?;
        Ok(page)
    }

    pub fn num_pages(&self) -> u32 {
        self.num_pages
    }
}
```

### 2. B+ Tree Implementation

```rust
use std::cmp::Ordering;

const MAX_KEYS_PER_NODE: usize = 127;  // Fits in 4KB page

#[derive(Debug, Clone)]
pub enum BTreeNode {
    Internal(InternalNode),
    Leaf(LeafNode),
}

#[derive(Debug, Clone)]
pub struct InternalNode {
    pub keys: Vec<Value>,
    pub children: Vec<u32>,  // Page IDs
}

#[derive(Debug, Clone)]
pub struct LeafNode {
    pub keys: Vec<Value>,
    pub values: Vec<Vec<u8>>,  // Serialized rows
    pub next_leaf: Option<u32>,  // For range scans
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    Null,
    Integer(i64),
    Real(OrderedFloat<f64>),
    Text(String),
    Blob(Vec<u8>),
}

pub struct BTree {
    page_manager: Arc<Mutex<PageManager>>,
    root_page_id: u32,
}

impl BTree {
    pub fn new(page_manager: Arc<Mutex<PageManager>>) -> Result<Self> {
        let mut pm = page_manager.lock().unwrap();
        let root_page = pm.allocate_page()?;
        let root_page_id = root_page.id;

        // Initialize as empty leaf
        let leaf = LeafNode {
            keys: Vec::new(),
            values: Vec::new(),
            next_leaf: None,
        };

        Self::write_node(&mut pm, root_page_id, &BTreeNode::Leaf(leaf))?;

        Ok(BTree {
            page_manager,
            root_page_id,
        })
    }

    pub fn insert(&mut self, key: Value, value: Vec<u8>) -> Result<()> {
        let mut pm = self.page_manager.lock().unwrap();

        // Find leaf node for insertion
        let (leaf_page_id, mut leaf) = self.find_leaf_for_insert(&mut pm, &key)?;

        // Insert into leaf
        let insert_pos = leaf.keys.binary_search(&key).unwrap_or_else(|e| e);
        leaf.keys.insert(insert_pos, key.clone());
        leaf.values.insert(insert_pos, value);

        // Check if split needed
        if leaf.keys.len() > MAX_KEYS_PER_NODE {
            self.split_leaf(&mut pm, leaf_page_id, leaf)?;
        } else {
            Self::write_node(&mut pm, leaf_page_id, &BTreeNode::Leaf(leaf))?;
        }

        Ok(())
    }

    pub fn search(&self, key: &Value) -> Result<Option<Vec<u8>>> {
        let mut pm = self.page_manager.lock().unwrap();
        let mut current_page_id = self.root_page_id;

        loop {
            let node = Self::read_node(&mut pm, current_page_id)?;

            match node {
                BTreeNode::Internal(internal) => {
                    // Binary search for child
                    let child_idx = match internal.keys.binary_search(key) {
                        Ok(idx) => idx + 1,
                        Err(idx) => idx,
                    };
                    current_page_id = internal.children[child_idx];
                }
                BTreeNode::Leaf(leaf) => {
                    // Binary search in leaf
                    return Ok(leaf.keys.binary_search(key)
                        .ok()
                        .map(|idx| leaf.values[idx].clone()));
                }
            }
        }
    }

    pub fn range_scan(&self, start: &Value, end: &Value) -> Result<Vec<(Value, Vec<u8>)>> {
        let mut pm = self.page_manager.lock().unwrap();
        let mut results = Vec::new();

        // Find starting leaf
        let (mut current_page_id, leaf) = self.find_leaf_for_insert(&mut pm, start)?;

        // Scan leaves
        loop {
            let node = Self::read_node(&mut pm, current_page_id)?;

            match node {
                BTreeNode::Leaf(leaf) => {
                    for (i, key) in leaf.keys.iter().enumerate() {
                        if key >= start && key <= end {
                            results.push((key.clone(), leaf.values[i].clone()));
                        } else if key > end {
                            return Ok(results);
                        }
                    }

                    // Move to next leaf
                    match leaf.next_leaf {
                        Some(next_page_id) => current_page_id = next_page_id,
                        None => break,
                    }
                }
                _ => return Err(Error::CorruptedIndex),
            }
        }

        Ok(results)
    }

    fn split_leaf(&mut self, pm: &mut PageManager, page_id: u32, mut leaf: LeafNode) -> Result<()> {
        let mid = leaf.keys.len() / 2;

        // Create new leaf with right half
        let new_page = pm.allocate_page()?;
        let new_leaf = LeafNode {
            keys: leaf.keys.split_off(mid),
            values: leaf.values.split_off(mid),
            next_leaf: leaf.next_leaf,
        };

        leaf.next_leaf = Some(new_page.id);

        // Write both leaves
        Self::write_node(pm, page_id, &BTreeNode::Leaf(leaf.clone()))?;
        Self::write_node(pm, new_page.id, &BTreeNode::Leaf(new_leaf.clone()))?;

        // Promote middle key to parent
        let promoted_key = new_leaf.keys[0].clone();
        self.insert_into_parent(pm, page_id, promoted_key, new_page.id)?;

        Ok(())
    }

    fn insert_into_parent(
        &mut self,
        pm: &mut PageManager,
        left_page_id: u32,
        key: Value,
        right_page_id: u32,
    ) -> Result<()> {
        // If splitting root, create new root
        if left_page_id == self.root_page_id {
            let new_root = InternalNode {
                keys: vec![key],
                children: vec![left_page_id, right_page_id],
            };

            let new_root_page = pm.allocate_page()?;
            Self::write_node(pm, new_root_page.id, &BTreeNode::Internal(new_root))?;
            self.root_page_id = new_root_page.id;
            return Ok(());
        }

        // Otherwise, insert into existing parent
        // (Implementation omitted for brevity - would recursively split if needed)
        todo!("Insert into existing parent node")
    }

    fn read_node(pm: &mut PageManager, page_id: u32) -> Result<BTreeNode> {
        let page = pm.read_page(page_id)?;
        // Deserialize from page.data
        bincode::deserialize(&page.data).map_err(|e| Error::Deserialization(e))
    }

    fn write_node(pm: &mut PageManager, page_id: u32, node: &BTreeNode) -> Result<()> {
        let mut page = pm.read_page(page_id)?;
        let serialized = bincode::serialize(node)?;
        page.data[..serialized.len()].copy_from_slice(&serialized);
        pm.write_page(&page)?;
        Ok(())
    }

    fn find_leaf_for_insert(&self, pm: &mut PageManager, key: &Value) -> Result<(u32, LeafNode)> {
        let mut current_page_id = self.root_page_id;

        loop {
            let node = Self::read_node(pm, current_page_id)?;

            match node {
                BTreeNode::Internal(internal) => {
                    let child_idx = match internal.keys.binary_search(key) {
                        Ok(idx) => idx + 1,
                        Err(idx) => idx,
                    };
                    current_page_id = internal.children[child_idx];
                }
                BTreeNode::Leaf(leaf) => {
                    return Ok((current_page_id, leaf));
                }
            }
        }
    }
}
```

### 3. SQL Parser

```rust
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::complete::{alpha1, alphanumeric1, multispace0, multispace1},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    CreateTable {
        name: String,
        columns: Vec<ColumnDef>,
    },
    Insert {
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Value>,
    },
    Select {
        columns: Vec<String>,
        from: String,
        where_clause: Option<Expr>,
        order_by: Option<Vec<String>>,
        limit: Option<usize>,
    },
    Update {
        table: String,
        assignments: Vec<(String, Value)>,
        where_clause: Option<Expr>,
    },
    Delete {
        from: String,
        where_clause: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Integer,
    Real,
    Text,
    Blob,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    PrimaryKey,
    NotNull,
    Unique,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Column(String),
    Value(Value),
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

pub fn parse_sql(input: &str) -> Result<Statement, ParseError> {
    let (_, stmt) = statement(input)
        .map_err(|e| ParseError::Syntax(format!("{:?}", e)))?;
    Ok(stmt)
}

fn statement(input: &str) -> IResult<&str, Statement> {
    alt((
        create_table,
        insert,
        select,
        update,
        delete,
    ))(input)
}

fn create_table(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag_no_case("CREATE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag_no_case("TABLE")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, table_name) = identifier(input)?;
    let (input, _) = multispace0(input)?;
    let (input, columns) = delimited(
        tag("("),
        column_def_list,
        tag(")"),
    )(input)?;

    Ok((input, Statement::CreateTable {
        name: table_name.to_string(),
        columns,
    }))
}

fn select(input: &str) -> IResult<&str, Statement> {
    let (input, _) = tag_no_case("SELECT")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, columns) = column_list(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag_no_case("FROM")(input)?;
    let (input, _) = multispace1(input)?;
    let (input, table) = identifier(input)?;
    let (input, where_clause) = opt(preceded(
        tuple((multispace1, tag_no_case("WHERE"), multispace1)),
        expr,
    ))(input)?;

    Ok((input, Statement::Select {
        columns: columns.iter().map(|s| s.to_string()).collect(),
        from: table.to_string(),
        where_clause,
        order_by: None,
        limit: None,
    }))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn column_list(input: &str) -> IResult<&str, Vec<&str>> {
    separated_list1(
        delimited(multispace0, tag(","), multispace0),
        identifier,
    )(input)
}

fn column_def_list(input: &str) -> IResult<&str, Vec<ColumnDef>> {
    separated_list1(
        delimited(multispace0, tag(","), multispace0),
        column_def,
    )(input)
}

fn column_def(input: &str) -> IResult<&str, ColumnDef> {
    let (input, _) = multispace0(input)?;
    let (input, name) = identifier(input)?;
    let (input, _) = multispace1(input)?;
    let (input, data_type) = data_type(input)?;
    let (input, _) = multispace0(input)?;
    let (input, constraints) = opt(constraint_list)(input)?;

    Ok((input, ColumnDef {
        name: name.to_string(),
        data_type,
        constraints: constraints.unwrap_or_default(),
    }))
}

fn data_type(input: &str) -> IResult<&str, DataType> {
    alt((
        map(tag_no_case("INTEGER"), |_| DataType::Integer),
        map(tag_no_case("REAL"), |_| DataType::Real),
        map(tag_no_case("TEXT"), |_| DataType::Text),
        map(tag_no_case("BLOB"), |_| DataType::Blob),
    ))(input)
}

fn constraint_list(input: &str) -> IResult<&str, Vec<Constraint>> {
    // Parse constraints like PRIMARY KEY, NOT NULL, UNIQUE
    todo!()
}

fn expr(input: &str) -> IResult<&str, Expr> {
    // Parse WHERE expressions with binary operators
    todo!()
}
```

### 4. Query Executor

```rust
pub struct QueryExecutor {
    storage: Arc<Mutex<StorageEngine>>,
    catalog: Arc<RwLock<Catalog>>,
}

pub struct Catalog {
    tables: HashMap<String, TableSchema>,
}

pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    pub root_page_id: u32,
}

impl QueryExecutor {
    pub fn execute(&self, stmt: Statement) -> Result<QueryResult> {
        match stmt {
            Statement::CreateTable { name, columns } => {
                self.execute_create_table(name, columns)
            }
            Statement::Insert { table, columns, values } => {
                self.execute_insert(table, columns, values)
            }
            Statement::Select { columns, from, where_clause, .. } => {
                self.execute_select(columns, from, where_clause)
            }
            Statement::Update { table, assignments, where_clause } => {
                self.execute_update(table, assignments, where_clause)
            }
            Statement::Delete { from, where_clause } => {
                self.execute_delete(from, where_clause)
            }
        }
    }

    fn execute_create_table(&self, name: String, columns: Vec<ColumnDef>) -> Result<QueryResult> {
        let mut catalog = self.catalog.write().unwrap();

        if catalog.tables.contains_key(&name) {
            return Err(Error::TableExists(name));
        }

        let mut storage = self.storage.lock().unwrap();
        let root_page_id = storage.create_table(&name)?;

        let schema = TableSchema {
            name: name.clone(),
            columns,
            root_page_id,
        };

        catalog.tables.insert(name, schema);

        Ok(QueryResult::Empty)
    }

    fn execute_insert(
        &self,
        table: String,
        columns: Option<Vec<String>>,
        values: Vec<Value>,
    ) -> Result<QueryResult> {
        let catalog = self.catalog.read().unwrap();
        let schema = catalog.tables.get(&table)
            .ok_or_else(|| Error::TableNotFound(table.clone()))?;

        // Validate columns and values match
        let col_names: Vec<String> = if let Some(cols) = columns {
            cols
        } else {
            schema.columns.iter().map(|c| c.name.clone()).collect()
        };

        if col_names.len() != values.len() {
            return Err(Error::ColumnCountMismatch);
        }

        // Serialize row
        let row = Row {
            values: values.clone(),
        };
        let row_data = bincode::serialize(&row)?;

        // Generate primary key
        let pk = match &values[0] {
            Value::Integer(i) => Value::Integer(*i),
            _ => return Err(Error::InvalidPrimaryKey),
        };

        // Insert into B+ tree
        let mut storage = self.storage.lock().unwrap();
        storage.insert(schema.root_page_id, pk, row_data)?;

        Ok(QueryResult::RowsAffected(1))
    }

    fn execute_select(
        &self,
        columns: Vec<String>,
        from: String,
        where_clause: Option<Expr>,
    ) -> Result<QueryResult> {
        let catalog = self.catalog.read().unwrap();
        let schema = catalog.tables.get(&from)
            .ok_or_else(|| Error::TableNotFound(from.clone()))?;

        // Full table scan (no indexes for simplicity)
        let storage = self.storage.lock().unwrap();
        let all_rows = storage.scan_table(schema.root_page_id)?;

        // Filter rows
        let mut result_rows = Vec::new();
        for (key, row_data) in all_rows {
            let row: Row = bincode::deserialize(&row_data)?;

            // Evaluate WHERE clause
            if let Some(ref expr) = where_clause {
                if !self.evaluate_expr(expr, &row, schema)? {
                    continue;
                }
            }

            // Project columns
            let projected = self.project_columns(&columns, &row, schema)?;
            result_rows.push(projected);
        }

        Ok(QueryResult::Rows(result_rows))
    }

    fn evaluate_expr(&self, expr: &Expr, row: &Row, schema: &TableSchema) -> Result<bool> {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expr_value(left, row, schema)?;
                let right_val = self.evaluate_expr_value(right, row, schema)?;

                Ok(match op {
                    BinaryOp::Eq => left_val == right_val,
                    BinaryOp::Ne => left_val != right_val,
                    BinaryOp::Lt => left_val < right_val,
                    BinaryOp::Le => left_val <= right_val,
                    BinaryOp::Gt => left_val > right_val,
                    BinaryOp::Ge => left_val >= right_val,
                    BinaryOp::And | BinaryOp::Or => {
                        return Err(Error::UnsupportedOperation("logical AND/OR"))
                    }
                })
            }
            _ => Ok(true),
        }
    }

    fn evaluate_expr_value(&self, expr: &Expr, row: &Row, schema: &TableSchema) -> Result<Value> {
        match expr {
            Expr::Value(v) => Ok(v.clone()),
            Expr::Column(col_name) => {
                let col_idx = schema.columns.iter()
                    .position(|c| &c.name == col_name)
                    .ok_or_else(|| Error::ColumnNotFound(col_name.clone()))?;
                Ok(row.values[col_idx].clone())
            }
            _ => Err(Error::InvalidExpression),
        }
    }

    fn project_columns(&self, columns: &[String], row: &Row, schema: &TableSchema) -> Result<Row> {
        let mut projected_values = Vec::new();

        for col_name in columns {
            let col_idx = schema.columns.iter()
                .position(|c| &c.name == col_name)
                .ok_or_else(|| Error::ColumnNotFound(col_name.clone()))?;
            projected_values.push(row.values[col_idx].clone());
        }

        Ok(Row { values: projected_values })
    }
}

#[derive(Debug)]
pub enum QueryResult {
    Empty,
    RowsAffected(usize),
    Rows(Vec<Row>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    pub values: Vec<Value>,
}
```

## Implementation Roadmap

### Phase 1: Page Manager & File I/O (Days 1-4)
- Implement page-based file format
- Page allocation and deallocation
- Read/write operations
- Test with large files (10k+ pages)

**Success criteria:**
- Can read/write pages reliably
- Handle file growth correctly
- No data corruption on crashes

### Phase 2: B+ Tree Storage (Days 5-10)
- Build B+ tree with insert/search/delete
- Implement node splitting
- Add range scans
- Test with 1M+ entries

**Success criteria:**
- Balanced tree maintains O(log n) operations
- Range scans work efficiently
- Handles duplicates correctly

### Phase 3: SQL Parser (Days 11-15)
- Write parser for CREATE, INSERT, SELECT
- Handle WHERE clauses
- Parse data types and constraints
- Comprehensive parser tests

**Success criteria:**
- Parses all basic SQL statements
- Good error messages
- Handles edge cases (whitespace, case-insensitivity)

### Phase 4: Query Executor (Days 16-22)
- Implement CREATE TABLE
- Implement INSERT/SELECT
- Add WHERE clause evaluation
- Column projection

**Success criteria:**
- Can create tables and insert data
- SELECT with filters works
- Multi-table database support

### Phase 5: Transactions (Days 23-28)
- Write-ahead logging (WAL)
- BEGIN/COMMIT/ROLLBACK
- Crash recovery
- Isolation (basic locking)

**Success criteria:**
- ACID guarantees
- Survives crashes without data loss
- Concurrent reads work

### Phase 6: Indexes & Optimization (Days 29-35)
- CREATE INDEX support
- Index usage in queries
- Basic query optimization
- EXPLAIN command

**Success criteria:**
- Indexes speed up queries measurably
- Optimizer chooses indexes when beneficial

### Phase 7: Polish & Advanced Features (Days 36-40)
- JOIN support (nested loop join)
- Aggregate functions (COUNT, SUM, AVG)
- GROUP BY and HAVING
- ORDER BY and LIMIT

## Performance Targets

- **Insert**: >10k rows/sec
- **Point query**: <1ms
- **Range scan**: >1M rows/sec
- **Table scan**: >500k rows/sec
- **Database size**: Handle 10GB+ files

## Success Criteria

- ✅ Persistent storage with page-based format
- ✅ B+ tree index with O(log n) operations
- ✅ SQL parser for basic statements
- ✅ Query executor with WHERE clauses
- ✅ Transactions with ACID guarantees
- ✅ Crash recovery via WAL
- ✅ Index support for performance
- ✅ Basic JOIN and aggregates

## Comparison with Real SQLite

**What you'll implement:**
- Page-based storage
- B+ tree indexes
- Basic SQL (CREATE, INSERT, SELECT, UPDATE, DELETE)
- Transactions with WAL
- Simple query optimization

**What SQLite has that you won't:**
- Virtual machine bytecode execution
- Advanced query optimizer (statistics, cost-based)
- Full SQL-92 support
- Views, triggers, CTEs
- Full-text search
- JSON support
- Encryption

**Learning focus:**
- Storage engine design
- B+ tree implementation
- SQL parsing and execution
- Transaction management
- Crash recovery

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_query_table() {
        let db = Database::open(":memory:").unwrap();

        db.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)").unwrap();
        db.execute("INSERT INTO users VALUES (1, 'Alice')").unwrap();
        db.execute("INSERT INTO users VALUES (2, 'Bob')").unwrap();

        let rows = db.query("SELECT * FROM users WHERE id = 1").unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get::<i64>("id").unwrap(), 1);
        assert_eq!(rows[0].get::<String>("name").unwrap(), "Alice");
    }

    #[test]
    fn test_transaction_rollback() {
        let db = Database::open(":memory:").unwrap();
        db.execute("CREATE TABLE counter (value INTEGER)").unwrap();
        db.execute("INSERT INTO counter VALUES (0)").unwrap();

        let tx = db.begin_transaction().unwrap();
        tx.execute("UPDATE counter SET value = 100").unwrap();
        tx.rollback().unwrap();

        let rows = db.query("SELECT value FROM counter").unwrap();
        assert_eq!(rows[0].get::<i64>("value").unwrap(), 0);
    }

    #[test]
    fn test_crash_recovery() {
        // Write to DB, simulate crash before commit
        // Reopen DB, verify data integrity
    }

    #[test]
    fn test_large_dataset() {
        // Insert 1M rows
        // Verify query performance
    }
}
```

## Extensions & Variations

**After completing the core project, try:**

1. **Add query optimization**:
   - Statistics collection
   - Cost-based optimizer
   - Index selection

2. **Implement JOINs**:
   - Hash join
   - Merge join
   - Query planning for joins

3. **Add concurrency**:
   - MVCC for readers
   - Row-level locking
   - Deadlock detection

4. **Build REPL**:
   - Interactive SQL shell
   - EXPLAIN QUERY PLAN
   - Performance metrics

5. **Network protocol**:
   - Client-server mode
   - Wire protocol like PostgreSQL

## Resources

**Database Internals:**
- "Database Internals" by Alex Petrov
- "Architecture of a Database System" (Berkeley paper)
- SQLite source code and documentation
- CMU Database Systems course (Andy Pavlo)

**Rust Crates:**
- `nom` - Parser combinators
- `bincode` - Serialization
- `serde` - Data structures
- `rusqlite` - Study reference implementation

**B+ Tree Resources:**
- "Modern B-Tree Techniques" paper
- Visualgo B+ tree visualization
- CMU lecture on B+ trees

**Similar Projects:**
- SQLite (reference implementation)
- ToyDB (Rust educational database)
- SimpleDB (Java educational database)

## Next Module

[Module 09: Compiler/Interpreter →](../module-09-compiler/)
