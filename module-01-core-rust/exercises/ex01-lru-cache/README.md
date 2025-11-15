# Exercise 01: LRU Cache

**Difficulty**: Medium
**Time**: 3-4 hours
**Focus**: Ownership, borrowing, generic types, interior mutability

## Overview

Build a **Least Recently Used (LRU) cache** from scratch. An LRU cache stores a limited number of key-value pairs and evicts the least recently used item when it reaches capacity.

This exercise will solidify your understanding of:
- Generic types and trait bounds
- Ownership and borrowing patterns
- Interior mutability (`Cell`, `RefCell`)
- HashMap and custom data structures
- Lifetimes in data structures

## What is an LRU Cache?

An LRU cache maintains items in order of use. When the cache is full and a new item is inserted, the **least recently used** item is evicted.

**Operations**:
- `get(key)` - Retrieve value, mark as recently used
- `put(key, value)` - Insert or update, mark as recently used
- If at capacity, evict least recently used before inserting

**Example**:
```
Cache capacity: 3

put(1, "a")  → Cache: [1:"a"]
put(2, "b")  → Cache: [1:"a", 2:"b"]
put(3, "c")  → Cache: [1:"a", 2:"b", 3:"c"]
get(1)       → Returns "a", Cache: [2:"b", 3:"c", 1:"a"]  (1 is now most recent)
put(4, "d")  → Cache: [3:"c", 1:"a", 4:"d"]  (2 evicted as LRU)
```

## Requirements

### API

Implement an `LRUCache<K, V>` with the following API:

```rust
pub struct LRUCache<K, V> {
    // Your implementation
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new LRU cache with given capacity
    pub fn new(capacity: usize) -> Self;

    /// Get a value from the cache, marking it as recently used
    /// Returns None if key not found
    pub fn get(&mut self, key: &K) -> Option<V>;

    /// Insert or update a key-value pair
    /// Evicts LRU item if at capacity
    pub fn put(&mut self, key: K, value: V);

    /// Get current number of items in cache
    pub fn len(&self) -> usize;

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool;

    /// Clear all items from cache
    pub fn clear(&mut self);
}
```

### Behavior

1. **Capacity enforcement**: Never exceed capacity
2. **LRU eviction**: When full, evict the least recently used item
3. **Update order**: Both `get` and `put` update the recency order
4. **Overwrite behavior**: `put` with existing key updates value and marks as recent

### Data Structure Hint

A typical LRU cache implementation uses:
- **HashMap** for O(1) key lookup
- **Doubly-linked list** for O(1) reordering and eviction

However, implementing a doubly-linked list with safe Rust is challenging due to ownership constraints. Consider these approaches:

1. **Vec-based** (simpler, O(n) reordering): Use a Vec to track order
2. **Index-based doubly-linked list**: Store indices instead of pointers
3. **VecDeque**: Use a deque for ordering
4. **External crate** (after attempting yourself): `linked-hash-map` crate

Start with the simpler approach, then optimize if needed.

## Acceptance Criteria

Your implementation must pass these tests:

```rust
#[test]
fn test_basic_operations() {
    let mut cache = LRUCache::new(2);

    cache.put(1, "a");
    cache.put(2, "b");
    assert_eq!(cache.get(&1), Some("a"));
    assert_eq!(cache.get(&2), Some("b"));
}

#[test]
fn test_eviction() {
    let mut cache = LRUCache::new(2);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.put(3, "c");  // Should evict key 1

    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some("b"));
    assert_eq!(cache.get(&3), Some("c"));
}

#[test]
fn test_get_updates_recency() {
    let mut cache = LRUCache::new(2);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.get(&1);      // Make 1 more recent
    cache.put(3, "c");  // Should evict 2, not 1

    assert_eq!(cache.get(&1), Some("a"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some("c"));
}

#[test]
fn test_update_existing() {
    let mut cache = LRUCache::new(2);

    cache.put(1, "a");
    cache.put(1, "b");  // Update value

    assert_eq!(cache.get(&1), Some("b"));
    assert_eq!(cache.len(), 1);
}

#[test]
fn test_clear() {
    let mut cache = LRUCache::new(2);

    cache.put(1, "a");
    cache.put(2, "b");
    cache.clear();

    assert_eq!(cache.len(), 0);
    assert!(cache.is_empty());
}
```

## Getting Started

1. **Understand the problem**: Read through the requirements and examples
2. **Choose a data structure**: Start with Vec-based for simplicity
3. **Implement step by step**:
   - Start with `new` and basic structure
   - Implement `put` without eviction
   - Add eviction logic
   - Implement `get` with reordering
   - Add helper methods
4. **Test incrementally**: Write tests as you go
5. **Refine**: Optimize or refactor once working

## Starter Code

```rust
use std::collections::HashMap;
use std::hash::Hash;

pub struct LRUCache<K, V> {
    capacity: usize,
    // TODO: Add fields for storage and tracking recency
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        // TODO: Initialize the cache
        todo!()
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        // TODO: Get value and update recency
        todo!()
    }

    pub fn put(&mut self, key: K, value: V) {
        // TODO: Insert/update and handle eviction
        todo!()
    }

    pub fn len(&self) -> usize {
        // TODO: Return number of items
        todo!()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        // TODO: Clear all items
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_operations() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "a");
        cache.put(2, "b");
        assert_eq!(cache.get(&1), Some("a"));
        assert_eq!(cache.get(&2), Some("b"));
    }

    #[test]
    fn test_eviction() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some("b"));
        assert_eq!(cache.get(&3), Some("c"));
    }

    #[test]
    fn test_get_updates_recency() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.get(&1);
        cache.put(3, "c");

        assert_eq!(cache.get(&1), Some("a"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some("c"));
    }

    #[test]
    fn test_update_existing() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "a");
        cache.put(1, "b");

        assert_eq!(cache.get(&1), Some("b"));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.clear();

        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }
}
```

## Tips

### Ownership Challenges

You'll encounter borrow checker errors. Common issues:

1. **Mutating while iterating**: Can't mutate a HashMap while holding a reference to its values
   - **Solution**: Collect keys to update first, then update in separate step

2. **Multiple mutable references**: Can't have multiple `&mut` to the same data
   - **Solution**: Break operations into steps, or use indices instead of references

3. **Cloning vs borrowing**: When to clone vs when to borrow
   - **Solution**: Clone when ownership is transferred, borrow when just reading

### Debugging Strategy

1. Start with the simplest case (capacity 1)
2. Print debug information to understand state
3. Run tests one at a time: `cargo test test_basic_operations`
4. Use `#[derive(Debug)]` on your types
5. Use `dbg!()` macro to inspect values

### Performance Considerations

For the first iteration, focus on **correctness** over performance. Optimize later.

- Vec-based approach: O(n) for reordering, O(1) for lookup
- Index-based linked list: O(1) for everything, more complex

## Stretch Goals

Once you have a working implementation:

1. **Add `peek` method**: Get value without updating recency
2. **Add `remove` method**: Explicitly remove an item
3. **Optimize**: Use index-based linked list for O(1) operations
4. **Generic over Clone**: Remove the `Clone` requirement (harder!)
5. **Add `capacity` method**: Return cache capacity
6. **Add iteration**: Implement `Iterator` to iterate in LRU order

## Resources

- [HashMap documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [VecDeque documentation](https://doc.rust-lang.org/std/collections/struct.VecDeque.html)
- Review Lecture 01 (Ownership), Lecture 05 (Generics), Lecture 06 (Collections)

## Next Steps

After completing this exercise:

1. **Compare with solution**: See [solutions/ex01-lru-cache/](../../solutions/ex01-lru-cache/) for reference implementation and commentary
2. **Experiment**: Try different data structures and measure performance
3. **Continue**: Move to [Exercise 02: Config-Driven CLI →](../ex02-config-cli/)

---

**Time to build!** Remember: struggle is part of learning. Spend at least 30 minutes before looking at hints or solutions.
