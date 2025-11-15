# Lecture 07: Testing and Development Workflow

**Built-in Testing, Cargo, and Development Best Practices**

## Introduction

Rust has first-class testing support built into the language and toolchain. Coming from TypeScript (Jest, Mocha) or Python (pytest, unittest), you'll find Rust's testing story refreshingly simple and integrated.

This lecture covers:
- Unit tests, integration tests, and doc tests
- Cargo workflow and commands
- Property-based testing
- Benchmarking basics

## Cargo: Rust's Build Tool

Cargo is Rust's package manager and build tool—like `npm` for Node or `pip` + `setuptools` for Python, but integrated from the start.

### Common Commands

```bash
# Create new project
cargo new my_project
cargo new my_lib --lib

# Build (debug mode)
cargo build

# Build (release mode, optimized)
cargo build --release

# Run (builds if needed)
cargo run
cargo run --release

# Run tests
cargo test

# Check code without building (faster)
cargo check

# Generate documentation
cargo doc --open

# Format code
cargo fmt

# Lint code
cargo clippy

# Clean build artifacts
cargo clean
```

### Project Structure

```
my_project/
├── Cargo.toml          # Package metadata and dependencies
├── Cargo.lock          # Locked dependency versions
├── src/
│   ├── main.rs         # Binary entry point (or lib.rs for library)
│   ├── lib.rs          # Library code (if both bin and lib)
│   └── bin/            # Additional binaries
├── tests/              # Integration tests
├── benches/            # Benchmarks
└── examples/           # Example programs
```

### Cargo.toml

```toml
[package]
name = "my_project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
proptest = "1.0"
criterion = "0.5"

[[bin]]
name = "my_binary"
path = "src/bin/my_binary.rs"
```

## Unit Tests

Unit tests live in the same file as the code they test, in a `tests` module.

### Basic Test

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;  // Import from parent module

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }
}
```

Run with:
```bash
cargo test
```

### Assertions

```rust
#[test]
fn test_assertions() {
    // Equality
    assert_eq!(2 + 2, 4);
    assert_ne!(2 + 2, 5);

    // Boolean
    assert!(true);
    assert!(!false);

    // Custom message
    let x = 5;
    assert_eq!(x, 5, "x should be 5, but was {}", x);
}
```

### Testing Errors

```rust
#[derive(Debug, PartialEq)]
enum DivideError {
    DivisionByZero,
}

fn divide(a: i32, b: i32) -> Result<i32, DivideError> {
    if b == 0 {
        Err(DivideError::DivisionByZero)
    } else {
        Ok(a / b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_divide_success() {
        assert_eq!(divide(10, 2), Ok(5));
    }

    #[test]
    fn test_divide_by_zero() {
        assert_eq!(divide(10, 0), Err(DivideError::DivisionByZero));
    }
}
```

### Testing Panics

```rust
fn divide_panic(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division by zero");
    }
    a / b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_divide_panic() {
        divide_panic(10, 0);
    }
}
```

### Returning Result from Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_with_result() -> Result<(), String> {
        if 2 + 2 == 4 {
            Ok(())
        } else {
            Err(String::from("Math is broken"))
        }
    }
}
```

Useful when testing code that returns `Result`.

### Ignoring Tests

```rust
#[test]
#[ignore]
fn expensive_test() {
    // Only runs with: cargo test -- --ignored
}
```

## Integration Tests

Integration tests live in the `tests/` directory and test your library's public API.

```rust
// tests/integration_test.rs
use my_project::add;  // Import from your crate

#[test]
fn test_add() {
    assert_eq!(add(2, 2), 4);
}
```

Run with:
```bash
cargo test  # Runs both unit and integration tests
```

**Key difference**: Integration tests only have access to **public** APIs, simulating external usage.

### Organizing Integration Tests

```
tests/
├── common/
│   └── mod.rs       # Shared test utilities (not a test file)
├── integration_test.rs
└── another_test.rs
```

```rust
// tests/common/mod.rs
pub fn setup() {
    // Common setup code
}

// tests/integration_test.rs
mod common;

#[test]
fn test_something() {
    common::setup();
    // Test code
}
```

## Doc Tests

Code examples in doc comments are automatically tested!

```rust
/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// let result = my_project::add(2, 2);
/// assert_eq!(result, 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Run with:
```bash
cargo test --doc
```

**Benefits**:
- Documentation stays up-to-date
- Examples are guaranteed to work

### Doc Test Options

```rust
/// # Panics
///
/// ```should_panic
/// my_project::divide(10, 0);
/// ```

/// # Compile-time failures
///
/// ```compile_fail
/// let x: i32 = "not a number";
/// ```

/// # Ignored examples
///
/// ```ignore
/// // This won't be tested
/// ```

/// # Hidden lines
///
/// ```
/// # fn expensive_setup() {}
/// # expensive_setup();
/// let result = my_project::add(2, 2);
/// assert_eq!(result, 4);
/// ```
/// // Lines starting with # are hidden in docs but run in tests
```

## Test Organization

### Running Specific Tests

```bash
# Run all tests
cargo test

# Run tests matching a name
cargo test test_add

# Run tests in a specific module
cargo test tests::math

# Run ignored tests
cargo test -- --ignored

# Show println! output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test

# Run tests serially
cargo test -- --test-threads=1
```

### Conditional Compilation

```rust
#[cfg(test)]
mod tests {
    // Only compiled when testing
}

#[cfg(not(test))]
fn production_only() {
    // Not compiled during tests
}

// Platform-specific
#[cfg(target_os = "linux")]
fn linux_specific() {}

#[cfg(windows)]
fn windows_specific() {}
```

## Property-Based Testing

Property-based testing generates random inputs to test invariants. Like Python's Hypothesis or TypeScript's fast-check.

### Using proptest

```toml
# Cargo.toml
[dev-dependencies]
proptest = "1.0"
```

```rust
#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    proptest! {
        #[test]
        fn test_add_commutative(a in 0..1000i32, b in 0..1000i32) {
            assert_eq!(add(a, b), add(b, a));
        }

        #[test]
        fn test_add_associative(a in 0..100i32, b in 0..100i32, c in 0..100i32) {
            assert_eq!(add(add(a, b), c), add(a, add(b, c)));
        }

        #[test]
        fn parse_doesnt_crash(s in "\\PC*") {
            let _ = s.parse::<i32>();  // Shouldn't panic
        }
    }
}
```

**Benefits**:
- Find edge cases you didn't think of
- Test invariants, not specific examples

### Strategies (Generators)

```rust
use proptest::prelude::*;

proptest! {
    // Generate strings
    #[test]
    fn test_string(s in ".*") {
        assert!(s.len() < 10000);
    }

    // Generate vectors
    #[test]
    fn test_vec(v in prop::collection::vec(0..100i32, 0..50)) {
        assert!(v.len() < 50);
    }

    // Generate custom types
    #[derive(Debug, Clone)]
    struct Person {
        name: String,
        age: u8,
    }

    fn person_strategy() -> impl Strategy<Value = Person> {
        ("[a-z]{5,10}", 0..100u8).prop_map(|(name, age)| Person { name, age })
    }

    #[test]
    fn test_person(person in person_strategy()) {
        assert!(person.age < 100);
    }
}
```

## Benchmarking

Rust has built-in benchmarking, but it requires nightly. For stable Rust, use **Criterion**.

### Using Criterion

```toml
# Cargo.toml
[dev-dependencies]
criterion = "0.5"

[[bench]]
name = "my_benchmark"
harness = false
```

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use my_project::add;

fn benchmark_add(c: &mut Criterion) {
    c.bench_function("add", |b| {
        b.iter(|| {
            add(black_box(2), black_box(2))  // black_box prevents optimization
        })
    });
}

criterion_group!(benches, benchmark_add);
criterion_main!(benches);
```

Run with:
```bash
cargo bench
```

**Output**:
```
add                     time:   [123.45 ps 125.67 ps 127.89 ps]
                        change: [-2.3456% +0.1234% +2.5678%]
```

### Comparing Performance

```rust
fn benchmark_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("sorting");

    let data: Vec<i32> = (0..1000).collect();

    group.bench_function("vec_sort", |b| {
        b.iter(|| {
            let mut v = data.clone();
            v.sort();
        });
    });

    group.bench_function("vec_sort_unstable", |b| {
        b.iter(|| {
            let mut v = data.clone();
            v.sort_unstable();
        });
    });

    group.finish();
}
```

## Development Workflow

### 1. cargo check (Fast Feedback)

```bash
cargo check
```

**When**: After every small change. Faster than `cargo build` (doesn't generate code).

### 2. cargo clippy (Lints)

```bash
cargo clippy
```

**When**: Before committing. Catches common mistakes and suggests improvements.

Example warnings:
- Using `.clone()` unnecessarily
- Verbose pattern matching
- Performance anti-patterns

### 3. cargo fmt (Format)

```bash
cargo fmt
```

**When**: Before committing. Enforces consistent style.

Configure in `.rustfmt.toml`:
```toml
max_width = 100
tab_spaces = 4
```

### 4. cargo test (Tests)

```bash
cargo test
```

**When**: After implementing features, before committing.

### 5. cargo build --release (Optimized Build)

```bash
cargo build --release
```

**When**: Benchmarking or deploying. Much slower to compile, but generates optimized code.

**Performance difference**: 10-100x faster than debug builds.

### Recommended Workflow

```bash
# 1. Make changes
# 2. Quick check
cargo check

# 3. Run tests
cargo test

# 4. Lint
cargo clippy

# 5. Format
cargo fmt

# 6. Commit
git add .
git commit -m "Add feature X"
```

### CI/CD Script

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo check
      - run: cargo test
      - run: cargo clippy -- -D warnings
      - run: cargo fmt -- --check
```

## Test-Driven Development in Rust

### Red-Green-Refactor

```rust
// 1. Red: Write failing test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_email() {
        let email = "user@example.com";
        let parsed = parse_email(email).unwrap();
        assert_eq!(parsed.local, "user");
        assert_eq!(parsed.domain, "example.com");
    }
}

// 2. Green: Make it pass (simplest way)
struct Email {
    local: String,
    domain: String,
}

fn parse_email(s: &str) -> Option<Email> {
    let parts: Vec<_> = s.split('@').collect();
    if parts.len() == 2 {
        Some(Email {
            local: parts[0].to_string(),
            domain: parts[1].to_string(),
        })
    } else {
        None
    }
}

// 3. Refactor: Improve without breaking tests
fn parse_email(s: &str) -> Option<Email> {
    let mut parts = s.split('@');
    let local = parts.next()?.to_string();
    let domain = parts.next()?.to_string();

    if parts.next().is_some() {
        return None;  // Multiple @
    }

    Some(Email { local, domain })
}
```

## Comparing to Other Languages

### TypeScript (Jest)

```typescript
// sum.test.ts
import { sum } from './sum';

test('adds 1 + 2 to equal 3', () => {
  expect(sum(1, 2)).toBe(3);
});
```

**Differences**:
- TypeScript: Separate test files, external framework
- Rust: Tests in same file (unit) or `tests/` (integration), built-in

### Python (pytest)

```python
# test_sum.py
from sum import add

def test_add():
    assert add(2, 2) == 4
```

**Differences**:
- Python: Convention-based (test_*.py, test_*), external framework
- Rust: Attribute-based (`#[test]`), built-in

**Rust advantages**:
- No external dependencies for basic testing
- Integrated with build system
- Doc tests ensure documentation is correct

## Best Practices

### 1. Test Public APIs in Integration Tests

```rust
// tests/integration.rs
use my_crate::public_function;

#[test]
fn test_public_api() {
    assert_eq!(public_function(), expected_value);
}
```

### 2. Test Edge Cases

```rust
#[test]
fn test_empty_input() {
    assert_eq!(process(&[]), expected_empty_result);
}

#[test]
fn test_large_input() {
    let large = vec![0; 10_000];
    assert!(process(&large).is_ok());
}
```

### 3. Use Descriptive Test Names

```rust
#[test]
fn adding_two_positive_numbers_returns_sum() {
    assert_eq!(add(2, 2), 4);
}

#[test]
fn adding_positive_and_negative_returns_difference() {
    assert_eq!(add(5, -3), 2);
}
```

### 4. Test Invariants with Property Tests

```rust
proptest! {
    #[test]
    fn reversing_twice_is_identity(v in prop::collection::vec(0..100i32, 0..50)) {
        let mut v1 = v.clone();
        v1.reverse();
        v1.reverse();
        assert_eq!(v, v1);
    }
}
```

### 5. Use Test Fixtures

```rust
#[cfg(test)]
mod tests {
    fn setup() -> TestContext {
        TestContext {
            // Setup common state
        }
    }

    #[test]
    fn test_something() {
        let ctx = setup();
        // Use ctx
    }
}
```

## Key Takeaways

1. **Cargo** is the all-in-one tool for building, testing, and managing Rust projects
2. **Unit tests** live in the same file, **integration tests** in `tests/`
3. **Doc tests** ensure documentation examples work
4. Use **proptest** for property-based testing
5. Use **Criterion** for benchmarking on stable Rust
6. **cargo check** for fast feedback during development
7. **cargo clippy** catches common mistakes
8. **cargo fmt** enforces consistent style
9. **TDD workflow**: Red-Green-Refactor with `cargo test --watch`

## Practice

Before moving on, ensure you can:

- [ ] Write unit tests with `#[test]`
- [ ] Write integration tests in `tests/`
- [ ] Write doc tests in documentation
- [ ] Use `cargo test`, `cargo check`, `cargo clippy`, `cargo fmt`
- [ ] Understand when to use unit vs integration tests
- [ ] Write property-based tests with proptest
- [ ] Benchmark code with Criterion

## Next Steps

Now you're ready for the exercises!

Start with [Exercise 01: LRU Cache →](../exercises/ex01-lru-cache/) to put all these concepts into practice.

Then continue with:
- [Exercise 02: Config-Driven CLI →](../exercises/ex02-config-cli/)
- [Exercise 03: Text Search Tool →](../exercises/ex03-text-search/)

After completing the exercises, move to [Module 02: Async + Networking →](../../module-02-async-networking/) to learn async Rust.
