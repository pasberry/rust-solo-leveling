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
