# Exercise 01: Bitcask-Style Key-Value Store

## Objective

Build a persistent key-value store using log-structured storage with compaction, similar to Bitcask.

**Estimated time:** 6-8 hours

## Requirements

### Core Features

1. **Basic Operations**
   - `set(key, value)` - Store key-value pair
   - `get(key)` - Retrieve value by key
   - `delete(key)` - Remove key
   - `exists(key)` - Check if key exists

2. **Log-Structured Storage**
   - Append-only log file for writes
   - In-memory index (HashMap) for lookups
   - Position-based retrieval

3. **Compaction**
   - Merge log files to reclaim space
   - Remove deleted/overwritten keys
   - Atomic switchover to new log

4. **Crash Recovery**
   - Rebuild index from log on startup
   - CRC checksums for integrity

### Technical Requirements

- Synchronous I/O (use std::fs)
- Binary encoding for efficiency
- Thread-safe API (Arc<Mutex>)
- Comprehensive error handling

## Architecture

```
┌──────────────────┐
│   KVStore API    │  set(), get(), delete()
└────────┬─────────┘
         │
┌────────▼─────────┐
│   In-Memory      │  HashMap<Key, LogPosition>
│   Index          │
└────────┬─────────┘
         │
┌────────▼─────────┐
│   Log Files      │  Append-only data files
│   (disk)         │  data.0, data.1, ...
└──────────────────┘
```

## Implementation Tasks

1. **Log Entry Format**
```rust
struct LogEntry {
    crc: u32,           // Checksum
    timestamp: u64,
    key_len: u32,
    value_len: u32,
    key: Vec<u8>,
    value: Vec<u8>,
}
```

2. **Index Entry**
```rust
struct IndexEntry {
    file_id: u32,
    offset: u64,
    value_len: u32,
    timestamp: u64,
}
```

3. **KVStore Structure**
```rust
pub struct KVStore {
    dir: PathBuf,
    index: HashMap<Vec<u8>, IndexEntry>,
    current_log: BufWriter<File>,
    current_file_id: u32,
    current_offset: u64,
}
```

## Testing Checklist

- [ ] Can set and get values
- [ ] Can delete keys
- [ ] Handles binary keys/values
- [ ] Survives restart (persistence)
- [ ] Compaction removes old data
- [ ] Handles corruption (CRC checks)
- [ ] Thread-safe operations
- [ ] Handles large values (>1MB)

## Bonus Challenges

1. **Hints**: Store file locations for faster compaction decisions
2. **Bloom Filters**: Reduce disk reads for non-existent keys
3. **WAL**: Add write-ahead logging for durability
4. **Range Scans**: Support iterating over key ranges
5. **TTL**: Auto-expire keys after timeout
