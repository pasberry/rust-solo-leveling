# Lecture 02: Building TCP Servers with Tokio

## Introduction

TCP (Transmission Control Protocol) is the foundation of most network communication. In this lecture, you'll learn to build concurrent TCP servers that can handle thousands of connections simultaneously.

**Duration:** 75 minutes

## TCP Basics

### What is TCP?

TCP provides:
- **Reliable delivery** - packets arrive in order, or you get an error
- **Connection-oriented** - explicit setup (handshake) and teardown
- **Stream-based** - continuous flow of bytes, not discrete messages
- **Flow control** - prevents overwhelming the receiver

### Client-Server Model

```
Client                          Server
  │                               │
  │  ──── SYN ────────────────►  │  (Listening)
  │  ◄─── SYN-ACK ────────────   │
  │  ──── ACK ────────────────►  │  (Connection established)
  │                               │
  │  ──── Data ───────────────►  │
  │  ◄─── Data ───────────────   │
  │                               │
  │  ──── FIN ────────────────►  │
  │  ◄─── ACK ────────────────   │  (Connection closed)
```

## Building Your First TCP Server

### Echo Server (Simplest Example)

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bind to address
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080");

    loop {
        // Accept new connection
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        // Handle connection concurrently
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket).await {
                eprintln!("Error handling connection: {}", e);
            }
        });
    }
}

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = vec![0; 1024];

    loop {
        // Read data from socket
        let n = socket.read(&mut buffer).await?;

        // Connection closed
        if n == 0 {
            return Ok(());
        }

        // Echo data back
        socket.write_all(&buffer[..n]).await?;
    }
}
```

**Testing with telnet:**
```bash
telnet localhost 8080
# Type anything and it echoes back
```

### Breaking It Down

#### 1. Creating a Listener

```rust
let listener = TcpListener::bind("127.0.0.1:8080").await?;
```

- **Binds to address and port** - reserves the port
- **Returns a future** - must `.await`
- **Supports IPv4 and IPv6**: `"[::1]:8080"` for IPv6 localhost

Binding options:
```rust
// Specific interface
TcpListener::bind("192.168.1.100:8080").await?;

// All interfaces
TcpListener::bind("0.0.0.0:8080").await?;

// Multiple addresses (tries each until one succeeds)
TcpListener::bind(&["127.0.0.1:8080", "0.0.0.0:8080"][..]).await?;
```

#### 2. Accepting Connections

```rust
let (socket, addr) = listener.accept().await?;
```

- **Blocks until connection arrives** - suspends task (non-blocking!)
- **Returns TcpStream + SocketAddr** - socket for I/O, addr for logging
- **Each connection gets own socket** - isolated from others

#### 3. Spawning Tasks

```rust
tokio::spawn(async move {
    handle_connection(socket).await;
});
```

- **Spawns concurrent task** - doesn't block accepting new connections
- **`move` captures socket** - transfers ownership to task
- **Returns JoinHandle** - can `.await` to get result

#### 4. Reading and Writing

```rust
// Read into buffer
let n = socket.read(&mut buffer).await?;

// Write entire buffer
socket.write_all(&buffer[..n]).await?;
```

**Important:** `read()` doesn't guarantee filling the buffer!
- May return partial data
- Returns 0 when connection closed
- May return less than buffer size

## Improved Echo Server with Framing

Real protocols need message boundaries. Let's add newline-delimited framing:

```rust
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

async fn handle_connection(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, mut writer) = socket.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        println!("Received: {}", line);

        // Echo back with newline
        writer.write_all(line.as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    Ok(())
}
```

**Key improvements:**
- `BufReader` - efficient buffering
- `lines()` - automatically handles line splitting
- `into_split()` - separate reader and writer

## Chat Server (Multi-Client Broadcast)

Let's build a real application - a chat room where messages are broadcast to all clients:

```rust
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    // Broadcast channel for messages
    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(100);

    println!("Chat server listening on port 8080");

    loop {
        let (socket, addr) = listener.accept().await?;
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.into_split();
            let reader = BufReader::new(reader);
            let mut lines = reader.lines();

            // Welcome message
            let _ = writer.write_all(b"Welcome to the chat! Type your message:\n").await;

            // Spawn task to receive broadcasts
            let mut send_task = tokio::spawn(async move {
                while let Ok((msg, sender_addr)) = rx.recv().await {
                    // Don't echo own messages
                    if sender_addr != addr {
                        let line = format!("[{}]: {}\n", sender_addr, msg);
                        if writer.write_all(line.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                }
            });

            // Read and broadcast messages
            while let Some(line) = lines.next_line().await.ok().flatten() {
                println!("[{}]: {}", addr, line);
                let _ = tx.send((line, addr));
            }

            send_task.abort();
        });
    }
}
```

**Testing:**
```bash
# Terminal 1
telnet localhost 8080

# Terminal 2
telnet localhost 8080

# Type in either terminal - messages appear in both!
```

### How It Works

1. **Broadcast channel** - `broadcast::channel` allows multiple receivers
2. **Each client subscribes** - gets own receiver with `.subscribe()`
3. **Messages sent to all** - one `tx.send()` reaches all `rx` receivers
4. **Separate read/write tasks** - can send and receive simultaneously

## Connection Lifecycle Management

### Graceful Shutdown

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    // Shutdown signal
    let (shutdown_tx, _) = broadcast::channel::<()>(1);

    // Spawn accept loop
    let accept_task = {
        let shutdown_rx = shutdown_tx.subscribe();
        tokio::spawn(accept_loop(listener, shutdown_rx))
    };

    // Wait for Ctrl+C
    signal::ctrl_c().await?;
    println!("Shutting down...");

    // Signal all tasks to stop
    let _ = shutdown_tx.send(());

    // Wait for accept loop to finish
    accept_task.await??;

    Ok(())
}

async fn accept_loop(
    listener: TcpListener,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        tokio::select! {
            result = listener.accept() => {
                let (socket, addr) = result?;
                tokio::spawn(handle_connection(socket, addr));
            }
            _ = shutdown.recv() => {
                println!("Accept loop shutting down");
                break;
            }
        }
    }

    Ok(())
}
```

### Connection Timeout

```rust
use tokio::time::{timeout, Duration};

async fn handle_connection_with_timeout(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    match timeout(Duration::from_secs(30), handle_connection(socket)).await {
        Ok(result) => result,
        Err(_) => {
            println!("Connection timed out");
            Ok(())
        }
    }
}
```

### Connection Limits

```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    // Limit to 100 concurrent connections
    let semaphore = Arc::new(Semaphore::new(100));

    loop {
        let (socket, addr) = listener.accept().await?;

        // Acquire permit
        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(permit) => permit,
            Err(_) => {
                println!("Connection limit reached, rejecting {}", addr);
                continue;
            }
        };

        tokio::spawn(async move {
            let _permit = permit;  // Released when dropped
            handle_connection(socket).await;
        });
    }
}
```

## Error Handling Patterns

### Per-Connection Errors

```rust
async fn handle_connection(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Connection-specific errors (client disconnects, etc.)
    // Should not crash the server
    Ok(())
}

// In main loop:
tokio::spawn(async move {
    if let Err(e) = handle_connection(socket).await {
        eprintln!("Connection error: {}", e);
    }
});
```

### Fatal Errors

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                tokio::spawn(async move {
                    handle_connection(socket).await;
                });
            }
            Err(e) => {
                // Fatal error - can't accept new connections
                eprintln!("Failed to accept connection: {}", e);
                return Err(e.into());
            }
        }
    }
}
```

## Performance Optimization

### 1. Buffer Pooling

```rust
use bytes::BytesMut;

async fn handle_connection(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = BytesMut::with_capacity(4096);

    loop {
        let n = socket.read_buf(&mut buf).await?;
        if n == 0 { break; }

        // Process data in buf
        process_data(&buf[..n]);

        // Clear for reuse
        buf.clear();
    }

    Ok(())
}
```

### 2. TCP_NODELAY for Low Latency

```rust
use tokio::net::TcpStream;

async fn handle_connection(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    // Disable Nagle's algorithm for lower latency
    socket.set_nodelay(true)?;

    // ... handle connection
    Ok(())
}
```

### 3. Explicit Backpressure

```rust
async fn handle_connection(socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let (reader, writer) = socket.into_split();
    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(10);  // Bounded channel

    // Reader task
    tokio::spawn(async move {
        let mut reader = BufReader::new(reader);
        let mut buffer = vec![0; 1024];

        loop {
            let n = reader.read(&mut buffer).await.unwrap();
            if n == 0 { break; }

            // Blocks if channel full (backpressure!)
            if tx.send(buffer[..n].to_vec()).await.is_err() {
                break;
            }
        }
    });

    // Writer task
    let mut writer = writer;
    while let Some(data) = rx.recv().await {
        writer.write_all(&data).await?;
    }

    Ok(())
}
```

## Comparison with Other Languages

### Node.js (TypeScript)

```typescript
import * as net from 'net';

const server = net.createServer((socket) => {
    socket.on('data', (data) => {
        socket.write(data);  // Echo
    });

    socket.on('error', (err) => {
        console.error('Socket error:', err);
    });
});

server.listen(8080, () => {
    console.log('Server listening on port 8080');
});
```

**Differences:**
- Node.js is callback-based (event-driven)
- Rust uses async/await (more readable)
- Node.js single-threaded, Rust multi-threaded
- Rust has compile-time memory safety

### Python (asyncio)

```python
import asyncio

async def handle_connection(reader, writer):
    while True:
        data = await reader.read(1024)
        if not data:
            break
        writer.write(data)
        await writer.drain()

    writer.close()

async def main():
    server = await asyncio.start_server(
        handle_connection, '127.0.0.1', 8080
    )
    async with server:
        await server.serve_forever()

asyncio.run(main())
```

**Differences:**
- Python syntax cleaner for async
- Rust faster (no GIL, compiled)
- Rust has stronger type safety
- Python easier for prototyping

## Common Pitfalls

### 1. Not Handling Connection Closure

```rust
// ❌ BAD: Infinite loop on closed connection
async fn bad_handler(mut socket: TcpStream) {
    let mut buf = vec![0; 1024];
    loop {
        socket.read(&mut buf).await.unwrap();  // Panics on close!
        socket.write_all(&buf).await.unwrap();
    }
}

// ✅ GOOD: Check for 0 bytes (connection closed)
async fn good_handler(mut socket: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024];
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 { break; }  // Connection closed
        socket.write_all(&buf[..n]).await?;
    }
    Ok(())
}
```

### 2. Unbounded Resource Usage

```rust
// ❌ BAD: Accepts unlimited connections
loop {
    let (socket, _) = listener.accept().await?;
    tokio::spawn(handle_connection(socket));  // Could spawn millions!
}

// ✅ GOOD: Use semaphore for limits
let semaphore = Arc::new(Semaphore::new(MAX_CONNECTIONS));
loop {
    let permit = semaphore.clone().acquire_owned().await?;
    let (socket, _) = listener.accept().await?;
    tokio::spawn(async move {
        let _permit = permit;
        handle_connection(socket).await;
    });
}
```

### 3. Not Flushing Writes

```rust
// ❌ BAD: Data might sit in kernel buffer
socket.write(&data).await?;

// ✅ GOOD: Ensure data is sent
socket.write_all(&data).await?;
socket.flush().await?;  // Force send
```

## Exercises

1. **Line Counter Server**: Accept connections and count lines sent by client
2. **File Server**: Send a file over TCP when client connects
3. **Concurrent Echo**: Build echo server that handles 1000+ concurrent connections
4. **Chat Server**: Extend the chat example with usernames and private messages

## Key Takeaways

1. **`TcpListener::accept()` is the core** - returns new connections
2. **Spawn tasks for each connection** - enables concurrency
3. **Always check for 0 bytes** - indicates connection closed
4. **Use channels for inter-task communication** - broadcast for pub/sub
5. **Set connection limits** - prevent resource exhaustion
6. **Graceful shutdown is important** - use select! with signals

## Next Lecture

We'll learn about HTTP servers using `axum`, including:
- Routing and handlers
- JSON serialization
- Middleware
- State management
- Error handling

## Resources

- [Tokio TCP Tutorial](https://tokio.rs/tokio/tutorial/io)
- [Mini-Redis Tutorial](https://tokio.rs/tokio/tutorial)
- [TCP RFC 793](https://www.rfc-editor.org/rfc/rfc793)
