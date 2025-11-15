# Lecture 05: Traits and Generics

**Abstraction Without Runtime Cost**

## Introduction

Traits are Rust's mechanism for defining shared behavior—similar to interfaces in TypeScript or protocols in Python (via ABC). Combined with generics, they enable **polymorphism** without inheritance or runtime overhead.

If you're coming from TypeScript, traits will feel familiar but more powerful. If you're from Python, think of them as a compile-time enforced version of duck typing.

## Traits: Defining Shared Behavior

A trait defines a set of methods that types can implement.

### Basic Trait Definition

```rust
trait Summary {
    fn summarize(&self) -> String;
}
```

This defines a contract: any type that implements `Summary` must provide a `summarize` method.

### Implementing a Trait

```rust
struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
}

struct Tweet {
    username: String,
    content: String,
}

impl Summary for Tweet {
    fn summarize(&self) -> String {
        format!("@{}: {}", self.username, self.content)
    }
}

// Usage
let article = Article { /*...*/ };
let tweet = Tweet { /*...*/ };

println!("{}", article.summarize());
println!("{}", tweet.summarize());
```

### Default Implementations

Traits can provide default method implementations:

```rust
trait Summary {
    fn summarize(&self) -> String {
        String::from("(Read more...)")
    }

    fn summarize_author(&self) -> String;

    fn full_summary(&self) -> String {
        format!("Summary: {}, by {}", self.summarize(), self.summarize_author())
    }
}

impl Summary for Article {
    // Only implement what's required
    fn summarize_author(&self) -> String {
        self.author.clone()
    }
    // summarize() and full_summary() use defaults
}
```

## Traits as Parameters

You can require parameters to implement a trait:

### Syntax 1: impl Trait

```rust
fn print_summary(item: &impl Summary) {
    println!("{}", item.summarize());
}

// Works with any type that implements Summary
print_summary(&article);
print_summary(&tweet);
```

### Syntax 2: Trait Bounds

```rust
fn print_summary<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}
```

These are equivalent. The generic version is more flexible for complex bounds.

### Multiple Trait Bounds

```rust
fn process<T: Summary + Display>(item: &T) {
    println!("{}", item);
    println!("{}", item.summarize());
}

// Or with where clause (more readable for complex bounds)
fn process<T>(item: &T)
where
    T: Summary + Display + Clone,
{
    // ...
}
```

## Returning Traits

You can return types that implement a trait:

```rust
fn make_summary() -> impl Summary {
    Article {
        title: String::from("Example"),
        author: String::from("Author"),
        content: String::from("Content"),
    }
}
```

**Limitation**: You can only return **one concrete type**:

```rust
// ❌ Error: can't return different types
fn make_summary(use_tweet: bool) -> impl Summary {
    if use_tweet {
        Tweet { /*...*/ }
    } else {
        Article { /*...*/ }
    }
}
```

For dynamic dispatch, use trait objects (covered later).

## Common Standard Library Traits

### Clone

```rust
trait Clone {
    fn clone(&self) -> Self;
}

// Usually derived
#[derive(Clone)]
struct Point {
    x: i32,
    y: i32,
}

let p1 = Point { x: 1, y: 2 };
let p2 = p1.clone();  // Explicit copy
```

### Copy

```rust
// Marker trait (no methods)
// Only for types that can be copied bit-by-bit
#[derive(Copy, Clone)]  // Copy requires Clone
struct Point {
    x: i32,
    y: i32,
}

let p1 = Point { x: 1, y: 2 };
let p2 = p1;  // p1 is copied, not moved
println!("{} {}", p1.x, p2.x);  // Both valid
```

**Copy vs Clone**:
- `Copy`: Implicit, for simple stack-only types
- `Clone`: Explicit, can be expensive (heap allocations)

### Debug

```rust
#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

let user = User { name: "Alice".to_string(), age: 30 };
println!("{:?}", user);  // Debug format
println!("{:#?}", user);  // Pretty-print
```

### PartialEq and Eq

```rust
#[derive(PartialEq, Eq)]
struct Point {
    x: i32,
    y: i32,
}

let p1 = Point { x: 1, y: 2 };
let p2 = Point { x: 1, y: 2 };
assert_eq!(p1, p2);
```

**PartialEq**: Defines `==` and `!=` (may not be reflexive, e.g., `NaN != NaN`)
**Eq**: Marker trait indicating full equivalence relation

### PartialOrd and Ord

```rust
#[derive(PartialOrd, Ord, PartialEq, Eq)]
struct Person {
    age: u32,
    name: String,
}

let people = vec![/* ... */];
people.sort();  // Requires Ord
```

### Display

```rust
use std::fmt;

struct Point {
    x: i32,
    y: i32,
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

let p = Point { x: 1, y: 2 };
println!("{}", p);  // Uses Display
```

### From and Into

```rust
struct Celsius(f64);

impl From<Fahrenheit> for Celsius {
    fn from(f: Fahrenheit) -> Self {
        Celsius((f.0 - 32.0) * 5.0 / 9.0)
    }
}

let f = Fahrenheit(98.6);
let c: Celsius = f.into();  // Into is automatic from From
let c = Celsius::from(f);   // Explicit From
```

**Best practice**: Implement `From`; `Into` is provided automatically.

### Default

```rust
#[derive(Default)]
struct Config {
    host: String,      // Empty string
    port: u16,         // 0
    timeout: u64,      // 0
}

let config = Config::default();

// Custom implementation
impl Default for Config {
    fn default() -> Self {
        Config {
            host: "localhost".to_string(),
            port: 8080,
            timeout: 30,
        }
    }
}
```

## Generics

Generics allow you to write code that works with multiple types.

### Generic Functions

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut largest = &list[0];
    for item in list {
        if item > largest {
            largest = item;
        }
    }
    largest
}

let numbers = vec![34, 50, 25, 100, 65];
println!("Largest: {}", largest(&numbers));

let chars = vec!['y', 'm', 'a', 'q'];
println!("Largest: {}", largest(&chars));
```

### Generic Structs

```rust
struct Point<T> {
    x: T,
    y: T,
}

let int_point = Point { x: 5, y: 10 };
let float_point = Point { x: 1.0, y: 4.0 };
```

### Multiple Type Parameters

```rust
struct Point<T, U> {
    x: T,
    y: U,
}

let point = Point { x: 5, y: 4.0 };  // x is i32, y is f64
```

### Generic Methods

```rust
impl<T> Point<T> {
    fn x(&self) -> &T {
        &self.x
    }
}

// Method only for specific type
impl Point<f64> {
    fn distance_from_origin(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }
}
```

### Generic Enums

```rust
enum Option<T> {
    Some(T),
    None,
}

enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

You're already using generics—`Option` and `Result` are generic!

## Associated Types

Associated types are a way to define placeholder types in traits.

### Without Associated Types

```rust
trait Iterator<T> {
    fn next(&mut self) -> Option<T>;
}

// Usage is verbose
fn process<T, I: Iterator<T>>(iter: I) { /*...*/ }
```

### With Associated Types

```rust
trait Iterator {
    type Item;  // Associated type
    fn next(&mut self) -> Option<Self::Item>;
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        // ...
    }
}

// Cleaner usage
fn process<I: Iterator>(iter: I) { /*...*/ }

// Access associated type
fn sum<I: Iterator<Item = i32>>(iter: I) -> i32 {
    iter.fold(0, |acc, x| acc + x)
}
```

**When to use**:
- **Associated types**: When there's **one logical type** for a given implementation (e.g., iterator items)
- **Generic parameters**: When a trait might be implemented **multiple times** for a type with different type parameters

## Trait Objects: Dynamic Dispatch

When you need to return different types from a function, use **trait objects**.

### Box<dyn Trait>

```rust
trait Shape {
    fn area(&self) -> f64;
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

// Returning trait objects
fn make_shape(circle: bool) -> Box<dyn Shape> {
    if circle {
        Box::new(Circle { radius: 5.0 })
    } else {
        Box::new(Rectangle { width: 10.0, height: 5.0 })
    }
}

// Heterogeneous collections
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 5.0 }),
    Box::new(Rectangle { width: 10.0, height: 5.0 }),
];

for shape in shapes {
    println!("Area: {}", shape.area());
}
```

**Tradeoff**:
- **Static dispatch** (`impl Trait`, generics): Monomorphization, no runtime overhead, larger binary
- **Dynamic dispatch** (`dyn Trait`): Virtual method table (vtable), small runtime overhead, smaller binary

### Object Safety

Not all traits can be trait objects. A trait is **object-safe** if:
1. No return type of `Self`
2. No generic type parameters

```rust
trait Cloneable {
    fn clone(&self) -> Self;  // ❌ Not object-safe (returns Self)
}

// ✅ This is object-safe
trait Draw {
    fn draw(&self);
}
```

## Advanced Trait Patterns

### Supertraits

```rust
trait OutlinePrint: fmt::Display {  // Requires Display
    fn outline_print(&self) {
        println!("*{}*", self);  // Can use Display methods
    }
}
```

### Newtype Pattern

Implement external traits on external types:

```rust
// Can't do: impl fmt::Display for Vec<T> (both external)

// Wrapper type
struct Wrapper(Vec<String>);

impl fmt::Display for Wrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]", self.0.join(", "))
    }
}

let w = Wrapper(vec!["hello".to_string(), "world".to_string()]);
println!("{}", w);  // [hello, world]
```

### Fully Qualified Syntax

Disambiguate when multiple traits have the same method name:

```rust
trait Pilot {
    fn fly(&self);
}

trait Wizard {
    fn fly(&self);
}

struct Human;

impl Pilot for Human {
    fn fly(&self) { println!("Captain speaking"); }
}

impl Wizard for Human {
    fn fly(&self) { println!("Up!"); }
}

impl Human {
    fn fly(&self) { println!("*waving arms*"); }
}

let person = Human;
person.fly();  // Calls Human::fly
Pilot::fly(&person);  // Calls Pilot::fly
Wizard::fly(&person);  // Calls Wizard::fly

// For associated functions (no self)
<Human as Pilot>::fly(&person);
```

### Blanket Implementations

Implement a trait for all types that satisfy a bound:

```rust
// From std::lib
impl<T: Display> ToString for T {
    fn to_string(&self) -> String {
        // ...
    }
}

// Now any type with Display automatically has to_string()
let num = 42;
let s = num.to_string();  // Works because i32: Display
```

## Comparing to Other Languages

### TypeScript Interfaces

```typescript
interface Summary {
    summarize(): string;
}

class Article implements Summary {
    summarize(): string {
        return "Article summary";
    }
}

function printSummary(item: Summary) {
    console.log(item.summarize());
}
```

**Differences**:
- TypeScript: Structural typing (if it has the right shape, it matches)
- Rust: Nominal typing (must explicitly implement the trait)
- TypeScript: No zero-cost abstraction guarantee
- Rust: Generics are monomorphized (zero runtime cost)

### Python Protocols / ABC

```python
from abc import ABC, abstractmethod

class Summary(ABC):
    @abstractmethod
    def summarize(self) -> str:
        pass

class Article(Summary):
    def summarize(self) -> str:
        return "Article summary"

def print_summary(item: Summary):
    print(item.summarize())
```

**Differences**:
- Python: Duck typing at runtime, ABC is optional enforcement
- Rust: Compile-time enforcement, no runtime checks
- Python: Runtime polymorphism (dynamic dispatch)
- Rust: Choose static or dynamic dispatch explicitly

## Zero-Cost Abstractions

Rust generics are **monomorphized**—the compiler generates specific code for each concrete type.

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    // ...
}

let numbers = vec![1, 2, 3];
largest(&numbers);  // Compiler generates largest_i32

let chars = vec!['a', 'b', 'c'];
largest(&chars);  // Compiler generates largest_char
```

**Result**: No runtime overhead. As fast as hand-written code for each type.

**Tradeoff**: Larger binary size (code duplication) vs dynamic dispatch (single implementation + vtable lookup).

## Best Practices

### 1. Prefer Generics Over Trait Objects When Possible

```rust
// ✅ Better: static dispatch
fn process<T: Summary>(item: &T) {
    println!("{}", item.summarize());
}

// Use only when you need heterogeneous collections or return different types
fn make_item() -> Box<dyn Summary> { /*...*/ }
```

### 2. Use impl Trait for Simple Cases

```rust
// ✅ Simpler
fn make_summary() -> impl Summary { /*...*/ }

// Use generics when you need to name the type or have complex bounds
fn process<T: Summary + Clone>(item: T) -> T { /*...*/ }
```

### 3. Derive Common Traits

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct User {
    name: String,
    age: u32,
}
```

### 4. Use Where Clauses for Readability

```rust
// ❌ Hard to read
fn process<T: Summary + Display + Clone, U: Summary + Debug>(item1: T, item2: U) { /*...*/ }

// ✅ Clearer
fn process<T, U>(item1: T, item2: U)
where
    T: Summary + Display + Clone,
    U: Summary + Debug,
{
    // ...
}
```

### 5. Implement Traits Consistently

```rust
// If you implement PartialEq, consider Eq
// If you implement PartialOrd, consider Ord
// If you implement Display, consider Debug
```

## Key Takeaways

1. **Traits** define shared behavior (like interfaces)
2. **Generics** enable code reuse without runtime cost
3. **Trait bounds** constrain generic types (`T: Summary`)
4. **Associated types** simplify trait definitions
5. **Static dispatch** (generics) = zero-cost, larger binary
6. **Dynamic dispatch** (`dyn Trait`) = runtime cost, flexibility
7. **Derive** common traits when possible
8. **Blanket implementations** provide traits for many types at once
9. Rust traits are **nominal** (explicit `impl`), TypeScript interfaces are **structural**

## Practice

Before moving on, ensure you can:

- [ ] Define and implement a trait
- [ ] Use trait bounds with generics
- [ ] Understand the difference between `Clone` and `Copy`
- [ ] Use `impl Trait` in function signatures
- [ ] Create trait objects with `Box<dyn Trait>`
- [ ] Implement common std traits (Debug, Display, PartialEq)
- [ ] Understand when to use static vs dynamic dispatch

## Next Steps

Read [Lecture 06: Collections and Iterators →](./06-collections-iterators.md) to learn how to work with Rust's collection types effectively.

Or practice traits in [Exercise 01: LRU Cache →](../exercises/ex01-lru-cache/), which uses generic types and trait bounds extensively.
