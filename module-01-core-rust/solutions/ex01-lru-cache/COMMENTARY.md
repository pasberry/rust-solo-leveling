# LRU Cache Solution Commentary

## Overview

This solution implements an LRU cache using a **HashMap + VecDeque** approach. While not the most performant possible implementation (a doubly-linked list would be O(1) for all operations), it's significantly simpler to implement correctly in safe Rust and performs well for most use cases.

## Design Decisions

### Data Structure Choice: HashMap + VecDeque

```rust
pub struct LRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,
}
```

**Why this approach?**

1. **HashMap**: O(1) average case for key lookup
2. **VecDeque**: Tracks access order, supports efficient front/back operations
3. **Simplicity**: Much easier to implement correctly than a doubly-linked list in safe Rust

**Tradeoffs**:
- **Pros**: Simple, safe, correct, good enough performance for most cases
- **Cons**: O(n) for reordering (need to find and move items in VecDeque)
- **Alternative**: Index-based doubly-linked list for O(1) everything, but significantly more complex

### Generic Constraints

```rust
where
    K: Eq + Hash + Clone,
    V: Clone,
```

**Why these bounds?**
- `Eq + Hash`: Required for HashMap keys
- `Clone`: We need to clone keys when reordering (VecDeque stores keys, HashMap stores both)
  - **Note**: In a more advanced implementation, we could use `Rc<K>` to avoid cloning

**This is a key design decision**: Simpler API with `Clone` requirement vs more complex implementation without it.

## Implementation Walkthrough

### Creating the Cache

```rust
pub fn new(capacity: usize) -> Self {
    assert!(capacity > 0, "Capacity must be greater than 0");
    LRUCache {
        capacity,
        map: HashMap::with_capacity(capacity),
        order: VecDeque::with_capacity(capacity),
    }
}
```

**Key points**:
- Assert capacity > 0 (undefined behavior for 0-capacity cache)
- Pre-allocate capacity to avoid reallocations
- Use `with_capacity` for both HashMap and VecDeque

### Getting a Value

```rust
pub fn get(&mut self, key: &K) -> Option<V> {
    if !self.map.contains_key(key) {
        return None;
    }

    // Update recency: move to back
    self.update_recency(key);

    self.map.get(key).cloned()
}
```

**Key points**:
1. **Check existence first**: Early return if not found
2. **Update recency**: Move accessed key to back (most recent)
3. **Clone the value**: Return owned value, not reference
   - Alternative: Return `Option<&V>` but requires dealing with lifetimes

**Why `&mut self`?**
- We need to mutate the order (update recency)
- In Rust, even "read" operations that update metadata require `&mut`
- Compare to C++: `const` methods on cache would be lying (mutation happens)

### Putting a Value

```rust
pub fn put(&mut self, key: K, value: V) {
    // If key exists, update value and recency
    if self.map.contains_key(&key) {
        self.map.insert(key.clone(), value);
        self.update_recency(&key);
        return;
    }

    // If at capacity, evict LRU
    if self.map.len() >= self.capacity {
        if let Some(lru_key) = self.order.pop_front() {
            self.map.remove(&lru_key);
        }
    }

    // Insert new entry
    self.map.insert(key.clone(), value);
    self.order.push_back(key);
}
```

**Key points**:
1. **Update existing**: If key exists, update value and move to back
2. **Eviction**: Before inserting new item, check capacity and evict LRU
3. **Insert**: Add to both HashMap and VecDeque

**Ownership considerations**:
- `key: K` takes ownership (we need to store it)
- `key.clone()` when we need to use it in multiple places
- Could optimize with `Rc<K>` to avoid clones, but adds complexity

### Updating Recency

```rust
fn update_recency(&mut self, key: &K) {
    // Find and remove key from current position
    if let Some(pos) = self.order.iter().position(|k| k == key) {
        self.order.remove(pos);
    }
    // Add to back (most recent)
    self.order.push_back(key.clone());
}
```

**Key points**:
- **Find**: O(n) scan to find key position
- **Remove**: O(n) to remove from middle of VecDeque
- **Add**: O(1) to add to back

**This is the performance bottleneck**: O(n) for each access. In a production implementation with high throughput requirements, you'd use a doubly-linked list.

**Why not use `retain`?**
```rust
// Alternative approach:
self.order.retain(|k| k != key);
self.order.push_back(key.clone());
```
This works but is less explicit. `position` + `remove` makes the intent clearer.

## Common Pitfalls and How We Avoid Them

### Pitfall 1: Stale Keys in Order VecDeque

**Problem**: If we don't remove keys from `order` when they're evicted or updated, we'll have stale entries.

**Our solution**:
- In `put` (update case): Call `update_recency` which removes old position
- In `put` (evict case): `pop_front` removes from order, then remove from map

### Pitfall 2: Capacity Edge Cases

**Problem**: What if capacity is 0? What if we insert when at capacity?

**Our solution**:
- Assert capacity > 0 in `new`
- Check `>= capacity` not `== capacity` (defensive)

### Pitfall 3: Borrow Checker Errors

**Common error**: Can't call `self.map.remove(&key)` while holding a reference to `self.order`

```rust
// ❌ This doesn't compile:
let lru_key = self.order.front().unwrap();
self.map.remove(lru_key);  // Error: can't borrow self as mutable

// ✅ Our solution: Use pop_front which takes ownership
if let Some(lru_key) = self.order.pop_front() {
    self.map.remove(&lru_key);
}
```

### Pitfall 4: Cloning vs Borrowing Confusion

**Problem**: When do we need to clone K?

**Our solution**:
- Store `K` in both `map` and `order`: Need clone when inserting
- Return reference from map: Don't need to clone K, but we clone V
- Update recency: Clone K when pushing to order

**Alternative**: Use `Rc<K>` to share keys between map and order, but adds complexity.

## Performance Analysis

### Time Complexity

| Operation | Our Implementation | Optimal (Doubly-Linked List) |
|-----------|-------------------|------------------------------|
| `get`     | O(n) | O(1) |
| `put`     | O(n) | O(1) |
| `len`     | O(1) | O(1) |
| `clear`   | O(n) | O(n) |

The O(n) comes from `update_recency` scanning the VecDeque.

### Space Complexity

- **HashMap**: O(n) where n is number of entries
- **VecDeque**: O(n) storing keys
- **Total**: O(n), with 2x overhead compared to storing keys once

**Optimization**: Use `Rc<K>` to share keys, reducing to ~1x overhead.

### When is This Good Enough?

This implementation is perfect for:
- **Small to medium caches** (< 1000 items)
- **Read-heavy workloads** where cache hits are common
- **Learning and prototyping**

For high-performance production use with large caches:
- Consider `linked-hash-map` crate
- Or implement index-based doubly-linked list

## Alternative Implementations

### Approach 1: IndexMap

Use `IndexMap` from the `indexmap` crate (maintains insertion order):

```rust
use indexmap::IndexMap;

pub struct LRUCache<K, V> {
    capacity: usize,
    map: IndexMap<K, V>,
}

// get: Move to back by remove + insert
// put: Similar logic, use move_index
```

**Pros**: Simpler (one data structure), maintained by community
**Cons**: Still O(n) for reordering, external dependency

### Approach 2: Index-Based Doubly-Linked List

Store indices instead of pointers:

```rust
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

pub struct LRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,  // Key -> node index
    nodes: Vec<Option<Node<K, V>>>,
    head: Option<usize>,
    tail: Option<usize>,
    free_list: Vec<usize>,
}
```

**Pros**: O(1) for all operations
**Cons**: Much more complex, easy to introduce bugs

### Approach 3: Using unsafe for Doubly-Linked List

Use raw pointers for a true doubly-linked list (requires `unsafe`):

```rust
struct Node<K, V> {
    key: K,
    value: V,
    prev: *mut Node<K, V>,
    next: *mut Node<K, V>,
}
```

**Pros**: O(1), matches C++ implementation
**Cons**: Requires unsafe, complex, error-prone

**Our choice**: Start with safe, simple implementation. Optimize later if profiling shows it's a bottleneck.

## Comparing to Other Languages

### TypeScript (JavaScript)

```typescript
class LRUCache<K, V> {
    private capacity: number;
    private cache = new Map<K, V>();

    get(key: K): V | undefined {
        if (!this.cache.has(key)) return undefined;
        const value = this.cache.get(key)!;
        this.cache.delete(key);  // Remove and re-insert to update order
        this.cache.set(key, value);
        return value;
    }

    put(key: K, value: V): void {
        if (this.cache.has(key)) {
            this.cache.delete(key);
        } else if (this.cache.size >= this.capacity) {
            const firstKey = this.cache.keys().next().value;
            this.cache.delete(firstKey);
        }
        this.cache.set(key, value);
    }
}
```

**Key differences**:
- TypeScript: Map maintains insertion order (since ES2015), making LRU simpler
- Rust: HashMap doesn't maintain order, need separate ordering structure
- TypeScript: No ownership/borrowing to worry about
- Rust: Must think carefully about when to clone vs borrow

### Python

```python
from collections import OrderedDict

class LRUCache:
    def __init__(self, capacity: int):
        self.cache = OrderedDict()
        self.capacity = capacity

    def get(self, key):
        if key not in self.cache:
            return None
        self.cache.move_to_end(key)  # Update order
        return self.cache[key]

    def put(self, key, value):
        if key in self.cache:
            self.cache.move_to_end(key)
        self.cache[key] = value
        if len(self.cache) > self.capacity:
            self.cache.popitem(last=False)  # Remove oldest
```

**Key differences**:
- Python: `OrderedDict` built-in, simpler implementation
- Rust: Need to build ordering mechanism yourself
- Python: Runtime overhead, GC
- Rust: Zero-cost abstractions, no GC

**Rust's advantage**: Compile-time guarantees, predictable performance, no GC pauses.

## Testing Strategy

Our tests cover:

1. **Basic operations**: Insert and retrieve
2. **Eviction**: LRU item is evicted when at capacity
3. **Recency updates**: Both `get` and `put` update recency
4. **Updates**: Updating existing key doesn't increase size
5. **Edge cases**: Clear, empty cache

**Additional tests to consider**:
- Capacity 1 (edge case)
- Large cache (performance)
- Interleaved operations (stress test)

## Key Takeaways

1. **Start simple**: VecDeque + HashMap is simpler than doubly-linked list
2. **Clone requirements**: Necessary for this simple approach, could be optimized with `Rc`
3. **Borrow checker**: Think about ownership at each step
4. **Performance tradeoffs**: O(n) is fine for small caches, optimize if needed
5. **Testing**: Cover basic operations, edge cases, and eviction logic
6. **Rust patterns**: Use `Option` for nullable values, `if let` for pattern matching

## Further Reading

- [HashMap documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [VecDeque documentation](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)
- [LRU crate](https://crates.io/crates/lru) - Production-ready implementation
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) - Best practices for API design
