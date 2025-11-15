# Module 02: Async + Networking

**Make Network Services Your Superpower**

## Overview

This module teaches you to build high-performance async network services with Rust. You'll learn Tokio, async/await patterns, and how to build production-ready HTTP and TCP servers.

**Duration**: 1-2 weeks (12-16 hours)

## Prerequisites

- Completed Module 01 (Core Rust Fluency)
- Understanding of basic networking concepts (TCP, HTTP)
- Familiarity with async concepts from Node.js or Python asyncio (helpful but not required)

## Learning Objectives

By the end of this module, you will:

1. **Master async/await** in Rust and understand how it differs from Node.js/Python
2. **Build with Tokio** - tasks, spawning, select!, timeouts
3. **Create TCP servers** - echo servers, chat servers, protocol design
4. **Build HTTP APIs** - REST APIs with Axum or Actix-web
5. **Handle concurrency** - Arc, Mutex, RwLock, channels
6. **Manage errors** in async code
7. **Test async code** effectively

## Module Structure

### Lectures

#### Lecture 01: Async Rust Fundamentals
**Topics:**
- What is async/await and why Rust needs it
- Futures and the Poll model
- Comparing to Node.js event loop and Python asyncio
- When to use async vs threads
- The `async fn` and `.await` syntax
- Pinning and `Unpin` (brief overview)

**Key Concepts:**
```rust
// Async functions return impl Future
async fn fetch_data() -> Result<String, Error> {
    let response = http_client.get("https://api.example.com").await?;
    Ok(response.text().await?)
}

// Futures are lazy - nothing happens until awaited or spawned
let future = fetch_data();  // No work done yet
let result = future.await;  // Now it executes
```

**Mental Model:**
- Async in Rust is zero-cost and doesn't require a runtime by default
- You choose your runtime (Tokio, async-std, smol)
- Futures compose without allocation
- Compared to Node: More explicit, no hidden event loop, compile-time checked

#### Lecture 02: Tokio Runtime and Tasks
**Topics:**
- Setting up Tokio
- The `#[tokio::main]` macro
- Spawning tasks with `tokio::spawn`
- Task cancellation and JoinHandles
- `select!` for racing futures
- Timeouts and intervals
- Blocking operations and `spawn_blocking`

**Key Patterns:**
```rust
#[tokio::main]
async fn main() {
    // Spawn concurrent tasks
    let handle1 = tokio::spawn(async { task_one().await });
    let handle2 = tokio::spawn(async { task_two().await });

    // Wait for both
    let (res1, res2) = tokio::join!(handle1, handle2);

    // Race tasks
    tokio::select! {
        result = async_operation() => println!("Completed: {:?}", result),
        _ = tokio::time::sleep(Duration::from_secs(5)) => println!("Timeout"),
    }
}
```

#### Lecture 03: TCP Networking with Tokio
**Topics:**
- TcpListener and TcpStream
- Reading and writing async
- Buffering strategies
- Handling multiple connections
- Graceful shutdown
- Protocol framing

**Example Server:**
```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(handle_client(socket, addr));
    }
}

async fn handle_client(mut socket: TcpStream, addr: SocketAddr) -> Result<()> {
    let mut buf = vec![0; 1024];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 { return Ok(()); }  // Connection closed

        socket.write_all(&buf[..n]).await?;
    }
}
```

#### Lecture 04: HTTP Services with Axum
**Topics:**
- Why Axum? (vs Actix, Rocket, Warp)
- Routing and handlers
- Extractors (Path, Query, Json)
- State management with Arc
- Middleware and layers
- Error handling in HTTP contexts
- Testing HTTP handlers

**Example API:**
```rust
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    db: Arc<RwLock<HashMap<u64, User>>>,
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUser>,
) -> Json<User> {
    let mut db = state.db.write().await;
    let user = User {
        id: db.len() as u64,
        name: payload.name,
    };
    db.insert(user.id, user.clone());
    Json(user)
}

#[tokio::main]
async fn main() {
    let state = AppState {
        db: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}
```

#### Lecture 05: Concurrency Patterns
**Topics:**
- Arc vs Rc (thread-safe reference counting)
- Mutex, RwLock, Semaphore
- Channels: mpsc, broadcast, watch, oneshot
- When to use each primitive
- Avoiding deadlocks
- Performance considerations
- Atomic operations

**Patterns:**
```rust
// Shared state with Mutex
use std::sync::Arc;
use tokio::sync::Mutex;

let counter = Arc::new(Mutex::new(0));

for _ in 0..10 {
    let counter = Arc::clone(&counter);
    tokio::spawn(async move {
        let mut num = counter.lock().await;
        *num += 1;
    });
}

// Message passing with channels
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(32);

tokio::spawn(async move {
    tx.send("hello").await.unwrap();
});

while let Some(msg) = rx.recv().await {
    println!("got: {}", msg);
}

// Broadcast for fan-out
use tokio::sync::broadcast;

let (tx, mut rx1) = broadcast::channel(16);
let mut rx2 = tx.subscribe();

tokio::spawn(async move {
    while let Ok(msg) = rx1.recv().await {
        println!("rx1: {}", msg);
    }
});

tokio::spawn(async move {
    while let Ok(msg) = rx2.recv().await {
        println!("rx2: {}", msg);
    }
});

tx.send("broadcast message").unwrap();
```

#### Lecture 06: Error Handling in Async Code
**Topics:**
- Error propagation with `?` in async functions
- `anyhow` vs `thiserror` in async contexts
- Handling errors in spawned tasks
- Timeout and cancellation errors
- Error boundaries and recovery

#### Lecture 07: Testing Async Code
**Topics:**
- `#[tokio::test]` macro
- Testing async functions
- Mocking async traits
- Integration testing for HTTP servers
- Property testing with async

**Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_data() {
        let result = fetch_data().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_http_endpoint() {
        let app = create_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
```

### Exercises

#### Exercise 01: Async TCP Echo Server
**Goal**: Build a TCP server that echoes messages back to clients

**Features:**
- Accept multiple concurrent connections
- Echo each message back
- Handle client disconnects gracefully
- Add timeout for idle connections
- Log connections and disconnections

**Skills:**
- TcpListener and TcpStream
- Spawning tasks
- Async read/write
- Timeouts
- Error handling

**Starter Code**: `exercises/ex01-echo-server/`
**Solution**: `solutions/ex01-echo-server/`

#### Exercise 02: Async Chat Server
**Goal**: Build a multi-client chat room server

**Features:**
- Multiple clients can connect
- Messages are broadcast to all clients
- Support /nick command to set nickname
- Support /list to see connected users
- Handle client disconnects
- Persist chat history (optional)

**Skills:**
- Broadcast channels
- Shared state with Arc<Mutex<>>
- Command parsing
- Protocol design
- Concurrent HashMap updates

**Starter Code**: `exercises/ex02-chat-server/`
**Solution**: `solutions/ex02-chat-server/`

#### Exercise 03: HTTP JSON API (CRUD)
**Goal**: Build a RESTful API for managing a resource (e.g., todos, users)

**Endpoints:**
```
POST   /items      - Create item
GET    /items      - List all items
GET    /items/:id  - Get specific item
PUT    /items/:id  - Update item
DELETE /items/:id  - Delete item
```

**Features:**
- In-memory storage with Arc<RwLock<HashMap>>
- JSON request/response
- Proper HTTP status codes
- Error handling and validation
- CORS support (optional)
- Request logging middleware

**Skills:**
- Axum routing and extractors
- Shared state
- JSON serialization
- HTTP best practices
- Middleware

**Starter Code**: `exercises/ex03-http-api/`
**Solution**: `solutions/ex03-http-api/`

## Comparing to Other Languages

### Node.js vs Rust Async

| Aspect | Node.js | Rust (Tokio) |
|--------|---------|--------------|
| **Runtime** | Built-in event loop | Explicit runtime (Tokio) |
| **Async by default** | Yes (all I/O is async) | No (opt-in with `async`) |
| **Syntax** | `async`/`await` | `async`/`await` |
| **Errors** | Exceptions or callbacks | `Result<T, E>` |
| **Concurrency** | Single-threaded event loop | Multi-threaded work-stealing |
| **Type safety** | TypeScript (optional) | Always type-safe |
| **Performance** | Fast for I/O-bound | Faster, no GC pauses |

**Example Comparison:**

```javascript
// Node.js
async function fetchUser(id) {
    try {
        const response = await fetch(`/users/${id}`);
        const user = await response.json();
        return user;
    } catch (error) {
        console.error(error);
        throw error;
    }
}
```

```rust
// Rust
async fn fetch_user(id: u64) -> Result<User, Error> {
    let response = client.get(&format!("/users/{}", id)).await?;
    let user = response.json::<User>().await?;
    Ok(user)
}
```

### Python Asyncio vs Rust Async

| Aspect | Python asyncio | Rust (Tokio) |
|--------|----------------|--------------|
| **Runtime** | Built-in event loop | Tokio runtime |
| **Syntax** | `async`/`await` | `async`/`await` |
| **Typing** | Optional (type hints) | Always typed |
| **GIL** | Yes (limits parallelism) | No GIL, true parallelism |
| **Performance** | Slower | Much faster |
| **Ecosystem** | Mature but fragmented | Growing, consistent |

## Best Practices

1. **Don't block the runtime**: Use `spawn_blocking` for CPU-intensive or blocking operations
2. **Use `Arc<Mutex<>>` sparingly**: Prefer message passing with channels when possible
3. **Timeout long operations**: Always add timeouts to prevent hanging
4. **Handle task panics**: Use `JoinHandle` to detect task failures
5. **Size your channels**: Bounded channels prevent memory issues
6. **Test with concurrency**: Use tools like `loom` for concurrent testing
7. **Profile async code**: Use `tokio-console` for runtime introspection

## Common Pitfalls

### Pitfall 1: Forgetting `.await`

```rust
// ❌ Future created but never executed
let future = fetch_data();
// nothing happens!

// ✅ Await the future
let result = fetch_data().await?;
```

### Pitfall 2: Blocking in Async

```rust
// ❌ Blocks the entire runtime
async fn bad() {
    std::thread::sleep(Duration::from_secs(1));  // BAD!
}

// ✅ Use async sleep
async fn good() {
    tokio::time::sleep(Duration::from_secs(1)).await;  // Good!
}

// ✅ Or spawn_blocking for CPU work
async fn process_heavy() {
    let result = tokio::task::spawn_blocking(|| {
        // CPU-intensive work here
        expensive_computation()
    }).await?;
}
```

### Pitfall 3: Over-cloning Arc

```rust
// ❌ Unnecessary clones
for i in 0..100 {
    let data = data.clone();  // Arc clone inside loop
    tokio::spawn(async move {
        process(data).await;
    });
}

// ✅ Clone once per spawn
for i in 0..100 {
    let data = Arc::clone(&data);  // Explicit and clear
    tokio::spawn(async move {
        process(data).await;
    });
}
```

## Resources

- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Async Book](https://rust-lang.github.io/async-book/)
- [Axum Examples](https://github.com/tokio-rs/axum/tree/main/examples)
- [Jon Gjengset's "Crust of Rust: async/await"](https://www.youtube.com/watch?v=ThjvMReOXYM)

## Success Criteria

You're ready for Module 03 when you can:

- ✅ Explain how async/await works in Rust
- ✅ Build a TCP server handling multiple connections
- ✅ Create an HTTP API with proper routing and state
- ✅ Choose appropriate concurrency primitives (Mutex, channels, etc.)
- ✅ Handle errors gracefully in async contexts
- ✅ Write tests for async code

## Next Steps

After mastering async Rust, you're ready to build real systems!

Proceed to [Module 03: Key-Value Store →](../module-03-kv-store/) to build your first storage engine.
