# Lecture 03: Enums and Pattern Matching

**Algebraic Data Types: Type-Driven Design**

## Introduction

Rust's enums are far more powerful than enums in TypeScript or Python. They're **algebraic data types** (ADTs) that let you model complex domains with precision.

Combined with **pattern matching**, enums eliminate entire classes of bugs by making invalid states unrepresentable and forcing you to handle all cases.

If you've used TypeScript discriminated unions, Rust enums will feel familiar but more powerful. If you're coming from Python, this will be a new way of thinking.

## Enums vs Other Languages

### Python Enums (Simple)

```python
from enum import Enum

class Status(Enum):
    PENDING = 1
    RUNNING = 2
    COMPLETED = 3

status = Status.PENDING
```

Python enums are just named constants. Each variant holds the same type (or no data).

### TypeScript Discriminated Unions (Powerful)

```typescript
type Message =
    | { type: 'Quit' }
    | { type: 'Move'; x: number; y: number }
    | { type: 'Write'; text: string }

function handle(msg: Message) {
    switch (msg.type) {
        case 'Quit':
            exit();
            break;
        case 'Move':
            moveTo(msg.x, msg.y);
            break;
        case 'Write':
            write(msg.text);
            break;
    }
}
```

TypeScript discriminated unions are close to Rust enums, but:
- Not exhaustiveness-checked by default (need strict mode + careful config)
- More verbose (need `type` discriminator field)
- Runtime overhead (extra field in memory)

### Rust Enums (Powerful + Zero-Cost)

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

fn handle(msg: Message) {
    match msg {
        Message::Quit => exit(),
        Message::Move { x, y } => move_to(x, y),
        Message::Write(text) => write(text),
        Message::ChangeColor(r, g, b) => set_color(r, g, b),
    }  // Compiler error if you miss a case!
}
```

**Advantages**:
- Exhaustiveness checking (compiler error if you forget a case)
- Zero runtime overhead (size of largest variant + tag)
- Type-safe access to variant data

## Defining Enums

### Basic Syntax

```rust
enum IpAddr {
    V4,
    V6,
}

let home = IpAddr::V4;
let loopback = IpAddr::V6;
```

### Enums with Data

Each variant can hold different types and amounts of data:

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

let home = IpAddr::V4(127, 0, 0, 1);
let loopback = IpAddr::V6(String::from("::1"));
```

### Enums with Named Fields

```rust
enum Message {
    Quit,                           // No data
    Move { x: i32, y: i32 },       // Named fields (like struct)
    Write(String),                  // Tuple-like (one field)
    ChangeColor(u8, u8, u8),       // Tuple-like (multiple fields)
}
```

Each variant can be thought of as a different type, all under the same enum umbrella.

### Enums Can Hold Any Type

```rust
enum WebEvent {
    PageLoad,
    PageUnload,
    KeyPress(char),
    Paste(String),
    Click { x: i64, y: i64 },
}

// Even other enums or structs!
enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
    Triangle(Triangle),
}

struct Circle { radius: f64 }
struct Rectangle { width: f64, height: f64 }
struct Triangle { base: f64, height: f64 }
```

## Pattern Matching

Pattern matching is how you **extract data** from enums and **branch** based on the variant.

### The `match` Expression

```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
    }
}
```

**Key properties**:
- Must be **exhaustive** (cover all variants)
- Each arm is `pattern => expression`
- Returns a value (like ternary or switch expressions)

### Binding Values in Patterns

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

fn describe(ip: IpAddr) -> String {
    match ip {
        IpAddr::V4(a, b, c, d) => format!("{}.{}.{}.{}", a, b, c, d),
        IpAddr::V6(addr) => addr,
    }
}

let home = IpAddr::V4(127, 0, 0, 1);
println!("{}", describe(home));  // "127.0.0.1"
```

The pattern `IpAddr::V4(a, b, c, d)` **destructures** the variant and binds the values to variables.

### Matching Named Fields

```rust
enum Message {
    Move { x: i32, y: i32 },
    Write(String),
}

match msg {
    Message::Move { x, y } => println!("Move to ({}, {})", x, y),
    Message::Write(text) => println!("Write: {}", text),
}
```

### Match Guards

Add extra conditions with `if`:

```rust
match number {
    n if n < 0 => println!("Negative"),
    n if n > 0 => println!("Positive"),
    _ => println!("Zero"),
}
```

### The `_` Wildcard

```rust
match dice_roll {
    1 => println!("One"),
    2 => println!("Two"),
    _ => println!("Other"),  // Matches everything else
}
```

`_` is used when you don't care about the other cases.

### Ignoring Values with `..`

```rust
struct Point { x: i32, y: i32, z: i32 }

match point {
    Point { x, .. } => println!("x is {}", x),  // Ignore y and z
}
```

## Option and Result: The Killer Apps

Rust's most important enums are `Option` and `Result`, defined in the standard library.

### Option<T>: Representing Optional Values

```rust
enum Option<T> {
    Some(T),
    None,
}
```

**Purpose**: Replace `null`/`undefined`/`None` with a type-safe alternative.

```rust
let some_number = Some(5);
let some_string = Some("a string");
let absent_number: Option<i32> = None;
```

**Why it's better than null**:

```typescript
// TypeScript
function getUser(id: number): User | null {
    // ...
}

let user = getUser(1);
user.name;  // ⚠️  Runtime error if user is null!
```

```rust
// Rust
fn get_user(id: u32) -> Option<User> {
    // ...
}

let user = get_user(1);
// user.name;  // ❌ Compile error: Option<User> is not User

// Must handle None case
match user {
    Some(u) => println!("{}", u.name),
    None => println!("User not found"),
}
```

The type system **forces** you to check for None before using the value.

### Working with Option

```rust
// Unwrapping (panics if None - use sparingly!)
let x = Some(5);
assert_eq!(x.unwrap(), 5);

// Safe unwrapping with default
let x: Option<i32> = None;
assert_eq!(x.unwrap_or(0), 0);

// Mapping over Option
let x = Some(5);
let y = x.map(|n| n * 2);  // Some(10)

let x: Option<i32> = None;
let y = x.map(|n| n * 2);  // None

// Chaining operations
let result = get_user(id)
    .map(|u| u.email)
    .unwrap_or("no-email@example.com");
```

### Result<T, E>: Error Handling

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}
```

**Purpose**: Represent operations that can fail, with error information.

```rust
use std::fs::File;
use std::io::Error;

fn open_file(path: &str) -> Result<File, Error> {
    File::open(path)
}

match open_file("config.toml") {
    Ok(file) => println!("File opened successfully"),
    Err(e) => println!("Failed to open file: {}", e),
}
```

More on Result in the next lecture on error handling.

## if let: Pattern Matching for One Case

When you only care about **one variant**, `if let` is more concise than `match`:

```rust
let some_value = Some(5);

// With match (verbose)
match some_value {
    Some(x) => println!("Value: {}", x),
    None => {}  // Do nothing
}

// With if let (concise)
if let Some(x) = some_value {
    println!("Value: {}", x);
}
```

**With else**:

```rust
if let Some(user) = get_user(id) {
    println!("Found: {}", user.name);
} else {
    println!("User not found");
}
```

### while let: Loop Until Pattern Fails

```rust
let mut stack = vec![1, 2, 3];

while let Some(top) = stack.pop() {
    println!("{}", top);
}  // Prints: 3, 2, 1
```

## Practical Patterns

### Pattern 1: State Machines

```rust
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected { session_id: String },
    Error { code: u32, message: String },
}

impl ConnectionState {
    fn handle_event(&mut self, event: Event) {
        *self = match (&self, event) {
            (ConnectionState::Disconnected, Event::Connect) => {
                ConnectionState::Connecting
            }
            (ConnectionState::Connecting, Event::Success(session_id)) => {
                ConnectionState::Connected { session_id }
            }
            (ConnectionState::Connecting, Event::Failure(code, msg)) => {
                ConnectionState::Error { code, message: msg }
            }
            _ => return,  // Ignore invalid transitions
        };
    }
}
```

Enums make **invalid states unrepresentable**. You can't have a session_id when Disconnected.

### Pattern 2: Command Pattern

```rust
enum Command {
    AddUser { name: String, email: String },
    RemoveUser { id: u64 },
    UpdateUser { id: u64, email: String },
}

fn execute(cmd: Command, db: &mut Database) {
    match cmd {
        Command::AddUser { name, email } => {
            db.insert_user(name, email);
        }
        Command::RemoveUser { id } => {
            db.delete_user(id);
        }
        Command::UpdateUser { id, email } => {
            db.update_user_email(id, email);
        }
    }
}
```

### Pattern 3: Recursive Data Structures

```rust
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
}

let data = Json::Object(HashMap::from([
    ("name".to_string(), Json::String("Alice".to_string())),
    ("age".to_string(), Json::Number(30.0)),
    ("active".to_string(), Json::Bool(true)),
]));
```

Enums can be **recursive** (variants containing the same enum type).

### Pattern 4: Polymorphism Without Inheritance

```rust
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}

impl Shape {
    fn area(&self) -> f64 {
        match self {
            Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
            Shape::Rectangle { width, height } => width * height,
            Shape::Triangle { base, height } => 0.5 * base * height,
        }
    }
}

let shapes = vec![
    Shape::Circle { radius: 5.0 },
    Shape::Rectangle { width: 10.0, height: 5.0 },
];

for shape in shapes {
    println!("Area: {}", shape.area());
}
```

No inheritance hierarchy, no virtual method dispatch overhead. Just data + exhaustive matching.

## Advanced Pattern Matching

### Destructuring Nested Data

```rust
enum Message {
    Move { x: i32, y: i32 },
    Write(String),
}

enum Event {
    Message(Message),
    Quit,
}

match event {
    Event::Message(Message::Move { x, y }) => {
        println!("Move to ({}, {})", x, y);
    }
    Event::Message(Message::Write(text)) => {
        println!("Write: {}", text);
    }
    Event::Quit => {
        println!("Quit");
    }
}
```

### Using `@` to Bind and Match

```rust
match age {
    n @ 0..=12 => println!("Child of age {}", n),
    n @ 13..=19 => println!("Teen of age {}", n),
    n => println!("Adult of age {}", n),
}
```

### Matching References

```rust
let x = &Some(5);

match x {
    Some(val) => println!("{}", val),  // val is &i32
    None => println!("None"),
}

// Or dereference in pattern
match *x {
    Some(val) => println!("{}", val),  // val is i32
    None => println!("None"),
}
```

## Memory Layout

Rust enums are **tagged unions**, stored efficiently.

```rust
enum Message {
    Quit,                        // 0 bytes of data
    Move { x: i32, y: i32 },    // 8 bytes
    Write(String),               // 24 bytes (String is 3 pointers)
}
```

Memory layout:
- **Tag** (discriminant): 1-8 bytes (depends on number of variants)
- **Data**: Size of the largest variant
- **Total**: Tag + largest variant, with alignment

For `Message`: ~32 bytes (8 for tag/alignment + 24 for String)

**Zero-cost abstraction**: No vtable, no dynamic dispatch, just a tag + data.

## Comparing to TypeScript

### TypeScript Discriminated Union

```typescript
type Result<T, E> =
    | { ok: true; value: T }
    | { ok: false; error: E }

function divide(a: number, b: number): Result<number, string> {
    if (b === 0) {
        return { ok: false, error: "Division by zero" };
    }
    return { ok: true, value: a / b };
}

const result = divide(10, 2);
if (result.ok) {
    console.log(result.value);
} else {
    console.log(result.error);
}
```

**Differences**:
- TypeScript: runtime `ok` field (memory overhead)
- Rust: compile-time tag (no runtime overhead)
- TypeScript: exhaustiveness depends on compiler flags
- Rust: always exhaustive (compile error if incomplete)

### Rust Enum

```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

match divide(10, 2) {
    Ok(value) => println!("{}", value),
    Err(error) => println!("{}", error),
}  // Compiler error if you forget a case
```

## Common Patterns

### Pattern: Builder Pattern with Enums

```rust
enum State {
    Empty,
    Partial { fields: Vec<String> },
    Complete { data: CompleteData },
}

struct Builder {
    state: State,
}

impl Builder {
    fn new() -> Self {
        Builder { state: State::Empty }
    }

    fn add_field(mut self, field: String) -> Self {
        self.state = match self.state {
            State::Empty => State::Partial { fields: vec![field] },
            State::Partial { mut fields } => {
                fields.push(field);
                State::Partial { fields }
            }
            State::Complete { .. } => panic!("Already complete"),
        };
        self
    }

    fn build(self) -> Result<CompleteData, String> {
        match self.state {
            State::Complete { data } => Ok(data),
            _ => Err("Incomplete".to_string()),
        }
    }
}
```

### Pattern: Type-State Pattern

Use enums to encode state transitions in the type system.

```rust
struct Locked;
struct Unlocked;

struct Door<State> {
    state: PhantomData<State>,
}

impl Door<Locked> {
    fn unlock(self, key: Key) -> Result<Door<Unlocked>, Self> {
        if key.is_valid() {
            Ok(Door { state: PhantomData })
        } else {
            Err(self)
        }
    }
}

impl Door<Unlocked> {
    fn open(&mut self) {
        println!("Door opened");
    }
}
```

(This uses zero-sized types, an advanced pattern covered in later modules.)

## Key Takeaways

1. **Enums** are algebraic data types, each variant can hold different data
2. **Pattern matching** is exhaustive (compiler checks all cases)
3. **Option<T>** replaces null/undefined with type safety
4. **Result<T, E>** is the foundation of error handling
5. **if let** and **while let** for matching single variants
6. **Enums enable type-driven design**: make invalid states unrepresentable
7. **Zero-cost abstraction**: no runtime overhead compared to hand-written tags
8. **Use enums for**: state machines, commands, polymorphism without inheritance

## Practice

Before moving on, ensure you can:

- [ ] Define an enum with multiple variants of different shapes
- [ ] Use `match` to handle all enum variants exhaustively
- [ ] Explain why `Option` is better than null
- [ ] Use `if let` for single-variant matching
- [ ] Model a state machine with enums
- [ ] Understand how enums differ from TypeScript unions

## Next Steps

Read [Lecture 04: Error Handling the Rust Way →](./04-error-handling.md) to dive deep into `Result` and idiomatic error handling.

Or start [Exercise 02: Config-Driven CLI →](../exercises/ex02-config-cli/) to practice enums and pattern matching.
