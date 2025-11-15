# Lecture 04: Error Handling the Rust Way

**From Exceptions to Values: Type-Safe Error Propagation**

## Introduction

Rust doesn't have exceptions. Instead, errors are **values** returned from functions, typically as `Result<T, E>`.

This is a fundamentally different model than try/catch in TypeScript or Python. It's more explicit, more composable, and the type system enforces error handling.

Coming from exception-based languages, this will feel verbose at first. But you'll soon appreciate how it eliminates forgotten error handling and makes control flow explicit.

## The Problem with Exceptions

### TypeScript Example

```typescript
function readConfig(path: string): Config {
    const content = fs.readFileSync(path, 'utf8');  // Can throw!
    const config = JSON.parse(content);  // Can throw!
    return config;
}

// Usage
try {
    const config = readConfig('config.json');
    runWithConfig(config);
} catch (error) {
    console.error('Failed:', error);
}
```

**Problems**:
1. **Invisible control flow**: `readFileSync` and `JSON.parse` can throw, but you can't tell from the signature
2. **Easy to forget**: Nothing forces you to handle errors
3. **Type-unsafe**: `error` is `any` or `unknown`, you don't know what error types to expect
4. **Performance**: Exception unwinding has overhead

### Python Example

```python
def read_config(path: str) -> Config:
    with open(path) as f:
        content = f.read()
    config = json.loads(content)  # Can raise!
    return config

# Usage
try:
    config = read_config('config.json')
    run_with_config(config)
except Exception as e:  # Catch-all, loses type information
    print(f'Failed: {e}')
```

Same problems as TypeScript.

## Rust's Approach: Errors as Values

```rust
use std::fs;
use std::io;

fn read_config(path: &str) -> Result<Config, io::Error> {
    let content = fs::read_to_string(path)?;  // Returns error if fails
    let config: Config = serde_json::from_str(&content)?;
    Ok(config)
}

// Usage
match read_config("config.json") {
    Ok(config) => run_with_config(config),
    Err(e) => eprintln!("Failed: {}", e),
}
```

**Advantages**:
1. **Explicit in signature**: You know the function can fail
2. **Type-safe errors**: You know exactly what error type to expect
3. **Forced handling**: Compiler error if you don't handle `Result`
4. **Composable**: Easy to chain operations with `?`

## The Result Type

```rust
enum Result<T, E> {
    Ok(T),    // Success value
    Err(E),   // Error value
}
```

Functions that can fail return `Result<T, E>`:
- `T` is the success type
- `E` is the error type

### Basic Usage

```rust
use std::fs::File;

fn open_file(path: &str) -> Result<File, std::io::Error> {
    File::open(path)
}

// Handle with match
match open_file("config.toml") {
    Ok(file) => println!("Opened successfully"),
    Err(e) => eprintln!("Error: {}", e),
}

// Or with if let
if let Ok(file) = open_file("config.toml") {
    println!("Opened successfully");
}
```

### Unwrapping (Use Sparingly!)

```rust
// Panics if Err
let file = File::open("config.toml").unwrap();

// Panics with custom message
let file = File::open("config.toml").expect("Failed to open config");
```

**When to use `unwrap`**:
- Prototyping / quick scripts
- When failure is truly unrecoverable
- When you **know** the operation can't fail (use `expect` with explanation)

**Never use in production code** unless you have a good reason.

### unwrap_or and unwrap_or_else

```rust
// Provide default value
let file = File::open("config.toml").unwrap_or(default_file());

// Compute default lazily
let file = File::open("config.toml").unwrap_or_else(|_| {
    eprintln!("Using default config");
    default_file()
});
```

## The ? Operator: Error Propagation

The `?` operator is syntactic sugar for "return early if error, otherwise unwrap".

### Without ?

```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let f = File::open("username.txt");

    let mut f = match f {
        Ok(file) => file,
        Err(e) => return Err(e),  // Early return
    };

    let mut s = String::new();

    match f.read_to_string(&mut s) {
        Ok(_) => Ok(s),
        Err(e) => Err(e),  // Early return
    }
}
```

Verbose and repetitive.

### With ?

```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut f = File::open("username.txt")?;  // Propagate error
    let mut s = String::new();
    f.read_to_string(&mut s)?;  // Propagate error
    Ok(s)
}
```

Much cleaner! `?` unwraps `Ok` or returns `Err` early.

### Even More Concise

```rust
fn read_username_from_file() -> Result<String, io::Error> {
    let mut s = String::new();
    File::open("username.txt")?.read_to_string(&mut s)?;
    Ok(s)
}

// Or using fs::read_to_string
fn read_username_from_file() -> Result<String, io::Error> {
    fs::read_to_string("username.txt")
}
```

### How ? Works

```rust
let value = some_function()?;

// Expands to:
let value = match some_function() {
    Ok(v) => v,
    Err(e) => return Err(e.into()),  // Note: .into() for type conversion
};
```

The `?` operator:
1. If `Ok(v)`, unwraps to `v`
2. If `Err(e)`, returns `Err(e)` (converting via `From` if needed)

**Important**: Can only be used in functions that return `Result` (or `Option`).

```rust
fn main() {
    let f = File::open("file.txt")?;  // ❌ Error: main() doesn't return Result
}

// Fix: Make main return Result
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open("file.txt")?;  // ✅ OK
    Ok(())
}
```

## Custom Error Types

For complex applications, you'll want custom error types.

### Simple Enum Errors

```rust
#[derive(Debug)]
enum ConfigError {
    Io(io::Error),
    Parse(serde_json::Error),
    Missing(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
            ConfigError::Missing(field) => write!(f, "Missing field: {}", field),
        }
    }
}

impl std::error::Error for ConfigError {}

fn read_config(path: &str) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)
        .map_err(ConfigError::Io)?;

    let config: Config = serde_json::from_str(&content)
        .map_err(ConfigError::Parse)?;

    if config.name.is_empty() {
        return Err(ConfigError::Missing("name".to_string()));
    }

    Ok(config)
}
```

`map_err` converts one error type to another.

### Using thiserror (Recommended)

The `thiserror` crate simplifies custom errors:

```rust
use thiserror::Error;

#[derive(Error, Debug)]
enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    Missing(String),
}

fn read_config(path: &str) -> Result<Config, ConfigError> {
    let content = fs::read_to_string(path)?;  // Auto-converts io::Error
    let config: Config = serde_json::from_str(&content)?;  // Auto-converts serde_json::Error

    if config.name.is_empty() {
        return Err(ConfigError::Missing("name".to_string()));
    }

    Ok(config)
}
```

`#[from]` implements automatic conversion, so `?` works without `map_err`.

## The anyhow Crate: Application-Level Errors

For **applications** (not libraries), `anyhow` provides a catch-all error type:

```rust
use anyhow::{Result, Context};

fn read_config(path: &str) -> Result<Config> {
    let content = fs::read_to_string(path)
        .context("Failed to read config file")?;

    let config: Config = serde_json::from_str(&content)
        .context("Failed to parse config")?;

    Ok(config)
}

fn main() -> Result<()> {
    let config = read_config("config.json")?;
    run_with_config(config)?;
    Ok(())
}
```

**Result<T>** is shorthand for **Result<T, anyhow::Error>**.

**When to use**:
- **anyhow**: Applications, CLIs, quick scripts (less boilerplate)
- **thiserror**: Libraries, when you want precise error types

## Error Context

Add context to errors as they propagate:

```rust
use anyhow::{Context, Result};

fn process_user(id: u64) -> Result<()> {
    let user = fetch_user(id)
        .context(format!("Failed to fetch user {}", id))?;

    save_user(&user)
        .context("Failed to save user to database")?;

    Ok(())
}
```

If an error occurs deep in the stack, you get a nice error chain:

```
Error: Failed to fetch user 42
Caused by:
    0: Network error
    1: Connection refused
```

## Recoverable vs Unrecoverable Errors

### Recoverable: Use Result

```rust
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

match divide(10, 0) {
    Ok(result) => println!("Result: {}", result),
    Err(e) => println!("Error: {}", e),
}
```

Use when:
- Error is expected (file not found, parse error, network timeout)
- Caller should decide how to handle

### Unrecoverable: Use panic!

```rust
fn process(data: &[u8]) {
    if data.len() < 10 {
        panic!("Invalid data: too short");
    }
    // ...
}
```

Use when:
- Bug in the code (invariant violation)
- Caller can't reasonably recover
- Very rare in application code

**Examples of panic-worthy situations**:
- Array out of bounds (programmer error)
- Assertion failures in tests
- Truly impossible states (type system should prevent these)

### When in Doubt, Use Result

Panicking means the thread terminates (or the program, in a single-threaded app). Almost always prefer `Result`.

## Comparing to TypeScript/Python

### Exception Model (TypeScript/Python)

```typescript
function processData(data: string): Result {
    try {
        const parsed = JSON.parse(data);  // Can throw
        const validated = validate(parsed);  // Can throw
        return process(validated);  // Can throw
    } catch (error) {
        logger.error('Failed to process data', error);
        throw error;  // Re-throw or handle
    }
}
```

**Characteristics**:
- Invisible in type signature
- Easy to forget error handling
- Errors can bypass multiple call frames
- Performance overhead (stack unwinding)

### Result Model (Rust)

```rust
fn process_data(data: &str) -> Result<ProcessedData, ProcessError> {
    let parsed = parse_json(data)?;
    let validated = validate(parsed)?;
    let result = process(validated)?;
    Ok(result)
}
```

**Characteristics**:
- Visible in type signature
- Compiler enforces handling
- Explicit control flow
- No runtime overhead (just normal returns)

### Mental Model Shift

| Exceptions | Results |
|------------|---------|
| Control flow mechanism | Values |
| Invisible in signatures | Explicit in signatures |
| Can skip multiple frames | Must be handled at each level (or propagated with `?`) |
| `try/catch` for handling | `match` or `?` for handling |
| Runtime overhead | Zero overhead |

## Best Practices

### 1. Use ? for Propagation

```rust
// ❌ Verbose
fn foo() -> Result<(), Error> {
    match bar() {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    // ...
}

// ✅ Idiomatic
fn foo() -> Result<(), Error> {
    bar()?;
    // ...
}
```

### 2. Add Context

```rust
// ❌ Context lost
fs::read_to_string(path)?;

// ✅ Context added
fs::read_to_string(path)
    .context(format!("Failed to read config from {}", path))?;
```

### 3. Custom Errors for Libraries, anyhow for Apps

```rust
// Library code: precise error types
#[derive(Error, Debug)]
pub enum LibError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Application code: anyhow
use anyhow::Result;

fn main() -> Result<()> {
    run()?;
    Ok(())
}
```

### 4. Don't Swallow Errors

```rust
// ❌ Error information lost
let _ = dangerous_operation();

// ✅ At least log it
if let Err(e) = dangerous_operation() {
    eprintln!("Warning: operation failed: {}", e);
}
```

### 5. Return Early, Return Often

```rust
fn process(data: &Data) -> Result<Output, Error> {
    if !data.is_valid() {
        return Err(Error::InvalidData);
    }

    let intermediate = transform(data)?;

    if intermediate.is_empty() {
        return Err(Error::EmptyResult);
    }

    Ok(finalize(intermediate))
}
```

## Option and Result Together

### Option for "Not Found" vs Result for "Error"

```rust
fn find_user(id: u64) -> Option<User> {
    // None means "not found", not an error
}

fn fetch_user(id: u64) -> Result<User, DbError> {
    // Err means something went wrong (network error, db error)
    // Ok(user) or Ok might contain None-like data, depending on model
}
```

### Converting Between Option and Result

```rust
// Option -> Result
let result: Result<i32, &str> = Some(5).ok_or("Value was None");

// Result -> Option
let option: Option<i32> = Ok(5).ok();  // Discards error

// Option with ?
fn get_value() -> Option<i32> {
    let map = get_map()?;  // Returns None if get_map returns None
    map.get("key").copied()
}
```

## Error Handling Patterns

### Pattern 1: Retry Logic

```rust
fn fetch_with_retry(url: &str, retries: u32) -> Result<Response> {
    let mut last_error = None;

    for attempt in 0..=retries {
        match fetch(url) {
            Ok(response) => return Ok(response),
            Err(e) => {
                eprintln!("Attempt {} failed: {}", attempt + 1, e);
                last_error = Some(e);
                std::thread::sleep(Duration::from_secs(1 << attempt));
            }
        }
    }

    Err(last_error.unwrap())
}
```

### Pattern 2: Fallback Chain

```rust
fn load_config() -> Result<Config> {
    load_from_file("config.local.toml")
        .or_else(|_| load_from_file("config.toml"))
        .or_else(|_| Ok(Config::default()))
}
```

### Pattern 3: Collecting Results

```rust
// Process multiple items, stop at first error
fn process_all(items: &[Item]) -> Result<Vec<Output>> {
    items.iter().map(|item| process(item)).collect()
}

// Process all, collect errors
let results: Vec<Result<Output, Error>> = items.iter()
    .map(|item| process(item))
    .collect();

let (successes, failures): (Vec<_>, Vec<_>) = results.into_iter()
    .partition(Result::is_ok);
```

### Pattern 4: Validation

```rust
fn validate_config(config: &Config) -> Result<(), ValidationError> {
    if config.port == 0 {
        return Err(ValidationError::InvalidPort);
    }

    if config.timeout > 300 {
        return Err(ValidationError::TimeoutTooLarge);
    }

    Ok(())
}
```

## Key Takeaways

1. **Errors are values**, not exceptions
2. **Result<T, E>** is the primary error handling type
3. **? operator** makes error propagation concise
4. **Type signatures** explicitly show functions that can fail
5. **Compiler enforces** error handling (can't ignore `Result`)
6. **thiserror** for libraries (precise errors), **anyhow** for applications (convenience)
7. **panic!** for unrecoverable errors only (rare)
8. **Add context** as errors propagate up the stack
9. **Option** for "not found", **Result** for "error"

## Practice

Before moving on, ensure you can:

- [ ] Explain the difference between `Result` and exceptions
- [ ] Use the `?` operator to propagate errors
- [ ] Create custom error types with `thiserror`
- [ ] Use `anyhow` for application-level error handling
- [ ] Understand when to use `panic!` vs `Result`
- [ ] Convert between `Option` and `Result`
- [ ] Add context to errors with `.context()`

## Next Steps

Read [Lecture 05: Traits and Generics →](./05-traits-and-generics.md) to learn Rust's approach to polymorphism and abstraction.

Or practice error handling in [Exercise 02: Config-Driven CLI →](../exercises/ex02-config-cli/), which heavily uses `Result` and `anyhow`.
