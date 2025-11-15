mod database;
mod error;
mod parser;
mod table;
mod types;

use database::Database;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn print_results(result: &database::QueryResult) {
    if result.rows.is_empty() {
        println!("Empty result set");
        return;
    }

    // Print header
    println!("{}", result.columns.join(" | "));
    println!("{}", "-".repeat(result.columns.len() * 15));

    // Print rows
    for row in &result.rows {
        let values: Vec<String> = row.iter().map(|v| format!("{:?}", v)).collect();
        println!("{}", values.join(" | "));
    }
    println!();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "simple_db=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("=== Simple Database Demo ===\n");

    let mut db = Database::new();

    // Create a table
    println!("Creating table 'employees'...");
    db.execute("CREATE TABLE employees (id INTEGER PRIMARY KEY, name TEXT NOT NULL, salary INTEGER, active BOOLEAN)")?;

    // Insert some data
    println!("Inserting employees...");
    db.execute("INSERT INTO employees VALUES (1, 'Alice', 75000, TRUE)")?;
    db.execute("INSERT INTO employees VALUES (2, 'Bob', 65000, TRUE)")?;
    db.execute("INSERT INTO employees VALUES (3, 'Charlie', 80000, FALSE)")?;
    db.execute("INSERT INTO employees VALUES (4, 'Diana', 90000, TRUE)")?;

    // Query all employees
    println!("\n--- All Employees ---");
    let result = db.execute("SELECT * FROM employees")?;
    print_results(&result);

    // Query active employees
    println!("--- Active Employees ---");
    let result = db.execute("SELECT name, salary FROM employees WHERE active = TRUE")?;
    print_results(&result);

    // Query high earners
    println!("--- Employees earning > 70000 ---");
    let result = db.execute("SELECT name, salary FROM employees WHERE salary > 70000")?;
    print_results(&result);

    // Create another table
    println!("Creating table 'departments'...");
    db.execute("CREATE TABLE departments (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")?;

    db.execute("INSERT INTO departments VALUES (1, 'Engineering')")?;
    db.execute("INSERT INTO departments VALUES (2, 'Sales')")?;

    println!("--- All Departments ---");
    let result = db.execute("SELECT * FROM departments")?;
    print_results(&result);

    // List all tables
    println!("Tables in database: {:?}", db.list_tables());

    println!("\n=== Demo Complete ===");

    Ok(())
}
