# Module 03: Key-Value Store

**Build Your First Storage Engine**

## Overview

Build a persistent, log-structured key-value store from scratch. This is your first deep dive into storage systems, teaching you how databases work under the hood.

**Duration**: 1-3 weeks (15-20 hours)

## What You'll Build

A production-ready KV store library with:
- **Persistent storage** using append-only logs
- **In-memory index** for fast lookups
- **Compaction** to reclaim space from deleted/updated keys
- **Crash recovery** with write-ahead logging
- **CLI and embedded library** interfaces
- **Comprehensive test suite** including crash simulation

Think of it as a simplified [Bitcask](https://riak.com/assets/bitcask-intro.pdf) or the storage layer of Redis persistence.

## Learning Objectives

1. Understand **log-structured storage** and why it's fast
2. Master **file I/O** in Rust (sync and async)
3. Design **clean APIs** with traits and modules
4. Implement **compaction** strategies
5. Handle **crash scenarios** and ensure durability
6. Test storage systems effectively

## Architecture Overview

```
┌─────────────────────────────────────┐
│         KV Store API                │
│  get(key) put(key, value) delete()  │
└─────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
┌───────▼───────┐  ┌──────▼──────┐
│  In-Memory    │  │  Append-Only│
│  Hash Index   │  │  Data Log   │
│ key -> offset │  │   (file)    │
└───────────────┘  └─────────────┘
```

### Components

1. **Data Log**: Append-only file storing key-value entries
2. **Index**: HashMap mapping keys to file offsets
3. **Compaction**: Background process to merge and clean up logs
4. **Write-Ahead Log**: Ensures durability

### On-Disk Format

```
Entry:
┌────────┬────────┬────────┬─────────┬───────────┐
│ CRC32  │ KSize  │ VSize  │ Key     │ Value     │
│ (4B)   │ (4B)   │ (4B)   │ (var)   │ (var)     │
└────────┴────────┴────────┴─────────┴───────────┘
```

## Lecture Topics

### Lecture 01: Storage Engine Fundamentals

**Topics:**
- Why log-structured storage?
- Bitcask architecture
- Write amplification vs read amplification
- Comparing LSM trees, B-trees, and hash indexes
- Durability guarantees (fsync, write-ahead logging)

**Key Concepts:**
```rust
// The core API
trait KvStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()>;
    fn delete(&mut self, key: &[u8]) -> Result<()>;
}

// Log-structured means:
// 1. Writes are always appends (fast!)
// 2. Reads require index lookup
// 3. Updates don't modify in-place
// 4. Deleted entries marked as tombstones
```

### Lecture 02: File I/O and Serialization

**Topics:**
- std::fs vs memory-mapped files
- Buffering strategies (BufReader, BufWriter)
- Serialization formats (bincode, serde)
- checksums for data integrity (CRC32)
- File locking for concurrency

**Example:**
```rust
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write, Seek, SeekFrom};

struct DataLog {
    file: BufWriter<File>,
    offset: u64,
}

impl DataLog {
    fn append(&mut self, entry: &Entry) -> Result<u64> {
        let offset = self.offset;
        let bytes = entry.serialize()?;
        self.file.write_all(&bytes)?;
        self.file.flush()?;  // Ensure durability
        self.offset += bytes.len() as u64;
        Ok(offset)
    }

    fn read_at(&self, offset: u64) -> Result<Entry> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(offset))?;
        Entry::deserialize(&mut file)
    }
}
```

### Lecture 03: Compaction Strategies

**Topics:**
- Why compaction is necessary
- Stop-and-copy vs concurrent compaction
- Merge strategies
- Managing multiple log files
- Compaction triggers (size threshold, tombstone ratio)

**Algorithm:**
```rust
// Compaction merges old logs into new ones, keeping only latest values
fn compact(&mut self) -> Result<()> {
    // 1. Create new log file
    let new_log = DataLog::create("compacted.log")?;

    // 2. Iterate through index (which has latest offsets)
    for (key, offset) in &self.index {
        let value = self.read_value_at(offset)?;

        if value.is_some() {  // Skip tombstones
            let new_offset = new_log.append(key, value)?;
            self.index.insert(key.clone(), new_offset);
        }
    }

    // 3. Swap logs atomically
    std::fs::rename("compacted.log", "data.log")?;

    // 4. Delete old log files
    cleanup_old_logs()?;

    Ok(())
}
```

### Lecture 04: Crash Recovery and Testing

**Topics:**
- Building the index from log files on startup
- Write-ahead logging for atomic updates
- Testing crash scenarios
- Property-based testing for storage
- Benchmarking strategies

**Recovery:**
```rust
impl KvStore {
    fn recover(path: &Path) -> Result<Self> {
        let mut index = HashMap::new();
        let mut offset = 0;

        // Rebuild index by scanning log
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        while let Ok(entry) = Entry::deserialize(&mut reader) {
            if entry.is_tombstone() {
                index.remove(&entry.key);
            } else {
                index.insert(entry.key, offset);
            }
            offset += entry.size();
        }

        Ok(KvStore {
            log: DataLog::open(path)?,
            index,
        })
    }
}
```

## Project Structure

```
kv-store/
├── Cargo.toml
├── src/
│   ├── lib.rs           # Public API
│   ├── engine.rs        # Core KV engine
│   ├── log.rs           # Data log implementation
│   ├── entry.rs         # Log entry format
│   ├── index.rs         # In-memory index
│   ├── compaction.rs    # Compaction logic
│   └── error.rs         # Error types
├── benches/
│   └── benchmark.rs     # Performance benchmarks
├── tests/
│   ├── integration.rs   # Integration tests
│   └── crash_test.rs    # Crash recovery tests
└── examples/
    ├── cli.rs           # Command-line interface
    └── embedded.rs      # Library usage example
```

## Implementation Roadmap

### Phase 1: Basic In-Memory KV Store (Day 1)

```rust
// Start simple: HashMap wrapper
pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.map.get(key).cloned()
    }

    pub fn put(&mut self, key: String, value: String) {
        self.map.insert(key, value);
    }

    pub fn delete(&mut self, key: &str) {
        self.map.remove(key);
    }
}

// Tests
#[test]
fn test_basic_operations() {
    let mut store = KvStore::new();
    store.put("key1".into(), "value1".into());
    assert_eq!(store.get("key1"), Some("value1".into()));
    store.delete("key1");
    assert_eq!(store.get("key1"), None);
}
```

### Phase 2: Add Persistence (Days 2-3)

```rust
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct KvStore {
    index: HashMap<String, u64>,  // key -> offset
    log: BufWriter<File>,
    log_offset: u64,
}

impl KvStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(KvStore {
            index: HashMap::new(),
            log: BufWriter::new(file),
            log_offset: 0,
        })
    }

    pub fn put(&mut self, key: String, value: String) -> Result<()> {
        let entry = Entry::new(key.clone(), value);
        let bytes = entry.serialize()?;

        let offset = self.log_offset;
        self.log.write_all(&bytes)?;
        self.log.flush()?;

        self.index.insert(key, offset);
        self.log_offset += bytes.len() as u64;

        Ok(())
    }

    pub fn get(&self, key: &str) -> Result<Option<String>> {
        if let Some(&offset) = self.index.get(key) {
            let entry = self.read_entry_at(offset)?;
            Ok(Some(entry.value))
        } else {
            Ok(None)
        }
    }
}
```

### Phase 3: Add Deletion and Tombstones (Day 4)

```rust
#[derive(Serialize, Deserialize)]
enum EntryType {
    Put,
    Delete,
}

#[derive(Serialize, Deserialize)]
struct Entry {
    entry_type: EntryType,
    key: String,
    value: Option<String>,  // None for deletes
}

impl KvStore {
    pub fn delete(&mut self, key: &str) -> Result<()> {
        let tombstone = Entry {
            entry_type: EntryType::Delete,
            key: key.to_string(),
            value: None,
        };

        let bytes = tombstone.serialize()?;
        let offset = self.log_offset;

        self.log.write_all(&bytes)?;
        self.log.flush()?;

        self.index.remove(key);  // Remove from index
        self.log_offset += bytes.len() as u64;

        Ok(())
    }
}
```

### Phase 4: Implement Compaction (Days 5-6)

```rust
impl KvStore {
    pub fn compact(&mut self) -> Result<()> {
        // Create temporary compacted log
        let temp_path = self.path.with_extension("compact");
        let mut new_log = BufWriter::new(File::create(&temp_path)?);
        let mut new_index = HashMap::new();
        let mut new_offset = 0u64;

        // Write only live entries to new log
        for (key, old_offset) in &self.index {
            let entry = self.read_entry_at(*old_offset)?;

            let bytes = entry.serialize()?;
            new_log.write_all(&bytes)?;

            new_index.insert(key.clone(), new_offset);
            new_offset += bytes.len() as u64;
        }

        new_log.flush()?;
        drop(new_log);

        // Atomic swap
        std::fs::rename(&temp_path, &self.path)?;

        // Update in-memory state
        self.log = BufWriter::new(
            OpenOptions::new().append(true).open(&self.path)?
        );
        self.index = new_index;
        self.log_offset = new_offset;

        Ok(())
    }

    pub fn should_compact(&self) -> bool {
        // Compact if file is larger than threshold and has significant waste
        let file_size = self.log_offset;
        let live_data_size: u64 = self.index.len() as u64 * AVG_ENTRY_SIZE;

        file_size > COMPACT_THRESHOLD && file_size > live_data_size * 2
    }
}
```

### Phase 5: Add Crash Recovery (Day 7)

```rust
impl KvStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        if path.exists() {
            Self::recover(path)
        } else {
            Self::create(path)
        }
    }

    fn recover(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut index = HashMap::new();
        let mut offset = 0u64;

        // Rebuild index by scanning entire log
        loop {
            match Entry::deserialize(&mut reader) {
                Ok(entry) => {
                    match entry.entry_type {
                        EntryType::Put => {
                            index.insert(entry.key, offset);
                        }
                        EntryType::Delete => {
                            index.remove(&entry.key);
                        }
                    }
                    offset += entry.size() as u64;
                }
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => {
                    break;  // End of file
                }
                Err(e) => return Err(e.into()),
            }
        }

        let log = BufWriter::new(
            OpenOptions::new().append(true).open(path)?
        );

        Ok(KvStore {
            index,
            log,
            log_offset: offset,
        })
    }
}
```

### Phase 6: Add Checksums for Integrity (Day 8)

```rust
use crc::{Crc, CRC_32_ISCSI};

const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

#[derive(Serialize, Deserialize)]
struct Entry {
    checksum: u32,
    key: String,
    value: Option<String>,
}

impl Entry {
    fn new(key: String, value: String) -> Self {
        let mut entry = Entry {
            checksum: 0,
            key,
            value: Some(value),
        };
        entry.checksum = entry.calculate_checksum();
        entry
    }

    fn calculate_checksum(&self) -> u32 {
        let data = format!("{}{:?}", self.key, self.value);
        CASTAGNOLI.checksum(data.as_bytes())
    }

    fn verify(&self) -> bool {
        self.checksum == self.calculate_checksum()
    }

    fn deserialize(reader: &mut impl Read) -> Result<Self> {
        let entry: Entry = bincode::deserialize_from(reader)?;

        if !entry.verify() {
            return Err(Error::CorruptedData);
        }

        Ok(entry)
    }
}
```

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_put_get() {
        let temp = TempDir::new().unwrap();
        let mut store = KvStore::open(temp.path().join("db")).unwrap();

        store.put("key1".into(), "value1".into()).unwrap();
        assert_eq!(store.get("key1").unwrap(), Some("value1".into()));
    }

    #[test]
    fn test_persistence() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("db");

        {
            let mut store = KvStore::open(&path).unwrap();
            store.put("key1".into(), "value1".into()).unwrap();
        }

        // Reopen and verify data persisted
        let store = KvStore::open(&path).unwrap();
        assert_eq!(store.get("key1").unwrap(), Some("value1".into()));
    }

    #[test]
    fn test_compaction() {
        let temp = TempDir::new().unwrap();
        let mut store = KvStore::open(temp.path().join("db")).unwrap();

        // Write many updates to same key
        for i in 0..1000 {
            store.put("key".into(), format!("value{}", i)).unwrap();
        }

        let size_before = store.log_size();
        store.compact().unwrap();
        let size_after = store.log_size();

        assert!(size_after < size_before / 10);
        assert_eq!(store.get("key").unwrap(), Some("value999".into()));
    }
}
```

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn prop_get_returns_last_put(
        ops in prop::collection::vec(
            (any::<String>(), any::<String>()),
            0..100
        )
    ) {
        let temp = TempDir::new().unwrap();
        let mut store = KvStore::open(temp.path().join("db")).unwrap();

        let mut expected = HashMap::new();

        for (key, value) in ops {
            store.put(key.clone(), value.clone()).unwrap();
            expected.insert(key, value);
        }

        for (key, expected_value) in expected {
            assert_eq!(store.get(&key).unwrap(), Some(expected_value));
        }
    }
}
```

### Crash Simulation Tests

```rust
#[test]
fn test_crash_recovery() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("db");

    // Write some data
    {
        let mut store = KvStore::open(&path).unwrap();
        for i in 0..100 {
            store.put(format!("key{}", i), format!("value{}", i)).unwrap();
        }
        // Simulate crash - don't call close()
    }

    // Recover and verify all data present
    let store = KvStore::open(&path).unwrap();
    for i in 0..100 {
        assert_eq!(
            store.get(&format!("key{}", i)).unwrap(),
            Some(format!("value{}", i))
        );
    }
}
```

## CLI Interface

```rust
// examples/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Get { key: String },
    Put { key: String, value: String },
    Delete { key: String },
    Compact,
    Stats,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = KvStore::open("data.db")?;

    match cli.command {
        Commands::Get { key } => {
            match store.get(&key)? {
                Some(value) => println!("{}", value),
                None => println!("Key not found"),
            }
        }
        Commands::Put { key, value } => {
            store.put(key, value)?;
            println!("OK");
        }
        Commands::Delete { key } => {
            store.delete(&key)?;
            println!("OK");
        }
        Commands::Compact => {
            store.compact()?;
            println!("Compaction complete");
        }
        Commands::Stats => {
            println!("Keys: {}", store.len());
            println!("File size: {} bytes", store.file_size());
        }
    }

    Ok(())
}
```

## Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_writes(c: &mut Criterion) {
    c.bench_function("sequential writes", |b| {
        let temp = TempDir::new().unwrap();
        let mut store = KvStore::open(temp.path().join("db")).unwrap();

        b.iter(|| {
            store.put(
                black_box("key".into()),
                black_box("value".into())
            ).unwrap();
        });
    });
}

fn benchmark_reads(c: &mut Criterion) {
    let temp = TempDir::new().unwrap();
    let mut store = KvStore::open(temp.path().join("db")).unwrap();

    // Preload data
    for i in 0..10000 {
        store.put(format!("key{}", i), format!("value{}", i)).unwrap();
    }

    c.bench_function("random reads", |b| {
        b.iter(|| {
            let key = format!("key{}", black_box(5000));
            store.get(&key).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_writes, benchmark_reads);
criterion_main!(benches);
```

## Extensions

Once you have the basic store working:

1. **Multi-file logs**: Split into multiple files for better management
2. **Async I/O**: Make it work with Tokio for async applications
3. **Transactions**: Add begin/commit/rollback support
4. **Range queries**: Support scanning key ranges
5. **Snapshots**: Point-in-time snapshots for backups
6. **Replication**: Master-slave replication
7. **Compression**: Compress values before writing

## Success Criteria

- ✅ All data persists across restarts
- ✅ Handles crashes gracefully
- ✅ Compaction reclaims space
- ✅ Checksums detect corruption
- ✅ Performance: >10k writes/sec, >50k reads/sec
- ✅ Tests pass including property tests

## Next Steps

With a working KV store, you're ready for more complex storage systems!

Proceed to [Module 04: Redis Clone →](../module-04-redis-clone/) to build an in-memory data store with advanced features.

## Resources

- [Bitcask Paper](https://riak.com/assets/bitcask-intro.pdf)
- [Designing Data-Intensive Applications](https://dataintensive.net/)
- [PingCAP Talent Plan: KV](https://github.com/pingcap/talent-plan/tree/master/courses/rust)
