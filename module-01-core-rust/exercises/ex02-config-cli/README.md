# Exercise 02: Config-Driven CLI Tool

**Difficulty**: Easy-Medium
**Time**: 2-3 hours
**Focus**: Error handling, file I/O, serialization, CLI argument parsing

## Overview

Build a command-line tool that reads configuration from TOML or JSON files and processes data accordingly. This exercise emphasizes **idiomatic error handling** with `Result`, file I/O, and working with the `serde` ecosystem.

## What You'll Build

A CLI tool called `dataproc` that:
1. Reads a config file (TOML or JSON format)
2. Processes input data based on config settings
3. Outputs results with proper error handling
4. Supports multiple operations (filter, transform, aggregate)

## Requirements

### Configuration Format

Support TOML configs like this:

```toml
[input]
file = "data.txt"
format = "lines"  # or "csv", "json"

[processing]
operation = "filter"  # or "transform", "count"
pattern = "error"     # for filter operation
case_sensitive = false

[output]
file = "output.txt"
format = "text"
```

### Supported Operations

1. **Filter**: Keep only lines matching a pattern
2. **Transform**: Apply transformations (uppercase, lowercase, reverse)
3. **Count**: Count occurrences of patterns or words

### API Design

```rust
use std::path::Path;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    pub input: InputConfig,
    pub processing: ProcessingConfig,
    pub output: OutputConfig,
}

#[derive(Debug, serde::Deserialize)]
pub struct InputConfig {
    pub file: String,
    pub format: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ProcessingConfig {
    pub operation: String,
    #[serde(default)]
    pub pattern: Option<String>,
    #[serde(default)]
    pub case_sensitive: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct OutputConfig {
    pub file: String,
    pub format: String,
}

impl Config {
    /// Load config from a file (TOML or JSON based on extension)
    pub fn load(path: &Path) -> Result<Self, ConfigError>;

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError>;
}

/// Process data according to configuration
pub fn process(config: &Config) -> Result<ProcessingResult, ProcessingError>;
```

### Error Types

Create custom error types using `thiserror`:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),
}

#[derive(Error, Debug)]
pub enum ProcessingError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Pattern required for {0} operation")]
    MissingPattern(String),
}
```

## Acceptance Criteria

Your implementation must:

1. **Load TOML and JSON configs** based on file extension
2. **Validate configuration** before processing
3. **Handle all errors gracefully** with informative messages
4. **Support all three operations** (filter, transform, count)
5. **Write output to file** in specified format
6. **Provide helpful CLI** with `--help` and `--config` flags

### Example Usage

```bash
# Using default config.toml
cargo run

# Specify config file
cargo run -- --config my-config.toml

# Show help
cargo run -- --help
```

### Test Cases

```rust
#[test]
fn test_load_toml_config() {
    let config = Config::load(Path::new("test-config.toml")).unwrap();
    assert_eq!(config.input.file, "data.txt");
}

#[test]
fn test_filter_operation() {
    let data = vec!["ERROR: failed", "INFO: success", "ERROR: timeout"];
    let filtered = filter_lines(&data, "ERROR", false);
    assert_eq!(filtered.len(), 2);
}

#[test]
fn test_transform_uppercase() {
    let data = vec!["hello", "world"];
    let transformed = transform_lines(&data, Transform::Uppercase);
    assert_eq!(transformed, vec!["HELLO", "WORLD"]);
}

#[test]
fn test_count_occurrences() {
    let data = vec!["hello world", "hello rust", "goodbye world"];
    let counts = count_words(&data);
    assert_eq!(counts.get("hello"), Some(&2));
    assert_eq!(counts.get("world"), Some(&2));
}
```

## Getting Started

### Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
serde_json = "1.0"
thiserror = "1.0"
clap = { version = "4.0", features = ["derive"] }
anyhow = "1.0"
```

### Project Structure

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library code
├── config.rs        # Config types and loading
├── processing.rs    # Data processing logic
└── error.rs         # Error types
```

### Implementation Steps

1. **Define config types** with serde
2. **Implement config loading** (TOML/JSON)
3. **Add validation logic**
4. **Implement each operation** (filter, transform, count)
5. **Create CLI with clap**
6. **Write comprehensive tests**
7. **Add error context** with helpful messages

## Tips

### File I/O

```rust
use std::fs;
use std::path::Path;

// Reading
let content = fs::read_to_string(path)
    .context(format!("Failed to read file: {}", path.display()))?;

// Writing
fs::write(output_path, content)
    .context(format!("Failed to write to: {}", output_path.display()))?;
```

### Parsing Config Based on Extension

```rust
impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;

        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => toml::from_str(&content).map_err(Into::into),
            Some("json") => serde_json::from_str(&content).map_err(Into::into),
            _ => Err(ConfigError::Invalid("Unsupported file extension".into())),
        }
    }
}
```

### Using Clap for CLI

```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "dataproc")]
#[command(about = "Process data files based on configuration")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let config = Config::load(Path::new(&args.config))?;
    config.validate()?;
    let result = process(&config)?;
    println!("Processed {} items", result.count);
    Ok(())
}
```

## Stretch Goals

1. **Add more operations**: deduplicate, sort, sample
2. **Support stdin/stdout**: Read from stdin if no input file
3. **Add progress bars**: Using `indicatif` crate
4. **Parallel processing**: Use `rayon` for large files
5. **Add logging**: Use `env_logger` or `tracing`

## Learning Objectives

By completing this exercise, you'll master:
- ✅ File I/O with proper error handling
- ✅ Serde for serialization/deserialization
- ✅ Custom error types with `thiserror`
- ✅ CLI argument parsing with `clap`
- ✅ Error context with `anyhow`
- ✅ Validation patterns
- ✅ Testing file-based operations

## Resources

- [Serde documentation](https://serde.rs/)
- [Thiserror crate](https://docs.rs/thiserror/)
- [Clap documentation](https://docs.rs/clap/)
- Review Lecture 04 (Error Handling)

## Next Steps

After completing this exercise, move to [Exercise 03: Text Search Tool →](../ex03-text-search/) to practice collections and iterators in a more complex scenario.

Then review the [solutions](../../solutions/ex02-config-cli/) and compare your approach!
