# Lecture 01: Ownership and Borrowing

**The Core Idea That Makes Rust Different**

## Introduction

If you take away one concept from Rust, it should be **ownership**. Every other unique feature of Rust—the borrow checker, lifetimes, the safety guarantees—stems from ownership.

Coming from TypeScript or Python, you're used to garbage collection handling memory automatically. Rust takes a different approach: it tracks ownership at compile time and automatically frees memory when the owner goes out of scope. No GC overhead, no manual malloc/free, no use-after-free bugs.

## The Problem: Memory Management

Let's review the landscape:

### Manual Memory Management (C/C++)

```c
// C code
char* str = malloc(100);
strcpy(str, "hello");
// ... use str ...
free(str);  // ⚠️  Must remember to free!
// Later:
printf("%s", str);  // 💥 Use-after-free bug!
```

**Problems**:
- Forget to free → memory leak
- Free too early → use-after-free
- Free twice → double-free corruption
- Thread safety → data races

### Garbage Collection (TypeScript, Python, Go, Java)

```typescript
// TypeScript code
let str = "hello";
let str2 = str;  // Copy reference
// ... GC will clean up when no references remain
```

**Advantages**:
- No manual memory management
- No dangling pointers

**Tradeoffs**:
- Runtime overhead (GC pauses)
- Unpredictable latency
- Higher memory usage
- Still allows data races (in multi-threaded contexts)

### Rust's Approach: Ownership

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 is *moved* to s2

// println!("{}", s1);  // ❌ Compile error: value used after move
println!("{}", s2);  // ✅ OK
```

**Advantages**:
- No GC overhead
- No manual memory management
- Memory safety guaranteed at compile time
- Data race freedom guaranteed at compile time

**Tradeoff**:
- More restrictive type system
- Steeper learning curve
- Compiler errors when you violate ownership rules

## Ownership Rules

There are three fundamental rules:

1. **Each value in Rust has an owner**
2. **There can only be one owner at a time**
3. **When the owner goes out of scope, the value is dropped (freed)**

Let's explore each rule.

### Rule 1: Each Value Has an Owner

```rust
let s = String::from("hello");
//  ^               ^
//  owner           value
```

The variable `s` owns the String. When we say "owns," we mean `s` is responsible for cleaning up the String when it's no longer needed.

### Rule 2: One Owner at a Time

```rust
let s1 = String::from("hello");
let s2 = s1;  // Ownership transfers from s1 to s2

// s1 is no longer valid here
// s2 is now the owner
```

This is called a **move**. Ownership has moved from `s1` to `s2`.

**Why not copy?**
For types that manage heap data (like String, Vec, HashMap), copying would mean duplicating the heap allocation. Rust makes this explicit:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();  // Explicit deep copy

// Both s1 and s2 are valid
println!("{} {}", s1, s2);  // ✅ OK
```

**Stack-only types DO copy** (implementing the `Copy` trait):

```rust
let x = 5;
let y = x;  // x is copied (integers are Copy)

println!("{} {}", x, y);  // ✅ Both valid
```

Types that implement `Copy`: integers, floats, booleans, chars, tuples of Copy types.

### Rule 3: Drop When Out of Scope

```rust
{
    let s = String::from("hello");
    // s is valid here
}  // s goes out of scope, String is dropped (freed)

// s is no longer valid here
```

Rust automatically calls the `drop` function (like a destructor) when a value goes out of scope. No manual cleanup needed.

## Ownership and Functions

### Passing to Functions

```rust
fn take_ownership(s: String) {
    println!("{}", s);
}  // s is dropped here

fn main() {
    let s = String::from("hello");
    take_ownership(s);  // s is moved into function

    // println!("{}", s);  // ❌ Error: value used after move
}
```

The function parameter takes ownership. After the call, the original variable is no longer valid.

### Returning from Functions

```rust
fn give_ownership() -> String {
    let s = String::from("hello");
    s  // Ownership is transferred to caller
}

fn main() {
    let s = give_ownership();  // s now owns the String
    println!("{}", s);  // ✅ OK
}
```

Returning a value transfers ownership to the caller.

### The Problem: Taking and Returning is Tedious

```rust
fn calculate_length(s: String) -> (String, usize) {
    let length = s.len();
    (s, length)  // Return ownership back!
}

let s1 = String::from("hello");
let (s2, len) = calculate_length(s1);
```

This is annoying. We just want to read the string's length, not take ownership!

**Solution: Borrowing**

## Borrowing: References Without Ownership

Borrowing lets you refer to a value without taking ownership.

### Immutable Borrows

```rust
fn calculate_length(s: &String) -> usize {
    s.len()
}  // s goes out of scope, but doesn't drop the String (we don't own it)

let s1 = String::from("hello");
let len = calculate_length(&s1);  // Borrow s1

println!("{} {}", s1, len);  // ✅ s1 still valid
```

The `&` creates a reference. The function borrows the String but doesn't own it.

**Mental model**: Like a read-only pointer that the compiler guarantees won't outlive the data.

### Mutable Borrows

```rust
fn append_world(s: &mut String) {
    s.push_str(" world");
}

let mut s = String::from("hello");
append_world(&mut s);

println!("{}", s);  // "hello world"
```

The `&mut` creates a mutable reference, allowing modification.

## Borrowing Rules

These are **the most important rules in Rust**:

1. **At any given time, you can have EITHER:**
   - One mutable reference (`&mut T`)
   - OR any number of immutable references (`&T`)
   - **But not both**

2. **References must always be valid** (no dangling references)

### Why These Rules?

They prevent **data races** at compile time.

A data race occurs when:
- Two or more pointers access the same data
- At least one is writing
- No synchronization mechanism

Rust's borrowing rules make data races impossible:

```rust
let mut s = String::from("hello");

let r1 = &s;     // ✅ Immutable borrow
let r2 = &s;     // ✅ Immutable borrow
let r3 = &mut s; // ❌ Error: cannot borrow as mutable while immutably borrowed

println!("{} {}", r1, r2);
```

This is a **compile-time error**. In other languages, this could cause a data race at runtime.

### The Lifetime of a Borrow

```rust
let mut s = String::from("hello");

let r1 = &s;
let r2 = &s;
println!("{} {}", r1, r2);
// r1 and r2 are no longer used after this point

let r3 = &mut s;  // ✅ OK: no immutable borrows are active
println!("{}", r3);
```

**Non-Lexical Lifetimes (NLL)**: The borrow checker is smart enough to see that `r1` and `r2` aren't used after the println, so `r3` is allowed.

## Comparing to Other Languages

### TypeScript/JavaScript

```typescript
let obj = { value: 5 };
let ref1 = obj;
let ref2 = obj;
ref1.value = 10;

console.log(ref2.value);  // 10 - modified through ref1
// No compile-time guarantee about who can modify when
```

Multiple mutable references are allowed. The runtime doesn't prevent data races (though JS is single-threaded, this matters in async code or with SharedArrayBuffer).

### Python

```python
lst = [1, 2, 3]
ref1 = lst
ref2 = lst
ref1.append(4)

print(ref2)  # [1, 2, 3, 4]
# Again, no compile-time tracking
```

Same as TypeScript—references are freely copyable and mutable.

### C++

```cpp
std::string s = "hello";
std::string& ref1 = s;
std::string& ref2 = s;
ref1 += " world";
// Both ref1 and ref2 now refer to "hello world"
// But if s goes out of scope, ref1 and ref2 dangle!
```

C++ has references, but no compile-time enforcement that they remain valid.

**Rust's advantage**: The compiler prevents both data races AND dangling references.

## Common Patterns

### Pattern 1: Read-Only Access

```rust
fn print_string(s: &String) {
    println!("{}", s);
}

let s = String::from("hello");
print_string(&s);
print_string(&s);  // Can call multiple times
```

Use `&T` when you only need to read.

### Pattern 2: Modification

```rust
fn clear_string(s: &mut String) {
    s.clear();
}

let mut s = String::from("hello");
clear_string(&mut s);
assert_eq!(s, "");
```

Use `&mut T` when you need to modify.

### Pattern 3: Taking Ownership for Consumption

```rust
fn consume_string(s: String) {
    // Do something with s
    println!("{}", s);
}  // s is dropped

let s = String::from("hello");
consume_string(s);
// s no longer valid
```

Use `T` (not `&T`) when the function should consume the value.

**When to take ownership**:
- When the function needs to store the value (e.g., pushing into a Vec)
- When the function will transform and return it
- When the function is the final consumer

## String vs &str

This is a common confusion point.

### String
- Owned, heap-allocated, growable
- Like `std::string` in C++ or `string` in TypeScript

```rust
let s = String::from("hello");
let s2 = s.clone();  // Deep copy
```

### &str
- Borrowed string slice (reference to UTF-8 data)
- Immutable view into string data
- Like `std::string_view` in C++20

```rust
let s: &str = "hello";  // String literal
let s2: &str = &String::from("world");  // Borrow of String
```

**Best practice**: Use `&str` for function parameters (more flexible):

```rust
// ❌ Less flexible
fn print_string(s: &String) {
    println!("{}", s);
}

// ✅ More flexible
fn print_str(s: &str) {
    println!("{}", s);
}

let owned = String::from("hello");
let slice = "world";

print_str(&owned);  // ✅ Works
print_str(slice);   // ✅ Works

print_string(&owned);  // ✅ Works
// print_string(slice);  // ❌ Error: expected &String, found &str
```

`&str` accepts both String (via deref coercion) and string slices.

## Mental Models

### Mental Model 1: Move = Transfer of Responsibility

```rust
let s1 = String::from("hello");
let s2 = s1;  // s2 is now responsible for cleanup
```

Think of ownership as a "cleanup responsibility." Only one variable can be responsible.

### Mental Model 2: Borrowing = Read-Write Locks

```rust
let mut data = vec![1, 2, 3];

let r1 = &data;     // Read lock
let r2 = &data;     // Another read lock
// let r3 = &mut data;  // ❌ Can't acquire write lock while read locks exist
```

- Immutable borrows = multiple readers
- Mutable borrow = single writer
- Can't mix readers and writers

Checked at compile time, not runtime.

### Mental Model 3: Lifetimes = Scope of Validity

```rust
let r;
{
    let x = 5;
    r = &x;  // ❌ Error: x doesn't live long enough
}
// r would be a dangling reference here
```

The compiler ensures references don't outlive the data they point to.

## Common Errors and Fixes

### Error: "Value used after move"

```rust
let s = String::from("hello");
let s2 = s;
println!("{}", s);  // ❌ Error
```

**Fix 1**: Clone if you need two copies:
```rust
let s = String::from("hello");
let s2 = s.clone();
println!("{} {}", s, s2);  // ✅ OK
```

**Fix 2**: Borrow instead:
```rust
let s = String::from("hello");
let s2 = &s;
println!("{} {}", s, s2);  // ✅ OK
```

### Error: "Cannot borrow as mutable more than once"

```rust
let mut s = String::from("hello");
let r1 = &mut s;
let r2 = &mut s;  // ❌ Error
println!("{} {}", r1, r2);
```

**Fix**: Limit mutable borrow scope:
```rust
let mut s = String::from("hello");
{
    let r1 = &mut s;
    r1.push_str(" world");
}  // r1 goes out of scope

let r2 = &mut s;  // ✅ OK now
r2.push_str("!");
```

### Error: "Cannot borrow as mutable while immutably borrowed"

```rust
let mut s = String::from("hello");
let r1 = &s;
let r2 = &mut s;  // ❌ Error
println!("{} {}", r1, r2);
```

**Fix**: Ensure immutable borrows aren't used after mutable borrow:
```rust
let mut s = String::from("hello");
let r1 = &s;
println!("{}", r1);  // Last use of r1

let r2 = &mut s;  // ✅ OK: r1 is no longer active
r2.push_str(" world");
```

## Key Takeaways

1. **Ownership** means exactly one variable is responsible for cleaning up a value
2. **Moving** transfers ownership (no copy for heap types by default)
3. **Borrowing** (`&` and `&mut`) allows access without ownership
4. **Borrow rules** prevent data races at compile time:
   - Many `&T` OR one `&mut T`, never both
5. **References never outlive data** (enforced by lifetimes, discussed next lecture)
6. Use `&str` for string parameters, not `&String`
7. The compiler is strict to guarantee memory safety and thread safety

## Practice

Before moving to the next lecture, ensure you can:

- [ ] Explain why `let s2 = s1;` moves ownership
- [ ] Describe the difference between `&T` and `&mut T`
- [ ] Explain why you can't have `&mut` and `&` to the same data
- [ ] Fix "value used after move" errors
- [ ] Fix "cannot borrow as mutable" errors

## Next Steps

Read [Lecture 02: Lifetimes Demystified →](./02-lifetimes.md) to understand how Rust tracks the scope of borrows.

Or if you want to practice first, try [Exercise 01: LRU Cache →](../exercises/ex01-lru-cache/) which relies heavily on ownership patterns.
