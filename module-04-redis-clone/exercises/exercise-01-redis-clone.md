# Exercise 01: Build a Redis Clone

## Objective

Build a Redis-compatible in-memory data store from scratch that supports:
- RESP (REdis Serialization Protocol) for compatibility with `redis-cli`
- Multiple data types: strings, lists, sets, and hashes
- Key expiration with TTL (Time To Live)
- Concurrent client connections

## Requirements

### Core Features

1. **RESP Protocol Support**
   - Parse RESP messages from clients
   - Serialize responses in RESP format
   - Handle all RESP types: Simple Strings, Errors, Integers, Bulk Strings, Arrays

2. **String Commands**
   - `GET key` - Get the value of a key
   - `SET key value [EX seconds] [PX milliseconds]` - Set a key with optional expiration
   - `DEL key [key ...]` - Delete one or more keys
   - `EXISTS key [key ...]` - Check if keys exist
   - `EXPIRE key seconds` - Set a timeout on a key
   - `TTL key` - Get the time to live for a key

3. **List Commands**
   - `LPUSH key element [element ...]` - Prepend elements to a list
   - `RPUSH key element [element ...]` - Append elements to a list
   - `LPOP key [count]` - Remove and return first elements
   - `RPOP key [count]` - Remove and return last elements
   - `LRANGE key start stop` - Get a range of elements
   - `LLEN key` - Get list length

4. **Set Commands**
   - `SADD key member [member ...]` - Add members to a set
   - `SMEMBERS key` - Get all members of a set
   - `SISMEMBER key member` - Check if member exists in set
   - `SCARD key` - Get set cardinality

5. **Hash Commands**
   - `HSET key field value` - Set a hash field
   - `HGET key field` - Get a hash field value
   - `HGETALL key` - Get all fields and values
   - `HLEN key` - Get number of fields

6. **Server Commands**
   - `PING [message]` - Test server connectivity
   - `ECHO message` - Echo a message

### Non-Functional Requirements

1. **Type Safety**: Return `WRONGTYPE` error when operating on wrong data type
2. **Concurrency**: Handle multiple concurrent clients safely
3. **Expiration**: Implement both lazy expiration (on access) and active expiration (background task)
4. **Memory Safety**: No memory leaks, proper cleanup of expired keys

## Architecture

Your solution should have these modules:

```
src/
├── main.rs          # Server entry point
├── server.rs        # TCP server and connection handling
├── resp.rs          # RESP protocol parser and serializer
├── command.rs       # Command parsing and execution
├── db.rs            # Database storage and operations
└── error.rs         # Error types
```

## Getting Started

1. Create a new Cargo project:
```bash
cargo new redis-clone
cd redis-clone
```

2. Add dependencies to `Cargo.toml`:
```toml
[dependencies]
tokio = { version = "1.35", features = ["full"] }
bytes = "1.5"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
thiserror = "1.0"
```

3. Start with the RESP protocol:
   - Implement parsing for all RESP types
   - Add comprehensive unit tests
   - Handle incomplete messages

4. Build the database:
   - Start with string operations (GET/SET)
   - Add expiration support
   - Implement other data types

5. Create the server:
   - Accept TCP connections
   - Parse RESP commands from socket
   - Execute commands and send responses

## Testing

### Unit Tests

Test each component in isolation:
```rust
#[test]
fn test_parse_bulk_string() {
    let data = b"$6\r\nfoobar\r\n";
    let value = RespValue::parse(data).unwrap();
    assert_eq!(value, RespValue::BulkString(Some(b"foobar".to_vec())));
}

#[tokio::test]
async fn test_set_get() {
    let db = Db::new();
    db.set("key".into(), b"value".to_vec()).await.unwrap();
    let value = db.get("key").await.unwrap();
    assert_eq!(value, Some(b"value".to_vec()));
}
```

### Integration Tests

Test with `redis-cli`:
```bash
# Terminal 1: Start your server
cargo run

# Terminal 2: Use redis-cli
redis-cli -p 6379
127.0.0.1:6379> SET mykey "Hello"
OK
127.0.0.1:6379> GET mykey
"Hello"
127.0.0.1:6379> LPUSH mylist "world" "hello"
(integer) 2
127.0.0.1:6379> LRANGE mylist 0 -1
1) "hello"
2) "world"
```

### Performance Tests

Your implementation should handle:
- 1000+ concurrent connections
- 10,000+ operations per second for simple GET/SET
- Proper memory cleanup for expired keys

## Success Criteria

- ✅ Compatible with `redis-cli`
- ✅ All listed commands work correctly
- ✅ Expiration works (keys disappear after TTL)
- ✅ Type errors are detected (e.g., LPUSH on a string key)
- ✅ Multiple clients can connect simultaneously
- ✅ No panics or crashes under normal operation
- ✅ Comprehensive test coverage

## Hints

1. **RESP Parsing**: Use a cursor to track position in the buffer
2. **Incomplete Messages**: Return an error and wait for more data
3. **Expiration**: Use `std::time::Instant` for monotonic timestamps
4. **Concurrency**: Use `Arc<RwLock<HashMap>>` for shared state
5. **Type Safety**: Use an enum for different value types

## Bonus Challenges

1. **Persistence**: Add RDB snapshots or AOF logging
2. **Pub/Sub**: Implement PUBLISH/SUBSCRIBE commands
3. **Transactions**: Add MULTI/EXEC support
4. **Benchmarking**: Compare performance with real Redis
5. **Replication**: Implement leader-follower replication

## Resources

- [Redis Protocol Specification](https://redis.io/docs/reference/protocol-spec/)
- [Redis Commands Reference](https://redis.io/commands/)
- [Tokio Async Book](https://tokio.rs/tokio/tutorial)
- Real Redis source code for reference

## Estimated Time

- **Phase 1** (RESP): 2-3 hours
- **Phase 2** (Basic commands): 2-3 hours
- **Phase 3** (Expiration): 1-2 hours
- **Phase 4** (Data types): 3-4 hours
- **Phase 5** (Testing): 1-2 hours

**Total**: 10-15 hours

Good luck! This is a challenging but rewarding project that will teach you protocol design, concurrent programming, and systems thinking.
