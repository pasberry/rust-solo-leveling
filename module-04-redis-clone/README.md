# Module 04: Redis Clone

**Build an In-Memory Data Store with Protocol Support**

## Overview

Build a Redis-like in-memory data structure store with:
- RESP protocol support for Redis client compatibility
- Multiple data types (strings, lists, sets, hashes)
- Key expiration (TTL)
- Persistence options (RDB snapshots, AOF log)
- Concurrent client handling

**Duration**: 2-3 weeks (20-25 hours)

## What You'll Build

A production-ready in-memory store that can:
```bash
# Compatible with redis-cli
$ redis-cli -p 6379
127.0.0.1:6379> SET mykey "Hello"
OK
127.0.0.1:6379> GET mykey
"Hello"
127.0.0.1:6379> EXPIRE mykey 10
(integer) 1
127.0.0.1:6379> TTL mykey
(integer) 8
```

## Architecture

```
┌─────────────────────────────────────┐
│     Redis Protocol (RESP)           │
│   Parser & Serializer               │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│     Command Dispatcher              │
│   (GET, SET, DEL, EXPIRE, etc.)     │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│     Storage Engine                  │
│  ┌────────────────────────────────┐ │
│  │ String  │ List │ Set  │ Hash  │ │
│  │ Store   │Store │Store │ Store │ │
│  └────────────────────────────────┘ │
└─────────────────────────────────────┘
```

## Key Components

### 1. RESP Protocol Implementation

```rust
// RESP types
enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(Option<Vec<u8>>),
    Array(Option<Vec<RespValue>>),
}

impl RespValue {
    fn parse(bytes: &[u8]) -> Result<(Self, usize)> {
        match bytes[0] {
            b'+' => parse_simple_string(bytes),
            b'-' => parse_error(bytes),
            b':' => parse_integer(bytes),
            b'$' => parse_bulk_string(bytes),
            b'*' => parse_array(bytes),
            _ => Err(RespError::InvalidType),
        }
    }

    fn serialize(&self) -> Vec<u8> {
        match self {
            RespValue::SimpleString(s) => format!("+{}\r\n", s).into_bytes(),
            RespValue::Integer(i) => format!(":{}\r\n", i).into_bytes(),
            RespValue::BulkString(Some(bytes)) => {
                let mut result = format!("${}\r\n", bytes.len()).into_bytes();
                result.extend_from_slice(bytes);
                result.extend_from_slice(b"\r\n");
                result
            }
            // ... other types
        }
    }
}
```

### 2. Command Implementation

```rust
enum Command {
    Get { key: String },
    Set { key: String, value: Vec<u8>, px: Option<u64> },
    Del { keys: Vec<String> },
    Exists { keys: Vec<String> },
    Expire { key: String, seconds: u64 },
    // Lists
    LPush { key: String, values: Vec<Vec<u8>> },
    RPush { key: String, values: Vec<Vec<u8>> },
    LPop { key: String, count: Option<usize> },
    // Sets
    SAdd { key: String, members: Vec<Vec<u8>> },
    SMembers { key: String },
    // Hashes
    HSet { key: String, field: String, value: Vec<u8> },
    HGet { key: String, field: String },
}

impl Command {
    fn from_resp(resp: RespValue) -> Result<Self> {
        let array = resp.into_array()?;
        let cmd_name = array[0].as_str()?;

        match cmd_name.to_uppercase().as_str() {
            "GET" => Ok(Command::Get { key: array[1].as_str()?.into() }),
            "SET" => Ok(Command::Set {
                key: array[1].as_str()?.into(),
                value: array[2].as_bytes()?.into(),
                px: parse_px_option(&array[3..])?,
            }),
            // ... other commands
        }
    }

    async fn execute(&self, db: &Db) -> Result<RespValue> {
        match self {
            Command::Get { key } => {
                match db.get(key).await? {
                    Some(value) => Ok(RespValue::BulkString(Some(value))),
                    None => Ok(RespValue::BulkString(None)),
                }
            }
            Command::Set { key, value, px } => {
                db.set(key, value.clone()).await?;
                if let Some(ms) = px {
                    db.expire(key, Duration::from_millis(*ms)).await?;
                }
                Ok(RespValue::SimpleString("OK".into()))
            }
            // ... other commands
        }
    }
}
```

### 3. Data Storage

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{Instant, Duration};

#[derive(Clone)]
enum Value {
    String(Vec<u8>),
    List(Vec<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    Hash(HashMap<String, Vec<u8>>),
}

struct Entry {
    value: Value,
    expires_at: Option<Instant>,
}

pub struct Db {
    data: Arc<RwLock<HashMap<String, Entry>>>,
}

impl Db {
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => {
                match &entry.value {
                    Value::String(bytes) => Ok(Some(bytes.clone())),
                    _ => Err(DbError::WrongType),
                }
            }
            _ => Ok(None),
        }
    }

    pub async fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(key, Entry {
            value: Value::String(value),
            expires_at: None,
        });
        Ok(())
    }

    pub async fn expire(&self, key: &str, duration: Duration) -> Result<bool> {
        let mut data = self.data.write().await;

        if let Some(entry) = data.get_mut(key) {
            entry.expires_at = Some(Instant::now() + duration);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Instant::now() >= exp)
    }
}
```

### 4. Expiration Management

```rust
// Background task to clean up expired keys
async fn expire_task(db: Arc<Db>) {
    let mut interval = tokio::time::interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let expired_keys = {
            let data = db.data.read().await;
            data.iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(k, _)| k.clone())
                .collect::<Vec<_>>()
        };

        if !expired_keys.is_empty() {
            let mut data = db.data.write().await;
            for key in expired_keys {
                data.remove(&key);
            }
        }
    }
}
```

## Implementation Roadmap

### Phase 1: RESP Protocol (Days 1-2)
- Parse RESP messages
- Serialize responses
- Handle arrays, bulk strings, integers
- Unit tests for protocol

### Phase 2: Basic String Commands (Day 3)
- GET, SET, DEL
- Command parsing
- Response generation
- Integration tests

### Phase 3: Expiration (Days 4-5)
- EXPIRE, TTL commands
- Background expiration task
- Lazy expiration on access
- Tests for time-based behavior

### Phase 4: Lists (Day 6)
- LPUSH, RPUSH, LPOP, RPOP
- LRANGE, LLEN
- Type checking (return error if not a list)

### Phase 5: Sets and Hashes (Days 7-8)
- SADD, SMEMBERS, SISMEMBER
- HSET, HGET, HGETALL
- Type safety across operations

### Phase 6: Persistence (Days 9-10)
- RDB snapshots (background saves)
- AOF logging (append-only file)
- Recovery on startup

### Phase 7: Advanced Features (Days 11-12)
- Pipelining support
- Pub/Sub (bonus)
- Transactions (MULTI/EXEC)

## Testing Strategy

```rust
#[tokio::test]
async fn test_set_get() {
    let db = Db::new();

    db.set("key1".into(), b"value1".to_vec()).await.unwrap();
    let value = db.get("key1").await.unwrap();

    assert_eq!(value, Some(b"value1".to_vec()));
}

#[tokio::test]
async fn test_expiration() {
    let db = Db::new();

    db.set("key1".into(), b"value1".to_vec()).await.unwrap();
    db.expire("key1", Duration::from_millis(100)).await.unwrap();

    tokio::time::sleep(Duration::from_millis(150)).await;

    let value = db.get("key1").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_wrong_type_error() {
    let db = Db::new();

    // Set as string
    db.set("key1".into(), b"value".to_vec()).await.unwrap();

    // Try to use as list
    let result = db.lpush("key1".into(), vec![b"item".to_vec()]).await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), DbError::WrongType));
}
```

## Performance Targets

- **Throughput**: >100k ops/sec (GET/SET)
- **Latency**: <1ms p99 for single operations
- **Memory**: Efficient storage (minimal overhead)
- **Concurrency**: Handle 1000+ concurrent clients

## Success Criteria

- ✅ Compatible with `redis-cli`
- ✅ Supports core string, list, set, hash commands
- ✅ Expiration works correctly
- ✅ Handles concurrent clients without data races
- ✅ Persistence and recovery functional
- ✅ Performance meets targets

## Next Module

[Module 05: Message Queue →](../module-05-message-queue/)
