# Message Queue - Design Commentary

## Architecture Overview

This message queue system implements a persistent, at-least-once delivery queue with:
- **Persistent Log Storage** - Messages survive crashes and restarts
- **At-Least-Once Delivery** - Messages guaranteed to be delivered at least once
- **Consumer Groups** - Multiple consumers can subscribe and receive messages
- **Dead Letter Queue** - Failed messages after max retries are tracked
- **Acknowledgments** - Consumers must ack messages for completion

## Key Design Decisions

### 1. Log-Structured Storage

```rust
pub struct LogStore {
    path: PathBuf,
    writer: BufWriter<File>,
    index: HashMap<String, u64>,  // msg_id -> offset
    offset: u64,
}
```

**Why log-structured?**
- Append-only writes are fast (sequential I/O)
- Simple crash recovery (just scan the log)
- Easy to implement at-least-once semantics
- Natural audit trail of all message states

**Log Format:**
```
[Length: 4 bytes][LogEntry: Length bytes]
[Length: 4 bytes][LogEntry: Length bytes]
...
```

**LogEntry contains:**
- Message (ID, queue, payload, metadata)
- Status (Pending, Delivered, Acknowledged, Failed, DeadLettered)
- Timestamp

**State Transitions:**
```
Pending → Delivered → Acknowledged (happy path)
Pending → Delivered → Failed → Pending (retry)
Pending → Delivered → Failed → DeadLettered (max retries)
```

### 2. At-Least-Once Delivery

```rust
// Producer
queue.publish(msg).await?;  // Writes to log with Pending status

// Consumer
let msg = consumer.receive().await?;  // Writes Delivered status
// ... process ...
msg.ack().await?;  // Writes Acknowledged status
```

**How it works:**
1. Message published → written to log as `Pending`
2. Consumer receives → status updated to `Delivered`
3. Consumer processes → calls `ack()` → status updated to `Acknowledged`

**Failure scenarios:**
- **Crash after (1)**: Message recovered as `Pending`, redelivered
- **Crash after (2)**: Message recovered as `Delivered`, redelivered
- **Crash after (3)**: Message is `Acknowledged`, not redelivered

**Why not exactly-once?**
- Exactly-once requires distributed transactions or idempotent consumers
- At-least-once is simpler and sufficient for most use cases
- Consumers should be designed to handle duplicate messages (idempotency)

### 3. Persistent Index

```rust
index: HashMap<String, u64>  // message_id -> file_offset
```

**Why maintain an index?**
- Fast lookups by message ID
- Avoid scanning entire log for status updates
- Rebuilt on recovery by scanning log once

**Trade-off:**
- Memory usage: O(number of unacked messages)
- Recovery time: O(log size) - must scan to rebuild index
- Production improvement: Persist index snapshots to speed up recovery

### 4. Hybrid Storage: Log + In-Memory Buffer

```rust
pub struct Queue {
    log: Arc<Mutex<LogStore>>,       // Persistent
    buffer: Arc<Mutex<VecDeque<Message>>>,  // In-memory for fast delivery
    // ...
}
```

**Why both?**
- **Log**: Durability (survives crashes)
- **Buffer**: Performance (fast message delivery)
- **On publish**: Write to both log and buffer
- **On recovery**: Reload buffer from log

**Benefits:**
- Fast delivery without disk reads
- Guaranteed persistence
- Simple recovery mechanism

### 5. Consumer Acknowledgment Pattern

```rust
pub struct AckMessage {
    message: Message,
    log: Arc<Mutex<LogStore>>,
    max_retries: u32,
}

impl AckMessage {
    pub async fn ack(self) -> Result<()> {
        // Mark as acknowledged
    }

    pub async fn nack(mut self) -> Result<()> {
        // Increment attempts, requeue or DLQ
    }
}
```

**Why separate AckMessage type?**
- Forces consumer to explicitly acknowledge
- Prevents accidental message loss
- Can't forget to ack (compiler ensures ownership transfer)
- Clean API: either `ack()` or `nack()`

**Ownership semantics:**
```rust
let msg = consumer.receive().await?;
// msg owns the message, must either:
msg.ack().await?;   // consumes msg
// OR
msg.nack().await?;  // consumes msg
// Can't use msg after ack/nack
```

### 6. Dead Letter Queue Implementation

```rust
pub async fn nack(mut self) -> Result<()> {
    self.message.increment_attempts();

    if self.message.attempts >= self.max_retries {
        // Move to DLQ
        log.append(&self.message, MessageStatus::DeadLettered)?;
    } else {
        // Retry
        log.mark_failed(&self.message.id)?;
    }
}
```

**DLQ strategy:**
- Track max retry attempts in message
- After max retries, mark as `DeadLettered`
- Dead lettered messages not redelivered
- Can be queried separately for inspection

**Production improvements:**
- Separate DLQ queue for analysis
- Retry with exponential backoff
- Allow manual replay from DLQ

### 7. Log Compaction

```rust
pub fn compact(&mut self) -> Result<()> {
    // 1. Scan log, collect latest state per message
    // 2. Write only Pending/Failed messages to new file
    // 3. Replace old file with new file
    // 4. Rebuild index
}
```

**Why compact?**
- Log grows with every status update
- Acknowledged messages waste space
- Compaction removes historical entries

**When to compact?**
- Periodic background task
- When log exceeds size threshold
- On idle periods

**Trade-offs:**
- Compaction pauses writes (exclusive lock)
- Requires disk space for temporary file
- Faster recovery after compaction

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Publish | O(1) | Append to log + buffer |
| Receive | O(1) | Pop from buffer |
| Ack | O(1) | Append to log |
| Recovery | O(N) | N = log entries |
| Compaction | O(M) | M = unique messages |

**Bottlenecks:**
1. **Disk I/O**: Every publish writes to disk
2. **Lock contention**: Single lock for log
3. **Memory**: All pending messages in buffer

**Optimization strategies:**
- Batch writes to log (trade latency for throughput)
- Use async I/O (tokio::fs)
- Shard queues by topic
- Use ring buffer instead of VecDeque

## Comparison with Other Systems

### vs. RabbitMQ

**RabbitMQ:**
- ✅ Feature-rich (routing, exchanges, clustering)
- ✅ Battle-tested at scale
- ❌ Complex setup (Erlang runtime)
- ❌ Heavy resource usage

**Our Queue:**
- ✅ Simple, embeddable
- ✅ Low resource footprint
- ❌ Fewer features
- ❌ Single node (no clustering)

### vs. Kafka

**Kafka:**
- ✅ Extremely high throughput (millions/sec)
- ✅ Distributed, replicated
- ❌ Complex (ZooKeeper/KRaft)
- ❌ Designed for streaming, not queueing

**Our Queue:**
- ✅ Simpler model
- ✅ Good for moderate scale (10k/sec)
- ❌ Single node
- ❌ Lower throughput

### vs. Redis Streams

**Redis Streams:**
- ✅ Very fast (in-memory)
- ✅ Consumer groups built-in
- ❌ Primarily in-memory (persistence is secondary)
- ❌ Limited by RAM

**Our Queue:**
- ✅ Disk-persistent by default
- ✅ Unlimited size (disk-limited)
- ❌ Slower than in-memory
- ✅ Similar acknowledgment semantics

## Comparison with TypeScript/Node.js

**TypeScript equivalent (using SQLite):**
```typescript
class MessageQueue {
    private db: Database;

    async publish(message: Message): Promise<void> {
        await this.db.run(
            'INSERT INTO messages (id, queue, payload, status) VALUES (?, ?, ?, ?)',
            [message.id, message.queue, message.payload, 'pending']
        );
    }

    async receive(consumerId: string): Promise<Message | null> {
        return await this.db.get(
            'SELECT * FROM messages WHERE status = ? LIMIT 1',
            ['pending']
        );
    }

    async ack(messageId: string): Promise<void> {
        await this.db.run(
            'UPDATE messages SET status = ? WHERE id = ?',
            ['acknowledged', messageId]
        );
    }
}
```

**Differences:**
- **Rust**: Log-structured (append-only), zero-copy
- **TypeScript**: SQL database (update in place)
- **Rust**: Type-safe status tracking with enums
- **TypeScript**: String-based status (runtime errors possible)
- **Rust**: Async with Tokio (efficient concurrency)
- **TypeScript**: Promises (simpler but less control)

## Production Improvements

### 1. Async I/O

```rust
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

pub async fn append(&mut self, entry: &LogEntry) -> Result<()> {
    let data = bincode::serialize(entry)?;
    self.file.write_all(&data).await?;
    self.file.flush().await?;
    Ok(())
}
```

### 2. Batching Writes

```rust
pub struct BatchedLog {
    buffer: Vec<LogEntry>,
    flush_size: usize,
}

impl BatchedLog {
    pub async fn append(&mut self, entry: LogEntry) -> Result<()> {
        self.buffer.push(entry);

        if self.buffer.len() >= self.flush_size {
            self.flush().await?;
        }

        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        for entry in &self.buffer {
            // Write all entries
        }
        self.buffer.clear();
        Ok(())
    }
}
```

**Trade-off:** Latency vs. throughput

### 3. Consumer Groups with Load Balancing

```rust
pub enum LoadBalanceStrategy {
    RoundRobin,
    LeastBusy,
    Random,
}

pub struct ConsumerGroup {
    consumers: Vec<Consumer>,
    strategy: LoadBalanceStrategy,
    round_robin_index: AtomicUsize,
}

impl ConsumerGroup {
    fn next_consumer(&self) -> &Consumer {
        match self.strategy {
            LoadBalanceStrategy::RoundRobin => {
                let idx = self.round_robin_index.fetch_add(1, Ordering::SeqCst);
                &self.consumers[idx % self.consumers.len()]
            }
            // ... other strategies
        }
    }
}
```

### 4. Message TTL (Time To Live)

```rust
pub struct Message {
    // ... existing fields
    ttl: Option<Duration>,
    expires_at: Option<u64>,
}

impl Queue {
    pub async fn publish(&self, mut msg: Message) -> Result<()> {
        if let Some(ttl) = msg.ttl {
            msg.expires_at = Some(current_timestamp() + ttl.as_millis() as u64);
        }

        // ... rest of publish logic
    }

    async fn is_expired(&self, msg: &Message) -> bool {
        msg.expires_at.map_or(false, |exp| current_timestamp() > exp)
    }
}
```

### 5. Metrics and Observability

```rust
pub struct QueueMetrics {
    pub messages_published: AtomicU64,
    pub messages_delivered: AtomicU64,
    pub messages_acked: AtomicU64,
    pub messages_failed: AtomicU64,
}

impl Queue {
    pub async fn publish(&self, msg: Message) -> Result<()> {
        // ... existing logic
        self.metrics.messages_published.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
}
```

## Testing Strategy

**Unit tests:**
- Message serialization/deserialization
- Log append and recovery
- Index management
- Status transitions

**Integration tests:**
- Publish and receive
- Acknowledgment prevents redelivery
- Nack triggers redelivery
- Max retries moves to DLQ
- Persistence across restarts

**Stress tests:**
- Multiple producers, multiple consumers
- High message volume
- Crash and recovery
- Concurrent acknowledgments

## Common Pitfalls Avoided

### 1. Forgetting to Flush

**Bad:**
```rust
file.write_all(&data)?;
// If crash here, data may be lost!
```

**Good:**
```rust
file.write_all(&data)?;
file.flush()?;  // Ensure data is on disk
```

### 2. Not Handling Partial Writes

**Bad:**
```rust
// What if only part of the entry is written?
file.write_all(&serialized)?;
```

**Good:**
```rust
// Length-prefixed format allows detecting incomplete entries
file.write_all(&len.to_le_bytes())?;
file.write_all(&data)?;
// On recovery, if len doesn't match, skip corrupted entry
```

### 3. Race Conditions in Acknowledgment

**Bad:**
```rust
// Two threads could ack the same message
if messages.contains(&msg_id) {
    messages.remove(&msg_id);  // Race!
}
```

**Good:**
```rust
// Mutex ensures atomicity
let mut log = self.log.lock().await;
log.mark_acked(&msg_id)?;
```

## Usage Examples

**Basic pub/sub:**
```rust
let queue = Queue::open("orders", "./data").await?;

// Producer
queue.publish(Message::new("orders", b"order-123".to_vec())).await?;

// Consumer
let mut consumer = queue.subscribe("worker-1").await?;
if let Some(msg) = consumer.receive().await? {
    println!("Received: {:?}", msg.payload());
    msg.ack().await?;
}
```

**With error handling:**
```rust
let mut consumer = queue.subscribe("worker-1").await?;

while let Some(msg) = consumer.receive().await? {
    match process_order(msg.payload()).await {
        Ok(_) => {
            msg.ack().await?;
        }
        Err(e) => {
            eprintln!("Processing failed: {}", e);
            msg.nack().await?;  // Retry
        }
    }
}
```

## Conclusion

This message queue demonstrates:
- ✅ Persistent log-structured storage
- ✅ At-least-once delivery guarantees
- ✅ Consumer acknowledgment pattern
- ✅ Dead letter queue for failed messages
- ✅ Crash recovery and persistence

**Key learnings:**
- Log-structured storage patterns
- Delivery guarantees and trade-offs
- Consumer acknowledgment semantics
- State machine for message lifecycle
- Durability through write-ahead logging

Perfect foundation for understanding Kafka, RabbitMQ, and building reliable distributed systems!
