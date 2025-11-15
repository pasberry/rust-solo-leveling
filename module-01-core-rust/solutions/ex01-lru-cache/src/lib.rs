use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

/// An LRU (Least Recently Used) cache with a fixed capacity.
///
/// When the cache reaches capacity, the least recently used item is evicted.
/// Both `get` and `put` operations update the recency of accessed items.
///
/// # Examples
///
/// ```
/// use lru_cache::LRUCache;
///
/// let mut cache = LRUCache::new(2);
/// cache.put(1, "a");
/// cache.put(2, "b");
/// assert_eq!(cache.get(&1), Some("a"));
/// cache.put(3, "c");  // Evicts key 2
/// assert_eq!(cache.get(&2), None);
/// ```
pub struct LRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, V>,
    order: VecDeque<K>,  // Front = LRU, Back = MRU (most recently used)
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Creates a new LRU cache with the specified capacity.
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let cache: LRUCache<i32, String> = LRUCache::new(10);
    /// assert_eq!(cache.len(), 0);
    /// ```
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Capacity must be greater than 0");
        LRUCache {
            capacity,
            map: HashMap::with_capacity(capacity),
            order: VecDeque::with_capacity(capacity),
        }
    }

    /// Gets a value from the cache and marks it as recently used.
    ///
    /// Returns `None` if the key is not found in the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let mut cache = LRUCache::new(2);
    /// cache.put(1, "value");
    /// assert_eq!(cache.get(&1), Some("value"));
    /// assert_eq!(cache.get(&2), None);
    /// ```
    pub fn get(&mut self, key: &K) -> Option<V> {
        if !self.map.contains_key(key) {
            return None;
        }

        // Update recency: move to back (most recently used)
        self.update_recency(key);

        // Return cloned value
        // Note: We clone because returning a reference would require lifetimes
        // and complicate the API. For most use cases, this is acceptable.
        self.map.get(key).cloned()
    }

    /// Inserts or updates a key-value pair in the cache.
    ///
    /// If the key already exists, updates the value and marks it as recently used.
    /// If the cache is at capacity, evicts the least recently used item before inserting.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let mut cache = LRUCache::new(2);
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.put(3, "c");  // Evicts key 1
    /// assert_eq!(cache.get(&1), None);
    /// ```
    pub fn put(&mut self, key: K, value: V) {
        // Case 1: Key already exists - update value and recency
        if self.map.contains_key(&key) {
            self.map.insert(key.clone(), value);
            self.update_recency(&key);
            return;
        }

        // Case 2: At capacity - evict LRU before inserting
        if self.map.len() >= self.capacity {
            if let Some(lru_key) = self.order.pop_front() {
                self.map.remove(&lru_key);
            }
        }

        // Case 3: Insert new entry
        self.map.insert(key.clone(), value);
        self.order.push_back(key);
    }

    /// Returns the number of items currently in the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let mut cache = LRUCache::new(2);
    /// assert_eq!(cache.len(), 0);
    /// cache.put(1, "a");
    /// assert_eq!(cache.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the cache contains no items.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let mut cache = LRUCache::new(2);
    /// assert!(cache.is_empty());
    /// cache.put(1, "a");
    /// assert!(!cache.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes all items from the cache.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let mut cache = LRUCache::new(2);
    /// cache.put(1, "a");
    /// cache.put(2, "b");
    /// cache.clear();
    /// assert!(cache.is_empty());
    /// ```
    pub fn clear(&mut self) {
        self.map.clear();
        self.order.clear();
    }

    /// Returns the cache's capacity.
    ///
    /// # Examples
    ///
    /// ```
    /// use lru_cache::LRUCache;
    ///
    /// let cache: LRUCache<i32, String> = LRUCache::new(10);
    /// assert_eq!(cache.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Updates the recency of a key by moving it to the back of the order.
    ///
    /// This is called by both `get` and `put` to mark items as recently used.
    fn update_recency(&mut self, key: &K) {
        // Find the key's position in the order VecDeque
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            // Remove from current position
            self.order.remove(pos);
        }
        // Add to back (most recently used position)
        self.order.push_back(key.clone());
    }
}

// Bonus: Implement Debug for easier debugging
impl<K, V> std::fmt::Debug for LRUCache<K, V>
where
    K: Eq + Hash + Clone + std::fmt::Debug,
    V: Clone + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LRUCache")
            .field("capacity", &self.capacity)
            .field("len", &self.len())
            .field("order", &self.order)
            .finish()
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

    #[test]
    fn test_capacity() {
        let cache: LRUCache<i32, String> = LRUCache::new(5);
        assert_eq!(cache.capacity(), 5);
    }

    #[test]
    fn test_capacity_one() {
        let mut cache = LRUCache::new(1);

        cache.put(1, "a");
        assert_eq!(cache.get(&1), Some("a"));

        cache.put(2, "b");
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), Some("b"));
    }

    #[test]
    #[should_panic(expected = "Capacity must be greater than 0")]
    fn test_zero_capacity_panics() {
        let _cache: LRUCache<i32, String> = LRUCache::new(0);
    }

    #[test]
    fn test_different_types() {
        let mut cache: LRUCache<String, i32> = LRUCache::new(2);

        cache.put("one".to_string(), 1);
        cache.put("two".to_string(), 2);

        assert_eq!(cache.get(&"one".to_string()), Some(1));
        assert_eq!(cache.get(&"two".to_string()), Some(2));
    }

    #[test]
    fn test_complex_eviction_sequence() {
        let mut cache = LRUCache::new(3);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(3, "c");
        cache.get(&1);      // Order: 2, 3, 1
        cache.get(&2);      // Order: 3, 1, 2
        cache.put(4, "d");  // Evicts 3, Order: 1, 2, 4

        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&1), Some("a"));
        assert_eq!(cache.get(&2), Some("b"));
        assert_eq!(cache.get(&4), Some("d"));
    }

    #[test]
    fn test_update_moves_to_recent() {
        let mut cache = LRUCache::new(2);

        cache.put(1, "a");
        cache.put(2, "b");
        cache.put(1, "a_updated");  // Update 1, making it most recent
        cache.put(3, "c");          // Should evict 2

        assert_eq!(cache.get(&1), Some("a_updated"));
        assert_eq!(cache.get(&2), None);
        assert_eq!(cache.get(&3), Some("c"));
    }
}
