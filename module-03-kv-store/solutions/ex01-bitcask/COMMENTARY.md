# Bitcask Key-Value Store - Design Commentary

## Architecture

Classic Bitcask design with:
- **Append-only log files** - All writes go to end of current log
- **In-memory index** - HashMap<Key, FilePosition> for O(1) lookups
- **Compaction** - Merge logs to reclaim space from deleted/overwritten keys
- **CRC checksums** - Detect corruption

## Key Design Decisions

### 1. Log-Structured Storage

```rust
pub enum LogEntry {
    Set { key: Vec<u8>, value: Vec<u8> },
    Delete { key: Vec<u8> },
}
```

**Why append-only?**
- Sequential writes are fast (~100x faster than random)
- Simple crash recovery (read log to rebuild index)
- Enables fast compaction

**Format:**
```
[CRC:4][Len:4][Entry:Len]
```

### 2. In-Memory Index

```rust
struct IndexEntry {
    file_id: u32,
    offset: u64,
}
```

**Tradeoffs:**
- ✅ O(1) reads (no disk for lookups)
- ✅ Simple implementation
- ❌ Limited by RAM (can't store billions of keys)

**Production improvement:** Use hints file to speed up startup

### 3. Compaction Strategy

Trigger when `uncompacted_size > 1MB`:

1. Create new log file
2. Copy only live keys
3. Delete old logs
4. Update index atomically

**Why this threshold?**
- Balance between disk space and compaction overhead
- Configurable for different workloads

## Performance

| Operation | Complexity | Notes |
|-----------|-----------|-------|
| Set | O(1) | Append to log |
| Get | O(1) | Index lookup + one disk read |
| Delete | O(1) | Append tombstone |
| Compaction | O(n) | n = number of live keys |

**Bottlenecks:**
- RAM for index (1 pointer per key)
- Disk I/O for reads
- Compaction time (blocks writes)

## Comparison with Alternatives

**vs. B-Tree (LSM-tree, LevelDB):**
- Bitcask: Simple, predictable latency
- B-Tree: Better for range scans, less RAM

**vs. Hash table on disk:**
- Bitcask: Better write performance
- Hash table: Better space efficiency

## Production Enhancements

1. **Hints file** - Offsets for faster startup
2. **Bloom filters** - Reduce disk reads for missing keys
3. **Background compaction** - Don't block writes
4. **Replication** - Multiple nodes
5. **Partitioning** - Shard by key hash

## Testing

```bash
cargo test
```

All tests pass, covering:
- Basic set/get/delete
- Persistence across restarts
- Compaction correctness
- Corruption detection

## Conclusion

This Bitcask implementation demonstrates:
- ✅ Log-structured storage patterns
- ✅ Crash recovery from append-only logs
- ✅ Compaction to reclaim space
- ✅ CRC for data integrity

Perfect foundation for understanding systems like Riak, Cassandra hint system, and log-structured file systems.
