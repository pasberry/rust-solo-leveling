# Lecture 02: Lifetimes Demystified

**Understanding Reference Validity Without the Panic**

## Introduction

Lifetimes are Rust's way of ensuring references are always valid. They're the mechanism behind the "references must always be valid" rule from the previous lecture.

**The good news**: For most code, the compiler infers lifetimes automatically. You don't need to think about them.

**The challenge**: When building libraries, complex data structures, or certain patterns, you'll need to understand lifetime annotations.

This lecture focuses on **practical lifetime understanding**—when they matter, how to read them, and how to fix errors.

## What Are Lifetimes?

A lifetime is the **scope** for which a reference is valid.

```rust
{
    let r;                    // -------+-- 'a
                              //        |
    {                         //        |
        let x = 5;            // -+-- 'b|
        r = &x;               //  |     |
    }                         // -+     |
                              //        |
    println!("r: {}", r);     //        |
}                             // -------+
```

The lifetime `'b` is shorter than `'a`. The reference `r` with lifetime `'a` tries to point to data with lifetime `'b`. This is **invalid**—the reference would outlive the data.

**Compile error**:
```
error[E0597]: `x` does not live long enough
```

The compiler caught a use-after-free bug at compile time!

## Lifetime Annotations

Lifetime annotations describe the relationship between the lifetimes of references. They don't change how long references live—they describe constraints.

### Syntax

```rust
&'a T       // A reference to T with lifetime 'a
&'a mut T   // A mutable reference to T with lifetime 'a
```

`'a` is a **lifetime parameter**, like generic type parameters.

### When Do You Need Them?

The compiler can often infer lifetimes. You need annotations when:

1. **Multiple references** are involved and the compiler can't infer relationships
2. **Returning references** from functions
3. **Structs** that hold references
4. **Trait implementations** with references

Let's explore each case.

## Function Signatures

### Case 1: Single Reference (Inferred)

```rust
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

No lifetime annotations needed! The compiler infers:
- The returned reference has the same lifetime as the input

Explicit version (you don't write this):
```rust
fn first_word<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}
```

### Case 2: Multiple References (Must Annotate)

```rust
// ❌ Won't compile without lifetimes
fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

**Error**: "expected named lifetime parameter"

**Why?** The compiler doesn't know if the returned reference comes from `x` or `y`. It can't determine the lifetime of the return value.

**Fix**: Annotate the relationship:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}
```

**Meaning**: "Both inputs live at least as long as `'a`, and the return value also lives for `'a`."

**Usage**:
```rust
let string1 = String::from("long string");
let result;

{
    let string2 = String::from("short");
    result = longest(&string1, &string2);
    println!("{}", result);  // ✅ OK: result used before string2 is dropped
}

// println!("{}", result);  // ❌ Error: result might point to string2, which is dropped
```

The lifetime `'a` is the **shorter** of the two inputs' lifetimes.

### Case 3: Independent Lifetimes

```rust
fn first_and_append<'a, 'b>(x: &'a str, y: &'b mut String) -> &'a str {
    y.push_str(" appended");
    x
}
```

Here, `x` and `y` have **independent** lifetimes because:
- The return value only depends on `x`'s lifetime
- `y` is mutably borrowed but doesn't affect the return value

### Case 4: Returning a Reference

```rust
// ❌ Error: returning reference to local variable
fn dangle() -> &str {
    let s = String::from("hello");
    &s  // s is dropped here, reference would dangle!
}
```

**Fix**: Return an owned value instead:
```rust
fn no_dangle() -> String {
    let s = String::from("hello");
    s  // Ownership is transferred
}
```

**Rule**: You can only return references that were passed in (or to static data).

## Structs with Lifetimes

If a struct holds a reference, it needs a lifetime annotation.

```rust
struct Excerpt<'a> {
    part: &'a str,
}

impl<'a> Excerpt<'a> {
    fn get_part(&self) -> &str {
        self.part
    }
}

fn main() {
    let novel = String::from("Call me Ishmael. Some years ago...");
    let first_sentence = novel.split('.').next().unwrap();

    let excerpt = Excerpt {
        part: first_sentence,
    };

    println!("{}", excerpt.get_part());
}
```

**Meaning**: `Excerpt` holds a reference that must remain valid for the lifetime `'a`.

The instance of `Excerpt` **cannot outlive** the reference it holds.

```rust
let excerpt;
{
    let novel = String::from("Call me Ishmael.");
    let first = novel.split('.').next().unwrap();
    excerpt = Excerpt { part: first };
}
// println!("{}", excerpt.part);  // ❌ Error: novel is dropped
```

## Lifetime Elision Rules

The compiler uses **lifetime elision rules** to infer lifetimes in common cases. That's why you don't always write them.

### Three Elision Rules

1. **Each parameter gets its own lifetime**
   ```rust
   fn foo(x: &str, y: &str)
   // Becomes:
   fn foo<'a, 'b>(x: &'a str, y: &'b str)
   ```

2. **If there's exactly one input lifetime, it's assigned to all outputs**
   ```rust
   fn foo(x: &str) -> &str
   // Becomes:
   fn foo<'a>(x: &'a str) -> &'a str
   ```

3. **If there's a `&self` or `&mut self`, its lifetime is assigned to all outputs**
   ```rust
   impl Foo {
       fn bar(&self, x: &str) -> &str
       // Becomes:
       fn bar<'a, 'b>(&'a self, x: &'b str) -> &'a str
   }
   ```

If the compiler can't infer lifetimes after applying these rules, you must annotate manually.

## The 'static Lifetime

`'static` means the reference is valid for the **entire program duration**.

```rust
let s: &'static str = "I'm a string literal";
```

String literals have `'static` lifetime because they're baked into the binary.

### When to Use 'static

**Rarely.** Most references don't need to be `'static`.

Common use cases:
- String literals
- Global constants
- Leaked memory (intentionally never freed)

```rust
const MAX_SIZE: usize = 1024;  // 'static by definition

lazy_static! {
    static ref REGEX: Regex = Regex::new(r"\d+").unwrap();
}
```

**Common mistake**: Using `'static` when you just need to satisfy a trait bound. Often you want a generic lifetime instead.

## Lifetime Bounds

Just like traits, lifetimes can have bounds.

```rust
fn example<'a, T>(x: &'a T) where T: 'a {
    // T must live at least as long as 'a
}
```

**Meaning**: `T: 'a` means "T must not contain references that live shorter than `'a`."

This is often inferred, but sometimes needed when working with trait objects:

```rust
struct Ref<'a, T: 'a> {
    r: &'a T,
}
```

In modern Rust (2018 edition+), this bound is often implied.

## Common Patterns

### Pattern 1: Borrowed Data in Structs

```rust
struct Config<'a> {
    name: &'a str,
    timeout: u64,
}

impl<'a> Config<'a> {
    fn new(name: &'a str, timeout: u64) -> Self {
        Config { name, timeout }
    }
}

let name = String::from("my-service");
let config = Config::new(&name, 30);
// config cannot outlive name
```

**Tradeoff**: If you want `Config` to own its data, use `String` instead of `&str`.

### Pattern 2: Iterators

```rust
fn find_longest<'a>(words: &'a [String]) -> Option<&'a String> {
    words.iter().max_by_key(|w| w.len())
}

let words = vec![
    String::from("hello"),
    String::from("world"),
];

if let Some(longest) = find_longest(&words) {
    println!("Longest: {}", longest);
}
// longest cannot outlive words
```

### Pattern 3: Multiple Struct Lifetimes

```rust
struct Context<'s> {
    text: &'s str,
}

struct Parser<'c, 's> {
    context: &'c Context<'s>,
}

impl<'c, 's> Parser<'c, 's> {
    fn parse(&self) -> &'s str {
        self.context.text
    }
}
```

**Multiple lifetimes** when different references have different constraints.

## Comparing to Other Languages

### TypeScript/JavaScript

```typescript
interface Config {
    name: string;
}

function makeConfig(name: string): Config {
    return { name };  // name is copied (strings are immutable in JS)
}

let config: Config;
{
    let name = "my-service";
    config = makeConfig(name);  // ✅ OK: string is copied
}
console.log(config.name);  // ✅ No dangling reference
```

JavaScript/TypeScript don't have this problem because:
- Strings are immutable and cheap to copy
- GC prevents dangling references

But they also can't give you zero-cost abstractions or prevent use-after-free.

### C++

```cpp
struct Config {
    std::string_view name;  // Non-owning reference
};

Config make_config() {
    std::string name = "my-service";
    return Config { name };  // 💥 Dangling reference!
}
// C++ doesn't prevent this at compile time
```

C++ has the same potential for dangling references, but no compile-time enforcement.

### Rust's Advantage

Rust prevents dangling references at compile time while maintaining zero-cost abstractions.

## Reading Lifetime Errors

Let's practice reading errors.

### Error 1: "Does not live long enough"

```rust
fn main() {
    let r;
    {
        let x = 5;
        r = &x;
    }
    println!("{}", r);
}
```

**Error**:
```
error[E0597]: `x` does not live long enough
  --> src/main.rs:5:13
   |
5  |         r = &x;
   |             ^^ borrowed value does not live long enough
6  |     }
   |     - `x` dropped here while still borrowed
7  |     println!("{}", r);
   |                    - borrow later used here
```

**Fix**: Ensure `x` lives as long as `r` needs it:
```rust
let x = 5;
let r = &x;
println!("{}", r);
```

### Error 2: "Expected named lifetime parameter"

```rust
fn first(x: &str, y: &str) -> &str {
    x
}
```

**Error**:
```
error[E0106]: missing lifetime specifier
 --> src/lib.rs:1:35
  |
1 | fn first(x: &str, y: &str) -> &str {
  |             ----     ----     ^ expected named lifetime parameter
  |
  = help: this function's return type contains a borrowed value,
          but the signature does not say whether it is borrowed from `x` or `y`
```

**Fix**: Annotate the relationship:
```rust
fn first<'a>(x: &'a str, _y: &str) -> &'a str {
    x
}
```

Note: `y` doesn't need `'a` because it's not used in the return value.

### Error 3: "Lifetime mismatch"

```rust
struct Foo<'a> {
    x: &'a str,
}

impl Foo<'static> {  // ❌ Too restrictive
    fn new(x: &str) -> Self {
        Foo { x }
    }
}
```

**Fix**: Use the lifetime parameter properly:
```rust
impl<'a> Foo<'a> {
    fn new(x: &'a str) -> Self {
        Foo { x }
    }
}
```

## Mental Models

### Mental Model 1: Lifetimes are Scopes

Think of lifetimes as representing the scope in which data is valid.

```rust
{                       // 'a starts
    let x = 5;
    {                   // 'b starts
        let y = &x;     // y has lifetime 'b, borrows from 'a
    }                   // 'b ends
}                       // 'a ends
```

`'b` is a subset of `'a` (written `'b: 'a` or "`'b` outlives `'a`").

### Mental Model 2: Lifetimes as Constraints

Lifetime annotations are **constraints** on the caller, not instructions to the compiler.

```rust
fn foo<'a>(x: &'a str, y: &'a str) -> &'a str
```

Says: "The caller must ensure x and y live at least as long as 'a, and I guarantee the return value lives for 'a."

### Mental Model 3: The Borrow Checker is Conservative

The borrow checker must prove safety **at compile time**. Sometimes it rejects valid programs because it can't prove they're safe.

```rust
let mut vec = vec![1, 2, 3];
let first = &vec[0];

if some_condition() {
    vec.push(4);  // ❌ Error: vec is borrowed
}

println!("{}", first);
```

Even if `some_condition()` is always false, the compiler must consider all paths.

**Workaround**: Limit borrow scope:
```rust
let mut vec = vec![1, 2, 3];
{
    let first = &vec[0];
    println!("{}", first);
}  // first's borrow ends

if some_condition() {
    vec.push(4);  // ✅ OK
}
```

## Advanced: Lifetime Subtyping

A lifetime `'a` is a subtype of `'b` if `'a` outlives `'b` (written `'a: 'b`).

```rust
fn choose<'a, 'b>(
    x: &'a str,
    y: &'b str,
    condition: bool
) -> &'a str
where
    'b: 'a  // 'b must outlive 'a
{
    if condition { x } else { y }
}
```

This allows returning `y` (with lifetime `'b`) as if it had lifetime `'a`, because `'b` is guaranteed to be at least as long.

**Rarely needed** in application code, but common in advanced library code.

## When to Fight the Borrow Checker

### Don't Fight: Restructure Instead

Often, lifetime errors indicate a design issue.

**Example**: Trying to hold both `&mut` and `&` to the same data.

**Better**: Split data structure or use interior mutability patterns (covered in future exercises).

### Don't Fight: Clone Instead

If the data is small, cloning might be simpler than complex lifetime annotations.

```rust
// Complex lifetimes
fn process<'a>(data: &'a str) -> Result<Processed<'a>, Error> { ... }

// Simpler: own the data
fn process(data: String) -> Result<Processed, Error> { ... }
```

**Tradeoff**: Simplicity vs performance.

### Do Fight: When Building Libraries

Library code often requires precise lifetime control to provide flexible APIs.

## Key Takeaways

1. **Lifetimes** ensure references are always valid (no dangling pointers)
2. **Lifetime elision** means you often don't write them
3. **Annotations** describe relationships, they don't change behavior
4. **'a in `&'a T`** means "the reference is valid for lifetime 'a"
5. **Multiple lifetimes** when different references have different constraints
6. **'static** means valid for the entire program (rare in application code)
7. **Borrow checker errors** often indicate design issues, not just syntax problems

## Practice

Before moving on, ensure you can:

- [ ] Read and understand lifetime annotations in function signatures
- [ ] Explain why `'static` is rarely needed
- [ ] Fix "does not live long enough" errors
- [ ] Fix "expected named lifetime parameter" errors
- [ ] Understand when structs need lifetime parameters

## Next Steps

Read [Lecture 03: Enums and Pattern Matching →](./03-enums-and-patterns.md) to learn how Rust models complex data.

Or if you want hands-on practice with lifetimes, try [Exercise 01: LRU Cache →](../exercises/ex01-lru-cache/), which uses advanced borrowing patterns.
