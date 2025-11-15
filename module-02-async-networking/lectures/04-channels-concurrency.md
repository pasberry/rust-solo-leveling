# Lecture 04: Channels and Concurrent Patterns

## Introduction

Channels are the primary way to communicate between async tasks. They enable message-passing concurrency, a core paradigm in async Rust.

**Duration:** 75 minutes

## Channel Types in Tokio

### 1. mpsc (Multi-Producer, Single-Consumer)

Use when multiple tasks send messages to one receiver:

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(100);  // Buffer size: 100

    // Spawn producer tasks
    for i in 0..10 {
        let tx = tx.clone();
        tokio::spawn(async move {
            tx.send(format!("Message {}", i)).await.unwrap();
        });
    }

    // Drop original sender so receiver knows when done
    drop(tx);

    // Receive all messages
    while let Some(msg) = rx.recv().await {
        println!("Received: {}", msg);
    }
}
```

**Bounded vs Unbounded:**

```rust
// Bounded - blocks sender if full (backpressure)
let (tx, rx) = mpsc::channel::<i32>(10);

// Unbounded - never blocks sender (can cause OOM!)
let (tx, rx) = mpsc::unbounded_channel::<i32>();
```

### 2. broadcast (Multi-Producer, Multi-Consumer)

Use for pub/sub patterns where all receivers get all messages:

```rust
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let (tx, mut rx1) = broadcast::channel::<String>(100);
    let mut rx2 = tx.subscribe();
    let mut rx3 = tx.subscribe();

    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(format!("Message {}", i)).unwrap();
        }
    });

    // All receivers get the same messages
    tokio::join!(
        async move {
            while let Ok(msg) = rx1.recv().await {
                println!("RX1: {}", msg);
            }
        },
        async move {
            while let Ok(msg) = rx2.recv().await {
                println!("RX2: {}", msg);
            }
        },
        async move {
            while let Ok(msg) = rx3.recv().await {
                println!("RX3: {}", msg);
            }
        }
    );
}
```

### 3. watch (Single-Producer, Multi-Consumer State)

Use for broadcasting state changes:

```rust
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = watch::channel("initial");

    tokio::spawn(async move {
        for value in &["first", "second", "third"] {
            tx.send(*value).unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    // Receiver only sees latest value
    while rx.changed().await.is_ok() {
        println!("Value changed to: {}", *rx.borrow());
    }
}
```

### 4. oneshot (One-Time Message)

Use for single request-response:

```rust
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel::<String>();

    tokio::spawn(async move {
        // Do expensive work
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        tx.send("Result".to_string()).unwrap();
    });

    // Wait for result
    let result = rx.await.unwrap();
    println!("Got: {}", result);
}
```

## Practical Patterns

### Worker Pool Pattern

```rust
use tokio::sync::mpsc;

struct WorkItem {
    id: u64,
    data: Vec<u8>,
}

async fn worker(id: usize, mut rx: mpsc::Receiver<WorkItem>) {
    while let Some(item) = rx.recv().await {
        println!("Worker {} processing item {}", id, item.id);
        // Process work...
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    println!("Worker {} shutting down", id);
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<WorkItem>(100);

    // Spawn worker pool
    let mut workers = Vec::new();
    for i in 0..4 {
        let rx = rx.clone();  // Error! Can't clone receiver
    }
}
```

**Problem:** Can't clone mpsc receiver! **Solution:** Use multiple channels or a shared queue:

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

async fn worker_pool_correct() {
    let (tx, rx) = mpsc::channel::<WorkItem>(100);
    let rx = Arc::new(Mutex::new(rx));

    // Spawn workers
    let mut workers = Vec::new();
    for i in 0..4 {
        let rx = Arc::clone(&rx);
        let worker = tokio::spawn(async move {
            loop {
                let item = {
                    let mut rx = rx.lock().await;
                    rx.recv().await
                };

                match item {
                    Some(item) => {
                        println!("Worker {} processing {}", i, item.id);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    None => break,
                }
            }
        });
        workers.push(worker);
    }

    // Send work
    for i in 0..20 {
        tx.send(WorkItem { id: i, data: vec![] }).await.unwrap();
    }

    drop(tx);  // Signal workers to stop

    // Wait for all workers
    for worker in workers {
        worker.await.unwrap();
    }
}
```

### Actor Pattern

```rust
use tokio::sync::{mpsc, oneshot};

enum Message {
    Get { respond_to: oneshot::Sender<u64> },
    Increment,
    Decrement,
}

struct CounterActor {
    receiver: mpsc::Receiver<Message>,
    count: u64,
}

impl CounterActor {
    fn new(receiver: mpsc::Receiver<Message>) -> Self {
        CounterActor { receiver, count: 0 }
    }

    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                Message::Get { respond_to } => {
                    let _ = respond_to.send(self.count);
                }
                Message::Increment => {
                    self.count += 1;
                }
                Message::Decrement => {
                    self.count = self.count.saturating_sub(1);
                }
            }
        }
    }
}

#[derive(Clone)]
struct CounterHandle {
    sender: mpsc::Sender<Message>,
}

impl CounterHandle {
    fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let actor = CounterActor::new(receiver);
        tokio::spawn(actor.run());

        CounterHandle { sender }
    }

    async fn get(&self) -> u64 {
        let (tx, rx) = oneshot::channel();
        self.sender.send(Message::Get { respond_to: tx }).await.unwrap();
        rx.await.unwrap()
    }

    async fn increment(&self) {
        self.sender.send(Message::Increment).await.unwrap();
    }

    async fn decrement(&self) {
        self.sender.send(Message::Decrement).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let counter = CounterHandle::new();

    counter.increment().await;
    counter.increment().await;
    counter.decrement().await;

    let value = counter.get().await;
    println!("Counter value: {}", value);  // 1
}
```

## Select and Join

### select! Macro

Race multiple futures, act on first to complete:

```rust
use tokio::select;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(100);

    tokio::spawn(async move {
        sleep(Duration::from_secs(1)).await;
        tx.send("Message from channel".to_string()).await.unwrap();
    });

    select! {
        msg = rx.recv() => {
            println!("Got message: {:?}", msg);
        }
        _ = sleep(Duration::from_millis(500)) => {
            println!("Timeout!");
        }
    }
}
```

**Biased selection:**
```rust
select! {
    biased;  // Check branches in order, don't randomize

    msg = high_priority_rx.recv() => { /* ... */ }
    msg = low_priority_rx.recv() => { /* ... */ }
}
```

### join! and try_join!

Wait for multiple futures to complete:

```rust
use tokio::join;

async fn fetch_user() -> User { /* ... */ }
async fn fetch_posts() -> Vec<Post> { /* ... */ }
async fn fetch_comments() -> Vec<Comment> { /* ... */ }

#[tokio::main]
async fn main() {
    // All run concurrently
    let (user, posts, comments) = join!(
        fetch_user(),
        fetch_posts(),
        fetch_comments()
    );

    println!("Got {} posts and {} comments", posts.len(), comments.len());
}
```

With error handling:

```rust
use tokio::try_join;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Stops on first error
    let (user, posts, comments) = try_join!(
        fetch_user(),
        fetch_posts(),
        fetch_comments()
    )?;

    Ok(())
}
```

## Cancellation and Timeouts

### Task Cancellation

```rust
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        loop {
            println!("Working...");
            sleep(Duration::from_millis(500)).await;
        }
    });

    // Let it run for 2 seconds
    sleep(Duration::from_secs(2)).await;

    // Cancel the task
    handle.abort();

    match handle.await {
        Ok(_) => println!("Task completed"),
        Err(e) if e.is_cancelled() => println!("Task was cancelled"),
        Err(e) => println!("Task failed: {}", e),
    }
}
```

### Graceful Shutdown

```rust
use tokio::sync::broadcast;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Spawn workers
    for i in 0..3 {
        let mut shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(1)) => {
                        println!("Worker {} doing work", i);
                    }
                    _ = shutdown_rx.recv() => {
                        println!("Worker {} shutting down", i);
                        break;
                    }
                }
            }
        });
    }

    // Wait for Ctrl+C
    signal::ctrl_c().await?;
    println!("Shutdown signal received");

    // Send shutdown signal
    let _ = shutdown_tx.send(());

    // Give workers time to cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(())
}
```

### Timeout Pattern

```rust
use tokio::time::{timeout, Duration};

async fn with_retry<F, T>(mut f: F, retries: usize) -> Result<T, String>
where
    F: FnMut() -> BoxFuture<'static, Result<T, String>>,
{
    for attempt in 0..=retries {
        match timeout(Duration::from_secs(5), f()).await {
            Ok(Ok(result)) => return Ok(result),
            Ok(Err(e)) => {
                eprintln!("Attempt {} failed: {}", attempt, e);
                if attempt == retries {
                    return Err(e);
                }
            }
            Err(_) => {
                eprintln!("Attempt {} timed out", attempt);
                if attempt == retries {
                    return Err("Operation timed out".to_string());
                }
            }
        }

        // Exponential backoff
        let delay = Duration::from_millis(100 * 2_u64.pow(attempt as u32));
        tokio::time::sleep(delay).await;
    }

    unreachable!()
}
```

## Common Patterns

### Fan-Out/Fan-In

```rust
use tokio::sync::mpsc;
use futures::future::join_all;

async fn fan_out_fan_in() {
    let (result_tx, mut result_rx) = mpsc::channel::<u64>(100);

    // Fan-out: spawn multiple workers
    let mut handles = Vec::new();
    for i in 0..10 {
        let result_tx = result_tx.clone();
        let handle = tokio::spawn(async move {
            // Simulate work
            tokio::time::sleep(Duration::from_millis(100)).await;
            result_tx.send(i * i).await.unwrap();
        });
        handles.push(handle);
    }

    drop(result_tx);  // Close channel

    // Fan-in: collect results
    let mut results = Vec::new();
    while let Some(result) = result_rx.recv().await {
        results.push(result);
    }

    println!("Results: {:?}", results);
}
```

### Pipeline Pattern

```rust
async fn pipeline() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(10);
    let (tx2, mut rx2) = mpsc::channel::<i32>(10);
    let (tx3, mut rx3) = mpsc::channel::<String>(10);

    // Stage 1: Generate numbers
    tokio::spawn(async move {
        for i in 0..10 {
            tx1.send(i).await.unwrap();
        }
    });

    // Stage 2: Square numbers
    tokio::spawn(async move {
        while let Some(n) = rx1.recv().await {
            tx2.send(n * n).await.unwrap();
        }
    });

    // Stage 3: Format as strings
    tokio::spawn(async move {
        while let Some(n) = rx2.recv().await {
            tx3.send(format!("Result: {}", n)).await.unwrap();
        }
    });

    // Collect final results
    while let Some(result) = rx3.recv().await {
        println!("{}", result);
    }
}
```

## Performance Tips

### 1. Choose Right Channel Type

```rust
// High throughput, many senders → mpsc with large buffer
let (tx, rx) = mpsc::channel::<Message>(10000);

// State updates, latest value only → watch
let (tx, rx) = watch::channel(initial_state);

// Broadcasting to many → broadcast
let (tx, _) = broadcast::channel::<Event>(1000);
```

### 2. Avoid Async Mutex When Possible

```rust
// ❌ Slow: Lock held across await points
use tokio::sync::Mutex;
let data = Arc::new(Mutex::new(Vec::new()));
let mut guard = data.lock().await;
expensive_operation().await;  // Lock held!
guard.push(result);

// ✅ Better: Use channels
let (tx, mut rx) = mpsc::channel(100);
tokio::spawn(async move {
    while let Some(item) = rx.recv().await {
        data.push(item);  // No lock needed!
    }
});
```

### 3. Batch Operations

```rust
// Batch small messages to reduce channel overhead
let (tx, mut rx) = mpsc::channel::<Vec<Item>>(100);

// Send in batches
let mut batch = Vec::new();
for item in items {
    batch.push(item);
    if batch.len() >= 100 {
        tx.send(batch).await.unwrap();
        batch = Vec::new();
    }
}
```

## Exercises

1. **Rate Limiter**: Implement a token bucket rate limiter using channels
2. **Pipeline**: Build a 3-stage processing pipeline (read, process, write)
3. **Actor System**: Create a simple actor system with message passing
4. **Graceful Shutdown**: Implement server with clean shutdown on Ctrl+C

## Key Takeaways

1. **mpsc for task communication** - most common pattern
2. **broadcast for pub/sub** - all receivers get messages
3. **watch for state** - receivers see latest value
4. **select! for racing** - act on first completion
5. **Actors encapsulate state** - clean concurrency model

## Next Lecture

Error handling in async code and building resilient services.

## Resources

- [Tokio Channels](https://tokio.rs/tokio/tutorial/channels)
- [Async Book - Streams](https://rust-lang.github.io/async-book/05_streams/01_chapter.html)
