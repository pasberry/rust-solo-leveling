# Lecture 01: Async Fundamentals in Rust

## Introduction

Asynchronous programming allows you to write concurrent code that doesn't block threads while waiting for I/O operations. This is crucial for building high-performance network services that need to handle thousands of concurrent connections.

**Duration:** 60 minutes

## Why Async Programming?

### The Problem with Blocking I/O

Consider a simple web server that handles requests synchronously:

```rust
// Synchronous (blocking) server - DON'T DO THIS
fn handle_request(request: Request) -> Response {
    let data = database.query("SELECT * FROM users"); // Blocks thread for 10ms
    let enriched = external_api.fetch(data);           // Blocks thread for 50ms
    Response::new(enriched)
}

// Main loop
loop {
    let request = listener.accept(); // Blocks until new connection
    handle_request(request);         // Processes one request at a time
}
```

**Problems:**
- While waiting for database response, thread sits idle
- Can only handle one request at a time
- Need one thread per concurrent connection (expensive!)
- 10,000 concurrent connections = 10,000 threads (not feasible)

### The Thread-Per-Connection Approach

```rust
// Thread-per-connection - EXPENSIVE
loop {
    let request = listener.accept();
    thread::spawn(|| {
        handle_request(request);
    });
}
```

**Issues:**
- Each thread consumes ~2MB of stack memory
- Context switching overhead
- Limited by system thread limits (~10k threads max)
- Inefficient for I/O-bound workloads

### The Async Solution

```rust
// Async (non-blocking) - EFFICIENT
async fn handle_request(request: Request) -> Response {
    let data = database.query("SELECT * FROM users").await;
    let enriched = external_api.fetch(data).await;
    Response::new(enriched)
}

// Main loop - handles THOUSANDS of concurrent requests
loop {
    let request = listener.accept().await;
    tokio::spawn(handle_request(request));
}
```

**Benefits:**
- While waiting for I/O, can process other requests
- Thousands of concurrent tasks on a few threads
- Efficient use of system resources
- Better throughput and latency

## Core Concepts

### 1. Futures

A `Future` represents a value that will be available at some point:

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

// This is what every async function returns
pub trait Future {
    type Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),      // Value is ready now
    Pending,       // Not ready yet, try again later
}
```

**Key insight:** Futures are lazy - they do nothing until polled!

```rust
// This doesn't execute anything yet!
let future = async {
    println!("This won't print");
    42
};

// Still nothing happens...
// The future must be .await-ed or run by an executor
```

### 2. Async/Await Syntax

The `async` keyword transforms a function into one that returns a `Future`:

```rust
// Regular function
fn fetch_user(id: u64) -> User {
    // ... blocking code ...
}

// Async function
async fn fetch_user(id: u64) -> User {
    // ... async code with .await ...
}

// What the compiler sees:
fn fetch_user(id: u64) -> impl Future<Output = User> {
    // Returns a Future, doesn't execute immediately
}
```

The `.await` keyword:
- Suspends the current function
- Yields control back to the executor
- Resumes when the awaited future is ready

```rust
async fn example() {
    let user = fetch_user(123).await;  // Suspends here until ready
    println!("Got user: {:?}", user);  // Resumes here
}
```

### 3. The Async Runtime (Tokio)

Futures need an executor to run them. Tokio is the most popular async runtime:

```rust
use tokio;

#[tokio::main]  // This macro sets up the runtime
async fn main() {
    let result = some_async_function().await;
    println!("Result: {}", result);
}

// What the macro expands to:
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let result = some_async_function().await;
        println!("Result: {}", result);
    });
}
```

## Comparing with Other Languages

### TypeScript/JavaScript

```typescript
// TypeScript - Promises are eager, always execute
const promise = fetch('https://api.example.com/users');
// ↑ HTTP request is already in flight!

async function getUser(id: number): Promise<User> {
    const response = await fetch(`/users/${id}`);
    return response.json();
}
```

**Key differences:**
- JavaScript Promises are **eager** (execute immediately)
- Rust Futures are **lazy** (only execute when polled)
- JavaScript is single-threaded with event loop
- Rust can use multiple threads with work-stealing scheduler

### Python (asyncio)

```python
# Python asyncio
async def fetch_user(user_id: int) -> User:
    response = await http_client.get(f"/users/{user_id}")
    return response.json()

# Running the event loop
asyncio.run(fetch_user(123))
```

**Key differences:**
- Python uses cooperative multitasking (like Rust)
- Python has Global Interpreter Lock (GIL), Rust doesn't
- Rust has compile-time async checking, Python is runtime
- Rust async is zero-cost abstraction, Python has overhead

## Practical Example: Concurrent HTTP Requests

### Sequential (Slow)

```rust
async fn fetch_all_users_sequential(ids: Vec<u64>) -> Vec<User> {
    let mut users = Vec::new();

    for id in ids {
        let user = fetch_user(id).await;  // Wait for each one
        users.push(user);
    }

    users
}

// If each request takes 100ms, 10 requests = 1000ms total
```

### Concurrent (Fast)

```rust
use tokio;

async fn fetch_all_users_concurrent(ids: Vec<u64>) -> Vec<User> {
    let futures: Vec<_> = ids
        .into_iter()
        .map(|id| fetch_user(id))  // Create futures (lazy!)
        .collect();

    // Execute all concurrently
    let users = futures::future::join_all(futures).await;
    users
}

// 10 concurrent requests = ~100ms total (network latency)
```

### With Error Handling

```rust
async fn fetch_all_users_safe(ids: Vec<u64>) -> Result<Vec<User>, Error> {
    let futures: Vec<_> = ids
        .into_iter()
        .map(|id| fetch_user(id))
        .collect();

    // If any fails, return error immediately
    futures::future::try_join_all(futures).await
}
```

## The Tokio Runtime Architecture

```
┌─────────────────────────────────────────┐
│         Your Async Code                 │
│    (thousands of async tasks)           │
└────────────────┬────────────────────────┘
                 │
    ┌────────────┼─────────────┐
    │            │             │
┌───▼───┐   ┌───▼───┐    ┌───▼───┐
│Thread │   │Thread │    │Thread │  Work-stealing
│   1   │   │   2   │    │   3   │  thread pool
└───┬───┘   └───┬───┘    └───┬───┘
    │           │            │
    └───────────┴────────────┘
         Shared Task Queue
```

**Key features:**
- Work-stealing scheduler (idle threads steal work from busy ones)
- Automatically uses all CPU cores
- Efficient task switching (no OS context switch overhead)

## Common Patterns

### 1. Spawning Tasks

```rust
use tokio;

#[tokio::main]
async fn main() {
    // Spawn a background task
    let handle = tokio::spawn(async {
        // This runs concurrently
        expensive_computation().await
    });

    // Do other work
    other_work().await;

    // Wait for background task to finish
    let result = handle.await.unwrap();
}
```

### 2. Timeouts

```rust
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout() -> Result<User, Error> {
    match timeout(Duration::from_secs(5), fetch_user(123)).await {
        Ok(user) => Ok(user),
        Err(_) => Err(Error::Timeout),
    }
}
```

### 3. Select (Race Multiple Futures)

```rust
use tokio::select;

async fn either_or() {
    let result = select! {
        user = fetch_from_database() => {
            println!("Database returned first: {:?}", user);
            user
        }
        user = fetch_from_cache() => {
            println!("Cache returned first: {:?}", user);
            user
        }
    };
}
```

## Common Pitfalls

### 1. Holding Locks Across .await

```rust
use std::sync::Mutex;

// ❌ BAD: Lock held during .await
async fn bad_example(data: Arc<Mutex<Vec<u8>>>) {
    let mut guard = data.lock().unwrap();
    let result = some_async_operation().await;  // Lock held here!
    guard.push(result);
}

// ✅ GOOD: Use async-aware lock or minimize critical section
use tokio::sync::Mutex;

async fn good_example(data: Arc<Mutex<Vec<u8>>>) {
    let result = some_async_operation().await;
    let mut guard = data.lock().await;  // Async lock
    guard.push(result);
}
```

### 2. Not .await-ing Futures

```rust
// ❌ BAD: Future created but never executed
async fn oops() {
    fetch_user(123);  // Does nothing! No .await
}

// ✅ GOOD: Actually wait for the result
async fn correct() {
    fetch_user(123).await;
}
```

### 3. Blocking Operations in Async Code

```rust
// ❌ BAD: Blocks the entire thread
async fn bad() {
    let data = std::fs::read_to_string("file.txt").unwrap();  // Blocks!
}

// ✅ GOOD: Use async I/O
async fn good() {
    let data = tokio::fs::read_to_string("file.txt").await.unwrap();
}

// ✅ GOOD: Or spawn blocking work to thread pool
async fn also_good() {
    let data = tokio::task::spawn_blocking(|| {
        std::fs::read_to_string("file.txt").unwrap()
    }).await.unwrap();
}
```

## When to Use Async

**✅ Use async for:**
- Network servers (HTTP, WebSocket, gRPC)
- Database clients
- I/O-bound operations
- High-concurrency scenarios
- Real-time systems

**❌ Don't use async for:**
- CPU-bound computations (use rayon instead)
- Simple scripts
- Code that never does I/O
- When you need guaranteed low latency (OS scheduling is more predictable)

## Performance Characteristics

| Approach | Memory per task | Context switch | Max concurrent |
|----------|----------------|----------------|----------------|
| OS Threads | ~2MB | ~1-10µs | ~10,000 |
| Async tasks | ~2KB | ~10ns | Millions |

**Async wins for I/O-bound workloads:**
- 1000x less memory per task
- 1000x faster context switches
- Can handle millions of concurrent connections

## Exercises

1. **Warm-up:** Write an async function that sleeps for 1 second then returns "Hello"
2. **Concurrent fetch:** Fetch 10 URLs concurrently and print their response times
3. **Timeout:** Implement a function with retry logic and exponential backoff
4. **Race condition:** Use `select!` to implement a timeout for multiple operations

## Key Takeaways

1. **Futures are lazy** - they do nothing until polled
2. **`.await` suspends execution** - yields control to executor
3. **Tokio provides the runtime** - schedules and executes futures
4. **Don't block async code** - use `spawn_blocking` for sync operations
5. **Async is perfect for I/O** - terrible for CPU-bound work

## Next Lecture

In the next lecture, we'll build a real TCP server using Tokio and learn about:
- TcpListener and TcpStream
- Accepting connections asynchronously
- Handling multiple clients concurrently
- Graceful shutdown patterns

## Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Futures Crate Documentation](https://docs.rs/futures/)
- ["Async: What is blocking?" by Alice Ryhl](https://ryhl.io/blog/async-what-is-blocking/)
