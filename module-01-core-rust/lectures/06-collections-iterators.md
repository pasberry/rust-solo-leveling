# Lecture 06: Collections and Iterators

**Efficient Data Structures and Functional-Style Iteration**

## Introduction

Rust's standard library provides powerful collection types and a rich iterator API. If you're coming from TypeScript/JavaScript, you'll find iterators similar to array methods (`.map()`, `.filter()`, etc.) but **zero-cost**—compiled down to simple loops with no overhead.

From Python, you'll recognize the iterator pattern, but with Rust's compile-time guarantees and performance characteristics.

## Core Collections

### Vec<T>: Growable Array

The most common collection. Like `Array` in TypeScript or `list` in Python.

```rust
// Creation
let mut v: Vec<i32> = Vec::new();
let mut v = vec![1, 2, 3];  // vec! macro

// Adding elements
v.push(4);
v.push(5);

// Accessing elements
let third = &v[2];  // Panics if out of bounds
let third = v.get(2);  // Returns Option<&i32>

match v.get(2) {
    Some(value) => println!("Third: {}", value),
    None => println!("No third element"),
}

// Iteration
for item in &v {
    println!("{}", item);
}

// Mutable iteration
for item in &mut v {
    *item += 1;
}

// Taking ownership
for item in v {
    println!("{}", item);
}  // v is moved, no longer accessible
```

**Key operations**:
- `push(item)` - Add to end (amortized O(1))
- `pop()` - Remove from end (O(1))
- `insert(index, item)` - Insert at index (O(n))
- `remove(index)` - Remove at index (O(n))
- `len()` - Number of elements (O(1))
- `is_empty()` - Check if empty (O(1))

### HashMap<K, V>: Key-Value Store

Like `Map` in TypeScript or `dict` in Python.

```rust
use std::collections::HashMap;

// Creation
let mut scores = HashMap::new();
scores.insert(String::from("Blue"), 10);
scores.insert(String::from("Yellow"), 50);

// From iterator
let teams = vec!["Blue", "Yellow"];
let initial_scores = vec![10, 50];
let scores: HashMap<_, _> = teams.iter()
    .zip(initial_scores.iter())
    .collect();

// Accessing
let team_name = String::from("Blue");
let score = scores.get(&team_name);  // Returns Option<&V>

match score {
    Some(&s) => println!("Score: {}", s),
    None => println!("Team not found"),
}

// Updating
scores.insert(String::from("Blue"), 25);  // Overwrites

// Insert only if key doesn't exist
scores.entry(String::from("Blue")).or_insert(50);

// Update based on old value
let text = "hello world wonderful world";
let mut map = HashMap::new();

for word in text.split_whitespace() {
    let count = map.entry(word).or_insert(0);
    *count += 1;
}
```

**Key operations**:
- `insert(key, value)` - Insert or update (O(1) average)
- `get(&key)` - Get reference to value (O(1) average)
- `remove(&key)` - Remove entry (O(1) average)
- `contains_key(&key)` - Check existence (O(1) average)
- `entry(key)` - Entry API for complex updates

**Requirements**:
- `K` must implement `Eq` and `Hash`
- For best performance, use `ahash` or `fnv` crates for custom hashers

### HashSet<T>: Unique Values

Like `Set` in TypeScript or `set` in Python.

```rust
use std::collections::HashSet;

let mut set = HashSet::new();
set.insert(1);
set.insert(2);
set.insert(2);  // Duplicate, ignored

println!("{}", set.len());  // 2

// Set operations
let a: HashSet<_> = [1, 2, 3].iter().cloned().collect();
let b: HashSet<_> = [3, 4, 5].iter().cloned().collect();

let union: HashSet<_> = a.union(&b).cloned().collect();  // {1, 2, 3, 4, 5}
let intersection: HashSet<_> = a.intersection(&b).cloned().collect();  // {3}
let difference: HashSet<_> = a.difference(&b).cloned().collect();  // {1, 2}
```

### BTreeMap and BTreeSet: Sorted Collections

When you need ordered iteration or range queries:

```rust
use std::collections::BTreeMap;

let mut scores = BTreeMap::new();
scores.insert("Alice", 100);
scores.insert("Bob", 85);
scores.insert("Carol", 95);

// Ordered iteration
for (name, score) in &scores {
    println!("{}: {}", name, score);  // Alphabetical order
}

// Range queries
for (name, score) in scores.range("Bob".."Carol") {
    println!("{}: {}", name, score);
}
```

**When to use**:
- `HashMap/HashSet`: Default choice, faster for most operations
- `BTreeMap/BTreeSet`: Need sorted order or range queries

### VecDeque<T>: Double-Ended Queue

Efficient push/pop from both ends:

```rust
use std::collections::VecDeque;

let mut deque = VecDeque::new();
deque.push_back(1);   // Add to back
deque.push_front(2);  // Add to front
deque.pop_back();     // Remove from back
deque.pop_front();    // Remove from front
```

**When to use**: Need efficient operations at both ends (queue, ring buffer)

## Iterators: The Heart of Rust Collections

Iterators are lazy, composable, and **zero-cost**.

### Creating Iterators

```rust
let v = vec![1, 2, 3];

// Immutable iteration
for item in &v {
    println!("{}", item);  // item is &i32
}

// Mutable iteration
for item in &mut v {
    *item += 1;  // item is &mut i32
}

// Consuming iteration (takes ownership)
for item in v {
    println!("{}", item);  // item is i32
}  // v is no longer accessible
```

Explicitly:

```rust
let v = vec![1, 2, 3];
let mut iter = v.iter();  // Iterator<Item = &i32>

while let Some(item) = iter.next() {
    println!("{}", item);
}
```

**Three iteration methods**:
- `.iter()` - Borrows immutably (`&T`)
- `.iter_mut()` - Borrows mutably (`&mut T`)
- `.into_iter()` - Takes ownership (`T`)

### Iterator Adapters (Transformations)

Adapters are **lazy**—they don't do anything until consumed.

#### map: Transform Each Element

```rust
let v = vec![1, 2, 3];
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();
// [2, 4, 6]
```

#### filter: Select Elements

```rust
let v = vec![1, 2, 3, 4, 5];
let evens: Vec<_> = v.iter().filter(|&x| x % 2 == 0).collect();
// [2, 4]
```

#### filter_map: Filter and Map in One

```rust
let strings = vec!["42", "not a number", "7"];
let numbers: Vec<_> = strings
    .iter()
    .filter_map(|s| s.parse::<i32>().ok())
    .collect();
// [42, 7]
```

#### take, skip: Limit Iteration

```rust
let v = vec![1, 2, 3, 4, 5];
let first_three: Vec<_> = v.iter().take(3).collect();  // [1, 2, 3]
let after_two: Vec<_> = v.iter().skip(2).collect();    // [3, 4, 5]
```

#### enumerate: Add Index

```rust
let v = vec!["a", "b", "c"];
for (i, value) in v.iter().enumerate() {
    println!("{}: {}", i, value);
}
```

#### zip: Combine Two Iterators

```rust
let names = vec!["Alice", "Bob"];
let scores = vec![100, 85];

let pairs: Vec<_> = names.iter().zip(scores.iter()).collect();
// [("Alice", 100), ("Bob", 85)]
```

#### chain: Concatenate Iterators

```rust
let v1 = vec![1, 2];
let v2 = vec![3, 4];

let combined: Vec<_> = v1.iter().chain(v2.iter()).collect();
// [1, 2, 3, 4]
```

#### flat_map: Flatten Nested Structures

```rust
let v = vec![vec![1, 2], vec![3, 4]];
let flattened: Vec<_> = v.iter().flat_map(|inner| inner.iter()).collect();
// [1, 2, 3, 4]
```

### Iterator Consumers (Terminate Iteration)

Consumers trigger the lazy iterator chain.

#### collect: Build a Collection

```rust
let v = vec![1, 2, 3];
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();

// Can collect into different types
let set: HashSet<_> = v.into_iter().collect();
let string: String = "hello".chars().collect();
```

#### fold: Reduce to a Single Value

```rust
let v = vec![1, 2, 3, 4];
let sum = v.iter().fold(0, |acc, x| acc + x);  // 10

// Equivalent to:
let sum: i32 = v.iter().sum();
```

#### reduce: Like fold, but returns Option

```rust
let v = vec![1, 2, 3, 4];
let product = v.iter().reduce(|acc, x| acc * x);  // Some(24)

let empty: Vec<i32> = vec![];
let product = empty.iter().reduce(|acc, x| acc * x);  // None
```

#### any, all: Boolean Tests

```rust
let v = vec![1, 2, 3, 4];
let has_even = v.iter().any(|&x| x % 2 == 0);  // true
let all_positive = v.iter().all(|&x| x > 0);   // true
```

#### find: First Matching Element

```rust
let v = vec![1, 2, 3, 4];
let first_even = v.iter().find(|&&x| x % 2 == 0);  // Some(&2)
```

#### position: Index of First Match

```rust
let v = vec![1, 2, 3, 4];
let pos = v.iter().position(|&x| x == 3);  // Some(2)
```

#### count: Number of Elements

```rust
let v = vec![1, 2, 3, 4];
let count = v.iter().filter(|&&x| x % 2 == 0).count();  // 2
```

### Chaining Operations

Iterators compose beautifully:

```rust
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

let result: Vec<_> = numbers
    .iter()
    .filter(|&&x| x % 2 == 0)  // Even numbers
    .map(|x| x * x)             // Square them
    .take(3)                    // First 3
    .collect();

// [4, 16, 36]
```

**Zero overhead**: Compiles to the same efficient code as a hand-written loop.

## Comparing to TypeScript/Python

### TypeScript Array Methods

```typescript
const numbers = [1, 2, 3, 4, 5];

const result = numbers
    .filter(x => x % 2 === 0)
    .map(x => x * x)
    .slice(0, 3);

// [4, 16]
```

**Key differences**:
- TypeScript: Each method creates an intermediate array (memory allocations)
- Rust: Iterators are lazy, compiled to a single loop (zero allocations)

### Python List Comprehensions / Generators

```python
numbers = [1, 2, 3, 4, 5]

# List comprehension (eager)
result = [x * x for x in numbers if x % 2 == 0][:3]

# Generator (lazy)
result = list(itertools.islice(
    (x * x for x in numbers if x % 2 == 0),
    3
))
```

**Key differences**:
- Python generators are lazy but have runtime overhead
- Rust iterators have zero runtime overhead (compiled away)

## Custom Iterators

You can implement the `Iterator` trait for custom types:

```rust
struct Counter {
    count: u32,
    max: u32,
}

impl Counter {
    fn new(max: u32) -> Self {
        Counter { count: 0, max }
    }
}

impl Iterator for Counter {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count <= self.max {
            Some(self.count)
        } else {
            None
        }
    }
}

// Usage
let counter = Counter::new(5);
for num in counter {
    println!("{}", num);  // 1, 2, 3, 4, 5
}
```

Once you implement `Iterator`, you get all the adapter methods for free!

```rust
let counter = Counter::new(10);
let result: Vec<_> = counter
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .collect();
// [4, 16, 36, 64, 100]
```

## Performance Considerations

### Avoid Unnecessary Collections

```rust
// ❌ Creates intermediate Vec
let v = vec![1, 2, 3, 4, 5];
let intermediate: Vec<_> = v.iter().map(|x| x * 2).collect();
let result: Vec<_> = intermediate.iter().filter(|&&x| x > 5).collect();

// ✅ Single pass, no intermediate allocations
let result: Vec<_> = v.iter()
    .map(|x| x * 2)
    .filter(|&x| x > 5)
    .collect();
```

### collect() is Expensive

```rust
// ❌ Collects unnecessarily
let v = vec![1, 2, 3];
let sum: i32 = v.iter().map(|x| x * 2).collect::<Vec<_>>().iter().sum();

// ✅ Direct sum
let sum: i32 = v.iter().map(|x| x * 2).sum();
```

### Use &[T] for Function Parameters

```rust
// ❌ Only accepts Vec
fn process(v: &Vec<i32>) -> i32 {
    v.iter().sum()
}

// ✅ Accepts Vec, arrays, slices
fn process(slice: &[i32]) -> i32 {
    slice.iter().sum()
}

// Even better: accept any iterator
fn process<I>(iter: I) -> i32
where
    I: Iterator<Item = i32>,
{
    iter.sum()
}
```

### Iteration is Zero-Cost

This:

```rust
let sum: i32 = (1..100)
    .filter(|x| x % 2 == 0)
    .map(|x| x * x)
    .sum();
```

Compiles to the same assembly as:

```rust
let mut sum = 0;
for x in 1..100 {
    if x % 2 == 0 {
        sum += x * x;
    }
}
```

## Common Patterns

### Pattern 1: Processing Text

```rust
let text = "hello world";

// Word frequency
let mut freq = HashMap::new();
for word in text.split_whitespace() {
    *freq.entry(word).or_insert(0) += 1;
}

// Unique words
let unique: HashSet<_> = text.split_whitespace().collect();
```

### Pattern 2: Grouping

```rust
use std::collections::HashMap;

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
}

let people = vec![
    Person { name: "Alice".into(), age: 30 },
    Person { name: "Bob".into(), age: 25 },
    Person { name: "Carol".into(), age: 30 },
];

// Group by age
let mut by_age: HashMap<u32, Vec<&Person>> = HashMap::new();
for person in &people {
    by_age.entry(person.age).or_insert_with(Vec::new).push(person);
}
```

### Pattern 3: Finding Min/Max

```rust
let numbers = vec![3, 1, 4, 1, 5, 9];

let max = numbers.iter().max();  // Some(&9)
let min = numbers.iter().min();  // Some(&1)

// Custom comparison
#[derive(Debug)]
struct Person { name: String, age: u32 }

let people = vec![
    Person { name: "Alice".into(), age: 30 },
    Person { name: "Bob".into(), age: 25 },
];

let oldest = people.iter().max_by_key(|p| p.age);  // Alice
```

### Pattern 4: Partitioning

```rust
let numbers = vec![1, 2, 3, 4, 5, 6];
let (evens, odds): (Vec<_>, Vec<_>) = numbers
    .into_iter()
    .partition(|&x| x % 2 == 0);
// evens: [2, 4, 6], odds: [1, 3, 5]
```

### Pattern 5: Sorting

```rust
let mut numbers = vec![3, 1, 4, 1, 5];
numbers.sort();  // [1, 1, 3, 4, 5]

let mut numbers = vec![3, 1, 4, 1, 5];
numbers.sort_unstable();  // Faster, may reorder equal elements

// Custom sorting
#[derive(Debug, Eq, PartialEq)]
struct Person { name: String, age: u32 }

let mut people = vec![
    Person { name: "Alice".into(), age: 30 },
    Person { name: "Bob".into(), age: 25 },
];

people.sort_by_key(|p| p.age);
people.sort_by(|a, b| a.name.cmp(&b.name));
```

## Key Takeaways

1. **Vec<T>** is the default collection (growable array)
2. **HashMap<K, V>** for key-value storage
3. **HashSet<T>** for unique values
4. **Iterators are lazy** and **zero-cost**
5. **Adapters** transform (`map`, `filter`), **consumers** terminate (`collect`, `fold`)
6. **Chaining** iterator methods compiles to efficient single-pass code
7. Use **slices** (`&[T]`) for function parameters, not `&Vec<T>`
8. **Collect sparingly**—many operations don't need intermediate collections

## Practice

Before moving on, ensure you can:

- [ ] Create and manipulate `Vec`, `HashMap`, `HashSet`
- [ ] Use common iterator adapters (`map`, `filter`, `fold`)
- [ ] Chain iterator operations efficiently
- [ ] Understand when iterators are lazy vs eager
- [ ] Choose appropriate collection types for different scenarios
- [ ] Implement the `Iterator` trait for custom types
- [ ] Avoid unnecessary collections in iterator chains

## Next Steps

Read [Lecture 07: Testing and Development Workflow →](./07-testing.md) to learn how to test your Rust code effectively.

Or start [Exercise 03: Text Search Tool →](../exercises/ex03-text-search/) to practice collections and iterators in a real project.
