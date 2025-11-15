# Redis Clone - Design Commentary

## Architecture Overview

This Redis clone implements a complete in-memory data store with:
- **RESP Protocol** - Binary-safe serialization compatible with redis-cli
- **Multiple Data Types** - Strings, lists, sets, and hashes
- **Expiration** - TTL support with background cleanup
- **Concurrent Access** - Arc<RwLock> for thread-safe operations
- **Type Safety** - WRONGTYPE errors when operating on wrong data type

## Key Design Decisions

### 1. RESP Protocol Implementation

```rust
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Option<Vec<RespValue>>),
}
```

**Why RESP?**
- Redis standard protocol
- Binary-safe (can store any bytes)
- Self-describing format
- Easy to parse and generate
- Compatible with existing Redis clients

**Protocol Format:**
```
Simple String: +OK\r\n
Error:         -Error message\r\n
Integer:       :1000\r\n
Bulk String:   $6\r\nfoobar\r\n
Array:         *2\r\n$3\r\nfoo\r\n$3\r\nbar\r\n
```

**Implementation Notes:**
- Cursor-based parsing for efficient buffer management
- Handles incomplete messages (returns `Incomplete` error)
- Proper \r\n termination validation
- Support for null values ($-1\r\n, *-1\r\n)

### 2. Data Storage with Multiple Types

```rust
enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    Hash(HashMap<String, Vec<u8>>),
}

struct Entry {
    value: Value,
    expires_at: Option<Instant>,
}
```

**Why enum for values?**
- Type-safe operations (can't accidentally treat string as list)
- Clear WRONGTYPE errors
- Memory efficient (no vtables like trait objects)
- Pattern matching for exhaustive handling

**Data Structure Choices:**
- **String**: `Vec<u8>` for binary safety
- **List**: `VecDeque` for O(1) push/pop from both ends
- **Set**: `HashSet` for O(1) membership checks
- **Hash**: `HashMap` for O(1) field access

### 3. Concurrency Model

```rust
pub struct Db {
    data: Arc<RwLock<HashMap<String, Entry>>>,
}
```

**Why Arc<RwLock>?**
- Multiple readers, single writer
- GET operations don't block each other
- Only writes require exclusive lock
- Simple to reason about (no complex lock ordering)

**Alternative: Sharded locks**
```rust
// For higher concurrency (more complex)
struct ShardedDb {
    shards: Vec<RwLock<HashMap<String, Entry>>>,
}

fn get_shard(&self, key: &str) -> usize {
    hash(key) % self.shards.len()
}
```

**Tradeoffs:**
- Current: Simple, sufficient for most workloads
- Sharded: Better write concurrency, more complex

### 4. Expiration Management

**Two-tier approach:**

**Lazy expiration (on access):**
```rust
pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
    match data.get(key) {
        Some(entry) if !entry.is_expired() => Ok(Some(entry.value)),
        _ => Ok(None),
    }
}
```

**Background cleanup:**
```rust
async fn expire_task(db: Arc<Db>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
        // Find expired keys
        // Delete them
    }
}
```

**Why both?**
- Lazy: Ensures correctness (expired key never returned)
- Background: Reclaims memory from keys never accessed again
- Combined: Balance between accuracy and memory efficiency

**Redis Comparison:**
- Real Redis uses probabilistic sampling (checks random subset)
- We check all keys (simpler, fine for moderate datasets)
- Production would need Redis's approach for millions of keys

### 5. Command Parsing and Execution

```rust
pub enum Command {
    Get { key: String },
    Set { key: String, value: Vec<u8>, px: Option<u64> },
    // ... other commands
}

impl Command {
    fn from_resp(resp: RespValue) -> Result<Self>;
    async fn execute(self, db: &Db) -> Result<RespValue>;
}
```

**Why separate parsing from execution?**
- Testable in isolation
- Clear error boundaries
- Easier to add new commands
- Can validate before executing

**Example: SET with expiration**
```rust
"SET" => {
    let key = array[1].as_str()?.to_string();
    let value = array[2].as_bytes()?.to_vec();

    // Parse optional EX/PX
    match array[3..] {
        ["EX", seconds] => ex = Some(parse(seconds)?),
        ["PX", millis] => px = Some(parse(millis)?),
    }

    Ok(Command::Set { key, value, px, ex })
}
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| GET | O(1) | Hash lookup + RwLock read |
| SET | O(1) | Hash insert + RwLock write |
| LPUSH/RPUSH | O(1) | VecDeque push |
| LRANGE | O(N) | N = range size |
| SADD | O(1) | HashSet insert |
| HSET | O(1) | Nested HashMap insert |
| Expiration check | O(1) | Instant comparison |

**Bottlenecks:**
1. **Lock contention**: All operations on single RwLock
2. **Memory**: All data in RAM (no persistence yet)
3. **CPU**: Single-threaded execution per connection

**Real Redis advantages:**
- Single-threaded event loop (no locks!)
- I/O multiplexing with epoll/kqueue
- Sophisticated memory optimization
- Persistence (RDB snapshots, AOF log)

## Comparison with TypeScript/Node.js

**TypeScript equivalent:**
```typescript
class RedisClone {
    private data: Map<string, Entry> = new Map();

    async get(key: string): Promise<Buffer | null> {
        const entry = this.data.get(key);
        if (!entry || entry.isExpired()) return null;
        return entry.value;
    }

    async set(key: string, value: Buffer): Promise<void> {
        this.data.set(key, { value, expiresAt: null });
    }
}
```

**Key differences:**
- **Rust**: Compile-time type safety, zero-cost abstractions
- **TypeScript**: Runtime overhead, easier async/await
- **Rust**: Explicit memory management (Arc, RwLock)
- **TypeScript**: GC handles memory (simpler, less predictable)
- **Rust**: Can achieve Redis-level performance
- **Node.js**: Great for I/O, but single-threaded JS limits CPU

## Production Improvements

### 1. Persistence

**RDB Snapshots:**
```rust
impl Db {
    pub async fn save_snapshot(&self, path: &Path) -> Result<()> {
        let data = self.data.read().await;
        let snapshot = Snapshot {
            version: 1,
            entries: data.clone(),
        };
        let encoded = bincode::serialize(&snapshot)?;
        tokio::fs::write(path, encoded).await?;
        Ok(())
    }
}
```

**AOF (Append-Only File):**
```rust
pub struct AofLog {
    file: Arc<Mutex<File>>,
}

impl AofLog {
    pub async fn log_command(&self, cmd: &Command) -> Result<()> {
        let mut file = self.file.lock().await;
        writeln!(file, "{}", cmd.serialize())?;
        file.sync_all()?;
        Ok(())
    }
}
```

### 2. Sharded Locks for Scalability

```rust
pub struct ShardedDb {
    shards: Vec<RwLock<HashMap<String, Entry>>>,
}

impl ShardedDb {
    fn shard_for_key(&self, key: &str) -> &RwLock<HashMap<String, Entry>> {
        let hash = calculate_hash(key);
        &self.shards[hash % self.shards.len()]
    }
}
```

### 3. Pub/Sub Support

```rust
pub struct PubSub {
    channels: Arc<RwLock<HashMap<String, broadcast::Sender<Vec<u8>>>>>,
}

impl Command {
    Publish { channel, message } => {
        let tx = pubsub.get_or_create_channel(&channel).await;
        tx.send(message)?;
        Ok(RespValue::Integer(tx.receiver_count() as i64))
    }
}
```

### 4. Pipelining Optimization

```rust
async fn handle_connection_pipelined(socket: TcpStream, db: Db) {
    // Parse all commands in buffer
    let commands = parse_all_commands(&buffer)?;

    // Execute all commands
    let responses = futures::future::join_all(
        commands.into_iter().map(|cmd| cmd.execute(&db))
    ).await;

    // Send all responses
    for response in responses {
        socket.write_all(&response.serialize()).await?;
    }
}
```

## Testing Strategy

```rust
#[tokio::test]
async fn test_concurrent_access() {
    let db = Db::new();

    let handles: Vec<_> = (0..100).map(|i| {
        let db = db.clone();
        tokio::spawn(async move {
            db.set(format!("key{}", i), format!("value{}", i).into_bytes())
                .await.unwrap();
        })
    }).collect();

    for handle in handles {
        handle.await.unwrap();
    }

    for i in 0..100 {
        let value = db.get(&format!("key{}", i)).await.unwrap();
        assert_eq!(value, Some(format!("value{}", i).into_bytes()));
    }
}
```

**Test Coverage:**
- Unit tests for RESP parsing
- Unit tests for each data type operation
- Integration tests for server
- Concurrency tests
- Expiration timing tests

## Common Pitfalls Avoided

### 1. Memory Leaks from Expired Keys

**Bad:**
```rust
// Never cleaned up!
if entry.is_expired() {
    return None; // But key still in HashMap
}
```

**Good:**
```rust
// Background task cleans up
async fn expire_task() {
    loop {
        // Remove expired entries
    }
}
```

### 2. Deadlocks from Lock Ordering

**Bad:**
```rust
// Can deadlock if two threads acquire in different order
let db1 = self.db1.write().await;
let db2 = self.db2.write().await;
```

**Good:**
```rust
// Single lock, no ordering issues
let data = self.data.write().await;
```

### 3. Type Confusion

**Bad:**
```rust
// TypeScript: Runtime error
const value = db.get("mylist");
value.split(","); // Error if it's actually a list!
```

**Good:**
```rust
// Rust: Compile-time safety + runtime check
match &entry.value {
    Value::String(s) => Ok(s.clone()),
    _ => Err(DbError::WrongType), // Clear error
}
```

## Usage Examples

**With redis-cli:**
```bash
$ cargo run &
$ redis-cli -p 6379

127.0.0.1:6379> SET mykey "Hello, Redis!"
OK
127.0.0.1:6379> GET mykey
"Hello, Redis!"
127.0.0.1:6379> EXPIRE mykey 10
(integer) 1
127.0.0.1:6379> TTL mykey
(integer) 7
127.0.0.1:6379> LPUSH mylist "world" "hello"
(integer) 2
127.0.0.1:6379> LRANGE mylist 0 -1
1) "hello"
2) "world"
```

**Programmatic usage:**
```rust
let db = Db::new();

// String operations
db.set("user:1".into(), b"Alice".to_vec()).await?;
let name = db.get("user:1").await?;

// List operations
db.rpush("queue", vec![b"job1".to_vec(), b"job2".to_vec()]).await?;
let job = db.lpop("queue", 1).await?;

// Set operations
db.sadd("tags", vec![b"rust".to_vec(), b"redis".to_vec()]).await?;
let is_member = db.sismember("tags", b"rust").await?;

// Hash operations
db.hset("user:1:profile", "email".into(), b"alice@example.com".to_vec()).await?;
let email = db.hget("user:1:profile", "email").await?;
```

## Conclusion

This Redis clone demonstrates:
- ✅ Full RESP protocol implementation
- ✅ Multiple data types (strings, lists, sets, hashes)
- ✅ Expiration with TTL support
- ✅ Type-safe operations with clear errors
- ✅ Concurrent access with RwLock
- ✅ Compatibility with redis-cli

**Key learnings:**
- Protocol design and parsing
- Multi-type storage with Rust enums
- Concurrency patterns for shared state
- Time-based expiration strategies
- Error handling for network protocols

Perfect foundation for understanding Redis internals and building custom data stores!
