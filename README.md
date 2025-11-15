# Rust Systems Engineering Training Course

**From Experienced Engineer to Rust Systems Expert**

This is a comprehensive, project-based training program designed to take an experienced software engineer from Rust novice to expert-level systems programmer.

## About This Course

### Target Audience
- Senior engineers with 15+ years of experience
- Strong in TypeScript/JavaScript, Python, or similar high-level languages
- Comfortable with backend, cloud, and distributed systems concepts
- No need for beginner programming explanations
- Goal: Become truly strong in Rust and systems engineering

### Learning Philosophy
This course assumes you're a smart, experienced engineer. We skip generic beginner content and focus on:
- **Rust-specific mental models** (ownership, borrowing, lifetimes, traits)
- **Systems thinking** (performance, concurrency, reliability)
- **Real-world projects** that mirror production systems
- **Deep comparisons** to TypeScript, Python, and other familiar languages

## Course Structure

The course is organized into **12 modules**, each building on previous concepts:

### Phase 1: Foundation
- **[Module 01: Core Rust Fluency](./module-01-core-rust/)** - Ownership, traits, error handling, testing
- **[Module 02: Async + Networking](./module-02-async-networking/)** - Tokio, HTTP servers, concurrency

### Phase 2: Storage Systems
- **[Module 03: Key-Value Store](./module-03-kv-store/)** - Log-structured storage, file I/O
- **[Module 04: Redis Clone](./module-04-redis-clone/)** - In-memory store, protocol design, TTL
- **[Module 05: Messaging Queue](./module-05-message-queue/)** - Pub/sub, persistence, at-least-once delivery

### Phase 3: Distributed Systems
- **[Module 06: Distributed Cache](./module-06-distributed-cache/)** - Consistent hashing, cluster membership
- **[Module 07: S3-like Object Store](./module-07-object-store/)** - Object storage, streaming I/O, metadata

### Phase 4: Advanced Projects
- **[Module 08: SQLite-like Database](./module-08-embedded-db/)** - B-trees, page cache, query execution
- **[Module 09: Compiler/Interpreter](./module-09-compiler/)** - Lexer, parser, AST, VM or interpreter
- **[Module 10: Stock Trading System](./module-10-trading-system/)** - Order matching, low latency, capstone project

### Phase 5: Language Interop
- **[Module 11: Rust + Python](./module-11-python-interop/)** - PyO3, FFI, accelerating Python with Rust
- **[Module 12: Rust + TypeScript](./module-12-typescript-interop/)** - HTTP APIs, WASM, service design

## Module Format

Each module contains:

```
module-XX-name/
├── README.md              # Module overview and learning objectives
├── lectures/              # Written explanations and concepts
│   ├── 01-concept-a.md
│   ├── 02-concept-b.md
│   └── ...
├── exercises/             # Hands-on coding tasks
│   ├── ex01-task-name/
│   │   ├── README.md      # Exercise description and requirements
│   │   ├── Cargo.toml
│   │   └── src/           # Starter code (if applicable)
│   └── ...
└── solutions/             # Reference implementations with commentary
    ├── ex01-task-name/
    │   ├── COMMENTARY.md  # Explanation of solution and tradeoffs
    │   ├── Cargo.toml
    │   ├── src/
    │   └── tests/
    └── ...
```

## How to Use This Course

### 1. Sequential Learning (Recommended)
Work through modules in order. Each builds on concepts from previous modules.

```bash
# Start with Module 01
cd module-01-core-rust
cat README.md

# Read lectures
cat lectures/01-ownership-and-borrowing.md

# Do exercises
cd exercises/ex01-lru-cache
cargo test

# When stuck, check solutions
cd ../../solutions/ex01-lru-cache
cat COMMENTARY.md
```

### 2. Project-First Learning
If you learn best by building, jump to a project module (03-10) and reference earlier modules as needed.

### 3. Reference Mode
Use completed modules as reference implementations for your own projects.

## Prerequisites

### Required Tools
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version

# Recommended: Install rust-analyzer for your editor
```

### Recommended Setup
- **Editor**: VS Code with rust-analyzer, or any editor with LSP support
- **Tooling**: `cargo-watch`, `cargo-expand`, `cargo-flamegraph`
- **Testing**: Familiarity with `cargo test` and `cargo bench`

## Learning Path Estimates

Each module is designed for approximately:
- **Modules 01-02**: 1-2 weeks each (foundation)
- **Modules 03-07**: 1-3 weeks each (projects)
- **Modules 08-10**: 2-4 weeks each (complex projects)
- **Modules 11-12**: 1-2 weeks each (interop)

**Total**: 4-6 months at a steady pace (10-15 hours/week)

Adjust based on your learning speed and prior exposure to systems programming.

## Key Concepts Covered

### Rust-Specific
- Ownership, borrowing, and lifetimes (practical, not just theoretical)
- Traits and generics (trait objects, associated types, bounds)
- Enums and pattern matching (algebraic data types)
- Error handling (`Result`, `?`, custom error types)
- Async/await and futures (Tokio ecosystem)
- Unsafe Rust (when necessary, used responsibly)

### Systems Programming
- Memory layout and performance characteristics
- File I/O (buffered, unbuffered, async, sync)
- Network protocols (TCP, HTTP, custom protocols)
- Concurrency primitives (mutexes, channels, atomics)
- Serialization strategies (binary formats, zero-copy)
- Testing strategies (unit, integration, property-based)

### Distributed Systems
- Consistency models
- Replication and partitioning
- Failure handling and retry logic
- Observability (logging, metrics, tracing)

## Projects You'll Build

By the end of this course, you will have built **from scratch**:

### ✅ Complete with Working Code
1. **LRU Cache** - Core data structure with ownership patterns (Module 01)
2. **Config CLI Tool** - Serde, validation, error handling (Module 01)
3. **TCP Chat Server** - Multi-room chat, 1,500 lines (Module 02)
4. **REST API** - SQLx, validation, full CRUD, 1,100 lines (Module 02)
5. **WebSocket Notifications** - Real-time push, 700 lines (Module 02)
6. **Bitcask Key-Value Store** - Log-structured storage, 400 lines (Module 03)

### 📋 Comprehensive Roadmaps Available
7. **Redis Clone** - In-memory store with RESP protocol (Module 04)
8. **Message Queue** - Persistent queues, consumer groups (Module 05)
9. **Distributed Cache** - Consistent hashing, replication (Module 06)
10. **Object Store** - S3-like API with streaming uploads (Module 07)
11. **Embedded Database** - SQLite-like with B-tree index (Module 08)
12. **Compiler/Interpreter** - Lexer, parser, runtime (Module 09)
13. **Trading System** - Order matching engine, event bus (Module 10)
14. **Python Extensions** - High-performance Rust for Python (Module 11)
15. **TypeScript Services** - Rust backends for TS frontends (Module 12)

**Current Status**: 6 complete exercises with ~5,000 lines of production code + 8 detailed implementation roadmaps

## Engineering Practices

Throughout the course, we emphasize:

- **Idiomatic Rust**: Follow community conventions and leverage the type system
- **Testing**: Unit tests, integration tests, property-based testing
- **Documentation**: Clear APIs with doc comments and examples
- **Performance**: Benchmarking, profiling, optimization strategies
- **Observability**: Structured logging, metrics, distributed tracing
- **Error Handling**: Graceful degradation, meaningful error messages

## Comparisons to Other Languages

Each module includes discussions comparing Rust approaches to:
- **TypeScript/JavaScript**: async models, error handling, type systems
- **Python**: performance characteristics, memory management
- **Hack/PHP**: type safety, runtime characteristics
- **C/C++**: memory safety, modern tooling

## Getting Help

### When You're Stuck

1. **Read the error**: Rust compiler errors are excellent teachers
2. **Check the lectures**: Concepts are explained before exercises
3. **Review solutions**: Full reference implementations with commentary
4. **Consult docs**: [doc.rust-lang.org](https://doc.rust-lang.org)
5. **Ask the community**: [users.rust-lang.org](https://users.rust-lang.org)

### Common Pitfalls

Each module's solutions include a "Common Pitfalls" section addressing:
- Lifetime errors and how to fix them
- Borrowing issues and ownership patterns
- Performance traps and how to avoid them
- Architectural mistakes and better designs

## Beyond This Course

After completing this course, you'll be ready for:
- **Building production Rust systems** at scale
- **Contributing to open-source** Rust projects
- **Performance-critical services** and infrastructure
- **Systems programming roles** requiring Rust
- **Teaching others** Rust and systems concepts

### Recommended Next Steps
- **Read**: "The Rust Programming Language" (for reference)
- **Explore**: Async ecosystem (Tokio, async-std), web frameworks (Axum, Actix)
- **Build**: Your own projects combining concepts from multiple modules
- **Contribute**: To open-source Rust projects
- **Dive Deeper**: Unsafe Rust, embedded systems, WebAssembly

## Course Philosophy

This course is designed to:

1. **Respect your experience**: No "what is a variable" explanations
2. **Build systems knowledge**: Not just Rust syntax, but systems thinking
3. **Learn by building**: Every concept is applied in real projects
4. **Emphasize tradeoffs**: Understand *why*, not just *how*
5. **Prepare for production**: Real-world patterns and practices

## License

This course material is provided for educational purposes.

Code examples and solutions can be used freely in your own projects.

---

**Ready to level up your Rust skills?**

Start with [Module 01: Core Rust Fluency →](./module-01-core-rust/)
