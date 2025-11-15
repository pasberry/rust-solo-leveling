# Exercise 01: Build a Persistent Message Queue

## Objective

Build a production-ready message queue system with:
- Persistent log-based storage
- At-least-once delivery guarantees
- Consumer acknowledgments
- Dead letter queue for failed messages
- Support for multiple concurrent consumers

## Requirements

### Core Features

1. **Message Structure**
   - Unique message ID
   - Payload (arbitrary bytes)
   - Metadata (key-value pairs)
   - Delivery attempt counter
   - Creation timestamp

2. **Queue Operations**
   - `publish(message)` - Add message to queue
   - `subscribe(consumer_id)` - Create a consumer
   - `receive()` - Get next message
   - `ack(message_id)` - Acknowledge successful processing
   - `nack(message_id)` - Negative acknowledge (retry)

3. **Persistence**
   - Messages written to disk immediately
   - Survive process crashes and restarts
   - Rebuild state from log on startup

4. **Delivery Guarantees**
   - At-least-once delivery
   - Messages redelivered if not acknowledged
   - Track delivery attempts
   - Move to dead letter queue after max retries

5. **Consumer Groups**
   - Multiple consumers can subscribe to same queue
   - Messages distributed across consumers
   - Each message delivered to one consumer per group

## Architecture

Your solution should have these modules:

```
src/
├── main.rs          # Demo application
├── queue.rs         # Queue and consumer implementation
├── message.rs       # Message types and status
├── log.rs           # Persistent log storage
└── error.rs         # Error types
```

## Getting Started

1. Create a new Cargo project:
```bash
cargo new message-queue
cd message-queue
```

2. Add dependencies:
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
bincode = "1.3"
uuid = { version = "1.7", features = ["v4", "serde"] }
thiserror = "1.0"
```

3. Start with the message format:
   - Define `Message` struct
   - Define `MessageStatus` enum
   - Implement serialization with serde

4. Build the log store:
   - Append-only file format
   - Length-prefixed entries
   - Recovery by scanning log

5. Implement the queue:
   - In-memory buffer for fast delivery
   - Persistent log for durability
   - Notify subscribers of new messages

6. Add acknowledgments:
   - Track message status transitions
   - Requeue failed messages
   - Move to DLQ after max retries

## Log Format

Use a simple length-prefixed format:

```
[Length: 4 bytes (u32)][LogEntry: Length bytes]
[Length: 4 bytes (u32)][LogEntry: Length bytes]
...
```

Where `LogEntry` is:
```rust
struct LogEntry {
    message: Message,
    status: MessageStatus,
    updated_at: u64,
}
```

## Message Status State Machine

```
Pending → Delivered → Acknowledged (success)
       ↓           ↓
       → Delivered → Failed → Pending (retry)
                           ↓
                           → DeadLettered (max retries)
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_publish_receive() {
    let queue = Queue::open("test", "./test-data").await.unwrap();
    let mut consumer = queue.subscribe("c1").await.unwrap();

    let msg = Message::new("test", b"hello".to_vec());
    queue.publish(msg).await.unwrap();

    let received = consumer.receive().await.unwrap().unwrap();
    assert_eq!(received.payload(), b"hello");
}

#[tokio::test]
async fn test_ack_prevents_redelivery() {
    let queue = Queue::open("test", "./test-data").await.unwrap();
    let mut consumer = queue.subscribe("c1").await.unwrap();

    queue.publish(Message::new("test", b"data".to_vec())).await.unwrap();

    let msg = consumer.receive().await.unwrap().unwrap();
    msg.ack().await.unwrap();

    // Restart queue
    drop(queue);
    let queue = Queue::open("test", "./test-data").await.unwrap();

    // Should have no pending messages
    assert_eq!(queue.depth().await, 0);
}
```

### Integration Tests

Test with multiple producers and consumers:

```rust
#[tokio::test]
async fn test_stress_test() {
    let queue = Arc::new(Queue::open("stress", "./test-data").await.unwrap());

    // 10 producers publishing 100 messages each
    let mut producers = vec![];
    for i in 0..10 {
        let q = Arc::clone(&queue);
        producers.push(tokio::spawn(async move {
            for j in 0..100 {
                let msg = Message::new("stress", format!("{}-{}", i, j).into_bytes());
                q.publish(msg).await.unwrap();
            }
        }));
    }

    // 5 consumers processing messages
    let received = Arc::new(Mutex::new(HashSet::new()));
    let mut consumers = vec![];

    for _ in 0..5 {
        let q = Arc::clone(&queue);
        let r = Arc::clone(&received);
        let mut consumer = q.subscribe("worker").await.unwrap();

        consumers.push(tokio::spawn(async move {
            while let Some(msg) = consumer.receive().await.unwrap() {
                r.lock().await.insert(String::from_utf8_lossy(msg.payload()).to_string());
                msg.ack().await.unwrap();
            }
        }));
    }

    // Wait for all producers
    for handle in producers {
        handle.await.unwrap();
    }

    // Wait a bit for consumers to catch up
    tokio::time::sleep(Duration::from_secs(2)).await;

    assert_eq!(received.lock().await.len(), 1000);
}
```

## Performance Targets

- **Throughput**: 10,000+ messages/second
- **Latency**: <10ms p99 for publish
- **Recovery**: <5 seconds for 100k messages
- **Memory**: <1MB per 10k pending messages

## Success Criteria

- ✅ Messages persist across restarts
- ✅ At-least-once delivery works correctly
- ✅ Acknowledged messages not redelivered
- ✅ Failed messages requeued
- ✅ Dead letter queue after max retries
- ✅ Multiple consumers receive different messages
- ✅ All tests pass

## Hints

1. **Persistence**: Use `BufWriter` for buffered writes, flush after each message
2. **Recovery**: Scan log file, track latest status per message ID
3. **Acknowledgment**: Use ownership to ensure messages are ack'd or nack'd
4. **Concurrency**: Use `Arc<Mutex>` for log, `Arc<RwLock>` for subscribers
5. **Testing**: Use `tempfile` crate for test data directories

## Bonus Challenges

1. **Compaction**: Remove acknowledged messages from log
2. **Message TTL**: Expire old messages automatically
3. **Priority Queues**: High-priority messages first
4. **Delayed Messages**: Schedule for future delivery
5. **Metrics**: Track throughput, latency, queue depth

## Estimated Time

- **Phase 1** (Message types): 1 hour
- **Phase 2** (Log store): 3-4 hours
- **Phase 3** (Queue + consumers): 3-4 hours
- **Phase 4** (Acknowledgments): 2-3 hours
- **Phase 5** (DLQ): 1-2 hours
- **Phase 6** (Testing): 2-3 hours

**Total**: 12-18 hours

## Resources

- [Kafka Documentation](https://kafka.apache.org/documentation/) - For comparison
- [RabbitMQ Tutorials](https://www.rabbitmq.com/getstarted.html) - Message queue patterns
- [Write-Ahead Logging](https://en.wikipedia.org/wiki/Write-ahead_logging) - Persistence technique

Good luck! This project will teach you about durability, delivery guarantees, and building reliable systems.
