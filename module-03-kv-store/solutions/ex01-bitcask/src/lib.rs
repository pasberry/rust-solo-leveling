mod error;
mod log;
mod store;

pub use error::{KvError, Result};
pub use store::KvStore;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path()).unwrap();

        store.set(b"key1", b"value1").unwrap();
        assert_eq!(store.get(b"key1").unwrap(), Some(b"value1".to_vec()));

        store.set(b"key1", b"value2").unwrap();
        assert_eq!(store.get(b"key1").unwrap(), Some(b"value2".to_vec()));

        store.delete(b"key1").unwrap();
        assert_eq!(store.get(b"key1").unwrap(), None);
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();

        {
            let mut store = KvStore::open(temp_dir.path()).unwrap();
            store.set(b"persistent", b"data").unwrap();
        }

        {
            let store = KvStore::open(temp_dir.path()).unwrap();
            assert_eq!(store.get(b"persistent").unwrap(), Some(b"data".to_vec()));
        }
    }

    #[test]
    fn test_compaction() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(temp_dir.path()).unwrap();

        for i in 0..100 {
            store.set(&i.to_le_bytes(), b"value").unwrap();
        }
        for i in 0..50 {
            store.delete(&i.to_le_bytes()).unwrap();
        }

        store.compact().unwrap();

        for i in 50..100 {
            assert!(store.get(&i.to_le_bytes()).unwrap().is_some());
        }
        for i in 0..50 {
            assert!(store.get(&i.to_le_bytes()).unwrap().is_none());
        }
    }
}
