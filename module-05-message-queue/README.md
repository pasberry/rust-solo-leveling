# Module 05: Message Queue

**Build a Persistent Message Queue System**

## Overview

Build a production-ready message queue with:
- Named queues with pub/sub patterns
- Persistent log-based storage
- At-least-once delivery guarantees
- Consumer groups for load distribution
- Dead letter queues for failed messages

**Duration**: 2-3 weeks (20-25 hours)

## What You'll Build

A message queue service that supports:

```rust
// Producer
let queue = MessageQueue::open("orders").await?;
queue.publish(Message::new("order-123", data)).await?;

// Consumer
let consumer = queue.subscribe("worker-1").await?;
while let Some(msg) = consumer.receive().await? {
    process(msg.payload).await?;
    msg.ack().await?;  // Acknowledge successful processing
}
```

## Architecture

```
┌─────────────────────────────────────────┐
│         Message Queue API               │
│   publish() subscribe() ack() nack()    │
└──────────────┬──────────────────────────┘
               │
    ┌──────────┴──────────┐
    │                     │
┌───▼──────┐      ┌───────▼──────┐
│ In-Memory│      │  Persistent  │
│  Buffer  │◄────►│  Log Store   │
└──────────┘      └──────────────┘
    │                     │
    └──────────┬──────────┘
               │
    ┌──────────▼──────────┐
    │  Consumer Groups    │
    │  Load Distribution  │
    └─────────────────────┘
```

## Key Components

### 1. Message Format

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub queue: String,
    pub payload: Vec<u8>,
    pub created_at: u64,
    pub attempts: u32,
    pub metadata: HashMap<String, String>,
}

impl Message {
    pub fn new(queue: impl Into<String>, payload: Vec<u8>) -> Self {
        Message {
            id: Uuid::new_v4().to_string(),
            queue: queue.into(),
            payload,
            created_at: current_timestamp(),
            attempts: 0,
            metadata: HashMap::new(),
        }
    }
}
```

### 2. Queue Implementation

```rust
pub struct Queue {
    name: String,
    log: Arc<RwLock<LogStore>>,
    buffer: Arc<Mutex<VecDeque<Message>>>,
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    config: QueueConfig,
}

impl Queue {
    pub async fn publish(&self, mut msg: Message) -> Result<()> {
        msg.queue = self.name.clone();

        // Write to persistent log
        self.log.write().await.append(&msg).await?;

        // Add to in-memory buffer
        self.buffer.lock().await.push_back(msg.clone());

        // Notify subscribers
        self.notify_subscribers(msg).await?;

        Ok(())
    }

    pub async fn subscribe(&self, consumer_id: String) -> Result<Consumer> {
        let (tx, rx) = mpsc::channel(self.config.buffer_size);

        let subscriber = Subscriber {
            id: consumer_id,
            sender: tx,
        };

        self.subscribers.write().await.push(subscriber);

        Ok(Consumer {
            queue: self.name.clone(),
            receiver: rx,
            log: Arc::clone(&self.log),
        })
    }
}
```

### 3. Consumer with Acknowledgment

```rust
pub struct Consumer {
    queue: String,
    receiver: mpsc::Receiver<Message>,
    log: Arc<RwLock<LogStore>>,
}

impl Consumer {
    pub async fn receive(&mut self) -> Result<Option<AckMessage>> {
        match self.receiver.recv().await {
            Some(msg) => Ok(Some(AckMessage {
                message: msg,
                log: Arc::clone(&self.log),
            })),
            None => Ok(None),
        }
    }
}

pub struct AckMessage {
    message: Message,
    log: Arc<RwLock<LogStore>>,
}

impl AckMessage {
    pub async fn ack(self) -> Result<()> {
        // Mark as acknowledged in log
        self.log.write().await.mark_acked(&self.message.id).await
    }

    pub async fn nack(self) -> Result<()> {
        // Requeue or move to DLQ
        self.log.write().await.mark_failed(&self.message.id).await
    }

    pub fn payload(&self) -> &[u8] {
        &self.message.payload
    }
}
```

### 4. Persistent Log Store

```rust
pub struct LogStore {
    path: PathBuf,
    file: BufWriter<File>,
    index: HashMap<String, u64>,  // msg_id -> offset
    offset: u64,
}

impl LogStore {
    pub async fn append(&mut self, msg: &Message) -> Result<()> {
        let entry = LogEntry {
            msg_id: msg.id.clone(),
            status: EntryStatus::Pending,
            data: bincode::serialize(msg)?,
        };

        let bytes = bincode::serialize(&entry)?;
        let offset = self.offset;

        self.file.write_all(&bytes)?;
        self.file.flush()?;

        self.index.insert(msg.id.clone(), offset);
        self.offset += bytes.len() as u64;

        Ok(())
    }

    pub async fn mark_acked(&mut self, msg_id: &str) -> Result<()> {
        // Write tombstone or update entry status
        // Implementation depends on log format choice
        Ok(())
    }

    pub async fn recover(&mut self) -> Result<Vec<Message>> {
        // Scan log, rebuild index, return unacknowledged messages
        let mut pending = Vec::new();

        // Read log entries
        // For each entry:
        //   - If pending: add to pending vec
        //   - If acked: skip
        //   - Update index

        Ok(pending)
    }
}
```

## Implementation Roadmap

### Phase 1: In-Memory Queue (Days 1-2)
**Goal**: Basic pub/sub without persistence

```rust
pub struct InMemoryQueue {
    messages: Arc<Mutex<VecDeque<Message>>>,
    subscribers: Arc<Mutex<Vec<mpsc::Sender<Message>>>>,
}

// Implement:
// - publish() adds to queue
// - subscribe() creates new subscriber
// - notify_subscribers() sends to all subscribers
```

**Tests**:
- Single producer, single consumer
- Multiple consumers receive different messages
- Messages delivered in order

### Phase 2: Add Persistence (Days 3-4)
**Goal**: Messages survive restarts

```rust
// Add LogStore
// - Append messages to file
// - Build index on startup
// - Replay unacked messages
```

**Tests**:
- Messages persist across restarts
- Index rebuilt correctly
- Crash recovery works

### Phase 3: Acknowledgments (Days 5-6)
**Goal**: At-least-once delivery

```rust
// Implement:
// - ack() marks message as processed
// - nack() requeues message
// - Timeout for unacked messages
```

**Tests**:
- Acked messages not redelivered
- Nacked messages requeued
- Timeout triggers redelivery

### Phase 4: Consumer Groups (Days 7-8)
**Goal**: Load balancing across consumers

```rust
pub struct ConsumerGroup {
    name: String,
    consumers: Vec<Consumer>,
    strategy: LoadBalanceStrategy,
}

enum LoadBalanceStrategy {
    RoundRobin,
    LeastBusy,
    Random,
}
```

**Tests**:
- Messages distributed across group
- One message processed once per group
- Consumer failures handled

### Phase 5: Dead Letter Queue (Days 9-10)
**Goal**: Handle failed messages

```rust
pub struct DeadLetterQueue {
    queue: Queue,
    max_retries: u32,
}

impl DeadLetterQueue {
    async fn maybe_move_to_dlq(&self, msg: &Message) -> Result<()> {
        if msg.attempts >= self.max_retries {
            self.queue.publish(msg.clone()).await?;
        }
        Ok(())
    }
}
```

**Tests**:
- Messages moved after max retries
- DLQ queryable for inspection
- DLQ messages can be replayed

### Phase 6: Advanced Features (Days 11-12)
- Message priority
- Delayed messages (schedule for future)
- Message filtering/routing
- Metrics and monitoring

## Delivery Guarantees

### At-Least-Once
**What we implement**: Messages delivered one or more times

```rust
// How it works:
// 1. Publish writes to log
// 2. Consumer receives message
// 3. Consumer processes message
// 4. Consumer sends ACK
// 5. ACK written to log

// If crash between 2-4: message redelivered
```

### At-Most-Once (Bonus)
**Alternative**: Messages delivered zero or one time

```rust
// How it would work:
// 1. Publish writes to log
// 2. Mark as delivered immediately
// 3. Consumer receives message
// If crash: message lost, but never duplicated
```

### Exactly-Once (Hard)
**Advanced**: Requires distributed transactions or idempotency

## Testing Strategy

### Unit Tests

```rust
#[tokio::test]
async fn test_publish_receive() {
    let queue = Queue::new("test").await.unwrap();
    let mut consumer = queue.subscribe("c1").await.unwrap();

    queue.publish(Message::new("test", b"data".to_vec())).await.unwrap();

    let msg = consumer.receive().await.unwrap().unwrap();
    assert_eq!(msg.payload(), b"data");
}

#[tokio::test]
async fn test_ack_prevents_redelivery() {
    let queue = Queue::new("test").await.unwrap();
    let mut c1 = queue.subscribe("c1").await.unwrap();

    queue.publish(Message::new("test", b"data".to_vec())).await.unwrap();

    let msg = c1.receive().await.unwrap().unwrap();
    msg.ack().await.unwrap();

    // Restart queue
    drop(queue);
    let queue = Queue::open("test").await.unwrap();
    let mut c2 = queue.subscribe("c2").await.unwrap();

    // Should not receive message
    tokio::select! {
        _ = c2.receive() => panic!("Should not receive acked message"),
        _ = tokio::time::sleep(Duration::from_millis(100)) => {}
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_producer_consumer_stress() {
    let queue = Arc::new(Queue::new("stress").await.unwrap());
    let mut handles = vec![];

    // Spawn 10 producers
    for i in 0..10 {
        let q = Arc::clone(&queue);
        handles.push(tokio::spawn(async move {
            for j in 0..100 {
                q.publish(Message::new("stress", format!("{}-{}", i, j).into_bytes()))
                    .await
                    .unwrap();
            }
        }));
    }

    // Spawn 5 consumers
    let received = Arc::new(Mutex::new(HashSet::new()));
    for _ in 0..5 {
        let q = Arc::clone(&queue);
        let r = Arc::clone(&received);
        let mut consumer = q.subscribe("worker").await.unwrap();

        handles.push(tokio::spawn(async move {
            while let Some(msg) = consumer.receive().await.unwrap() {
                r.lock().await.insert(String::from_utf8_lossy(msg.payload()).to_string());
                msg.ack().await.unwrap();
            }
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(received.lock().await.len(), 1000);
}
```

## Performance Targets

- **Throughput**: >10k messages/sec
- **Latency**: <10ms p99
- **Persistence**: <1ms to write
- **Recovery**: <5 seconds for 1M messages

## Comparing to Other Systems

### RabbitMQ
- **Pros**: Feature-rich, battle-tested
- **Cons**: Complex setup, heavyweight
- **Our version**: Simpler, embedded-friendly

### Kafka
- **Pros**: Extremely high throughput, persistent
- **Cons**: Complex, requires ZooKeeper/KRaft
- **Our version**: Simpler, good for moderate scale

### Redis Streams
- **Pros**: Fast, simple
- **Cons**: In-memory primarily
- **Our version**: Similar semantics, persistent

## Success Criteria

- ✅ Messages persist across restarts
- ✅ At-least-once delivery guaranteed
- ✅ Consumer groups distribute load
- ✅ Dead letter queue handles failures
- ✅ Performance meets targets
- ✅ Crash recovery works correctly

## Extensions

1. **Message TTL**: Expire old messages
2. **Priority queues**: High/low priority
3. **Scheduled messages**: Deliver at future time
4. **Message routing**: Topic-based routing
5. **Replication**: Multi-node deployment
6. **Monitoring**: Metrics dashboard

## Next Module

[Module 06: Distributed Cache →](../module-06-distributed-cache/)
