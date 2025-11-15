# Module 01: Core Rust Fluency

**For Experienced Engineers: Skip the Basics, Master the Unique**

## Overview

This module is designed to make Rust feel comfortable and the compiler your ally, not your enemy. You already know how to program—you've built systems in TypeScript, Python, and other languages. This module focuses exclusively on what makes Rust *different* and *powerful*.

**Duration**: 1-2 weeks (10-15 hours)

## Learning Objectives

By the end of this module, you will:

1. **Think in ownership** - Understand Rust's ownership model deeply enough to design APIs that "feel right"
2. **Leverage the type system** - Use enums, pattern matching, and traits to model complex domains
3. **Handle errors idiomatically** - Move beyond try/catch to Result-based error handling
4. **Write testable code** - Structure code for unit and property-based testing
5. **Read and understand** any intermediate-level Rust codebase

## What Makes Rust Different

Coming from TypeScript/Python, here's what's fundamentally different:

| Concept | TypeScript/Python | Rust |
|---------|------------------|------|
| **Memory Management** | GC (automatic) | Ownership (compile-time tracked) |
| **Null Handling** | `null`/`undefined`/`None` | `Option<T>` (enforced by type system) |
| **Error Handling** | Exceptions (throw/catch) | `Result<T, E>` (values, not control flow) |
| **Concurrency** | Event loop / GIL | Fearless (compiler-checked data races) |
| **Type System** | Structural / duck typing | Nominal + traits (explicit contracts) |
| **Performance** | Runtime overhead | Zero-cost abstractions |

## Module Structure

### Lectures (Read in Order)

1. **[Ownership and Borrowing](./lectures/01-ownership-and-borrowing.md)**
   - The single most important concept in Rust
   - Mental models: move semantics, borrowing rules, lifetime basics
   - Comparisons to references in other languages

2. **[Lifetimes Demystified](./lectures/02-lifetimes.md)**
   - When and why lifetimes matter
   - Practical patterns, not just theory
   - How to read lifetime errors and fix them

3. **[Enums and Pattern Matching](./lectures/03-enums-and-patterns.md)**
   - Algebraic data types (ADTs) and why they're powerful
   - Exhaustive matching and the compiler's help
   - Modeling complex states with types

4. **[Error Handling the Rust Way](./lectures/04-error-handling.md)**
   - `Result<T, E>` and `Option<T>` in depth
   - The `?` operator and error propagation
   - Custom error types and thiserror/anyhow
   - Comparing to exceptions in TS/Python

5. **[Traits and Generics](./lectures/05-traits-and-generics.md)**
   - Traits as contracts (like interfaces, but more powerful)
   - Trait bounds and where clauses
   - Associated types vs generic parameters
   - Trait objects and dynamic dispatch

6. **[Collections and Iterators](./lectures/06-collections-iterators.md)**
   - Vec, HashMap, HashSet - when to use each
   - Iterator patterns and zero-cost abstractions
   - Comparison to array methods in TS, generators in Python

7. **[Testing and Development Workflow](./lectures/07-testing.md)**
   - Unit tests, integration tests, doc tests
   - Property-based testing with proptest
   - Benchmarking basics
   - Cargo workflow and useful commands

### Exercises (Hands-On Practice)

**Exercise 1: LRU Cache**
- Build a least-recently-used cache from scratch
- **Focus**: Ownership, interior mutability, generic types
- **Difficulty**: Medium
- **Time**: 3-4 hours

**Exercise 2: Config-Driven CLI Tool**
- Build a tool that reads TOML/JSON config and processes data
- **Focus**: Error handling, file I/O, serialization
- **Difficulty**: Easy-Medium
- **Time**: 2-3 hours

**Exercise 3: Text Search Tool (Inverted Index)**
- Build a simple search engine over local files
- **Focus**: Collections, iterators, performance thinking
- **Difficulty**: Medium-Hard
- **Time**: 4-5 hours

### Solutions (Reference Implementations)

Each exercise has a complete solution with:
- Fully working, tested code
- `COMMENTARY.md` explaining design choices
- Common pitfalls and how to avoid them
- Performance considerations
- Alternative approaches and tradeoffs

## How to Approach This Module

### 1. Read Lectures First
Don't skip ahead to exercises. The lectures build mental models you'll need.

### 2. Fight the Compiler (At First)
Your first Rust programs will not compile. This is *normal* and *good*. The compiler is teaching you.

### 3. Trust the Error Messages
Rust's error messages are among the best in any language. Read them carefully.

### 4. Try Before Looking at Solutions
Struggle is part of learning. Spend at least 30 minutes on each exercise before checking solutions.

### 5. Compare to What You Know
Throughout, think: "How would I do this in TypeScript? In Python?" Understanding the differences deepens both.

## Key Concepts Preview

### Ownership (The Big One)

```rust
// This is invalid Rust
let s1 = String::from("hello");
let s2 = s1;  // s1 is *moved* to s2
println!("{}", s1);  // ❌ Compile error: s1 no longer valid
```

**Why it matters**: No garbage collector, no dangling pointers, no data races. All checked at compile time.

**Mental model**: Think of ownership as a *linear type* - exactly one owner at a time.

### Borrowing (References That Don't Outlive)

```rust
fn print_length(s: &String) {  // Borrows, doesn't own
    println!("{}", s.len());
}

let s = String::from("hello");
print_length(&s);  // s is borrowed
println!("{}", s);  // ✅ s still valid
```

**Rule**: Many immutable borrows OR one mutable borrow, never both.

**Mental model**: Borrowing is like read-write locks, but checked at compile time.

### Enums as Data (Not Just Flags)

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

match msg {
    Message::Quit => exit(),
    Message::Move { x, y } => move_to(x, y),
    Message::Write(text) => write(text),
    Message::ChangeColor(r, g, b) => set_color(r, g, b),
}
```

**Why it matters**: Model complex state machines with types, not stringly-typed systems.

**Mental model**: Like TypeScript discriminated unions, but more powerful.

### Results as Values

```rust
fn parse_config(path: &str) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;  // ? propagates errors
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

// Usage
match parse_config("config.toml") {
    Ok(config) => run_with_config(config),
    Err(e) => eprintln!("Failed to load config: {}", e),
}
```

**Why it matters**: Errors are values, not exceptions. Type system forces you to handle them.

**Mental model**: Like Promise.then().catch() but synchronous and type-safe.

## Prerequisites

### Assumed Knowledge
- You know what a hash table is
- You understand Big-O notation
- You've used generics/templates before
- You know what a race condition is

### Not Assumed
- Any Rust syntax
- How ownership works
- Rust idioms and patterns

## Success Criteria

You're ready for Module 02 when you can:

1. ✅ Explain ownership and borrowing to a colleague
2. ✅ Read and understand intermediate Rust code without struggling
3. ✅ Fix common compiler errors (borrow checker, lifetime errors)
4. ✅ Write idiomatic error handling with Result and ?
5. ✅ Use pattern matching effectively
6. ✅ Implement a generic data structure with trait bounds

## Common Questions

**Q: Why is Rust so strict about ownership?**
A: To eliminate entire classes of bugs at compile time: use-after-free, double-free, data races, null pointer dereferences. You trade compiler fights for runtime reliability.

**Q: When should I use &String vs &str?**
A: Almost always use `&str` for function parameters. It's more flexible (works with String, &str, string literals).

**Q: Do I need to understand lifetimes to be productive?**
A: For basic code, the compiler often infers them. But understanding lifetimes unlocks advanced patterns and helps you read library code.

**Q: How does this compare to TypeScript's type system?**
A: Rust's type system is more powerful (prevents memory bugs, data races) but less flexible (no `any`, stricter rules). Both have excellent inference.

**Q: Is Rust slower to write than Python/TypeScript?**
A: Initially, yes. After this module, you'll be ~70% as fast. After a few months, just as fast, with fewer runtime bugs.

## Next Steps

After completing this module:

1. **Review your solutions** against the reference implementations
2. **Write a small project** combining concepts (suggestions in exercises)
3. **Move to Module 02** for async Rust and networking

---

**Ready to start?**

Begin with [Lecture 01: Ownership and Borrowing →](./lectures/01-ownership-and-borrowing.md)

Or jump to [Exercise 01: LRU Cache →](./exercises/ex01-lru-cache/) if you learn by doing.
