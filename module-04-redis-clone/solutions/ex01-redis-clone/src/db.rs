use crate::error::{DbError, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// The different value types supported by our Redis clone
#[derive(Debug, Clone)]
pub enum Value {
    String(Vec<u8>),
    List(VecDeque<Vec<u8>>),
    Set(HashSet<Vec<u8>>),
    Hash(HashMap<String, Vec<u8>>),
}

/// An entry in the database with optional expiration
#[derive(Debug, Clone)]
struct Entry {
    value: Value,
    expires_at: Option<Instant>,
}

impl Entry {
    fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| Instant::now() >= exp)
    }
}

/// The main database structure
#[derive(Clone)]
pub struct Db {
    data: Arc<RwLock<HashMap<String, Entry>>>,
}

impl Db {
    pub fn new() -> Self {
        Db {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Spawn a background task to clean up expired keys
    pub fn spawn_expiration_task(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let expired_keys = {
                    let data = self.data.read().await;
                    data.iter()
                        .filter(|(_, entry)| entry.is_expired())
                        .map(|(k, _)| k.clone())
                        .collect::<Vec<_>>()
                };

                if !expired_keys.is_empty() {
                    let mut data = self.data.write().await;
                    for key in expired_keys {
                        data.remove(&key);
                    }
                }
            }
        });
    }

    // String operations

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::String(bytes) => Ok(Some(bytes.clone())),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(None),
        }
    }

    pub async fn set(&self, key: String, value: Vec<u8>) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(
            key,
            Entry {
                value: Value::String(value),
                expires_at: None,
            },
        );
        Ok(())
    }

    pub async fn del(&self, key: &str) -> Result<bool> {
        let mut data = self.data.write().await;
        Ok(data.remove(key).is_some())
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let data = self.data.read().await;
        match data.get(key) {
            Some(entry) if !entry.is_expired() => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn expire(&self, key: &str, duration: Duration) -> Result<bool> {
        let mut data = self.data.write().await;

        if let Some(entry) = data.get_mut(key) {
            if entry.is_expired() {
                return Ok(false);
            }
            entry.expires_at = Some(Instant::now() + duration);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn ttl(&self, key: &str) -> Result<i64> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match entry.expires_at {
                Some(expires_at) => {
                    let now = Instant::now();
                    if now >= expires_at {
                        Ok(-2) // Key is expired
                    } else {
                        let remaining = expires_at.duration_since(now);
                        Ok(remaining.as_secs() as i64)
                    }
                }
                None => Ok(-1), // Key exists but has no expiration
            },
            _ => Ok(-2), // Key doesn't exist
        }
    }

    // List operations

    pub async fn lpush(&self, key: &str, values: Vec<Vec<u8>>) -> Result<usize> {
        let mut data = self.data.write().await;

        match data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::List(list) => {
                    for value in values.into_iter().rev() {
                        list.push_front(value);
                    }
                    Ok(list.len())
                }
                _ => Err(DbError::WrongType),
            },
            _ => {
                let mut list = VecDeque::new();
                for value in values.into_iter().rev() {
                    list.push_front(value);
                }
                let len = list.len();
                data.insert(
                    key.to_string(),
                    Entry {
                        value: Value::List(list),
                        expires_at: None,
                    },
                );
                Ok(len)
            }
        }
    }

    pub async fn rpush(&self, key: &str, values: Vec<Vec<u8>>) -> Result<usize> {
        let mut data = self.data.write().await;

        match data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::List(list) => {
                    for value in values {
                        list.push_back(value);
                    }
                    Ok(list.len())
                }
                _ => Err(DbError::WrongType),
            },
            _ => {
                let mut list = VecDeque::new();
                for value in values {
                    list.push_back(value);
                }
                let len = list.len();
                data.insert(
                    key.to_string(),
                    Entry {
                        value: Value::List(list),
                        expires_at: None,
                    },
                );
                Ok(len)
            }
        }
    }

    pub async fn lpop(&self, key: &str, count: usize) -> Result<Option<Vec<Vec<u8>>>> {
        let mut data = self.data.write().await;

        match data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::List(list) => {
                    let mut result = Vec::new();
                    for _ in 0..count.min(list.len()) {
                        if let Some(value) = list.pop_front() {
                            result.push(value);
                        }
                    }
                    if result.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(result))
                    }
                }
                _ => Err(DbError::WrongType),
            },
            _ => Ok(None),
        }
    }

    pub async fn rpop(&self, key: &str, count: usize) -> Result<Option<Vec<Vec<u8>>>> {
        let mut data = self.data.write().await;

        match data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::List(list) => {
                    let mut result = Vec::new();
                    for _ in 0..count.min(list.len()) {
                        if let Some(value) = list.pop_back() {
                            result.push(value);
                        }
                    }
                    if result.is_empty() {
                        Ok(None)
                    } else {
                        Ok(Some(result))
                    }
                }
                _ => Err(DbError::WrongType),
            },
            _ => Ok(None),
        }
    }

    pub async fn lrange(&self, key: &str, start: i64, stop: i64) -> Result<Vec<Vec<u8>>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::List(list) => {
                    let len = list.len() as i64;
                    let start = normalize_index(start, len);
                    let stop = normalize_index(stop, len);

                    if start > stop || start >= len {
                        return Ok(Vec::new());
                    }

                    let result = list
                        .iter()
                        .skip(start as usize)
                        .take((stop - start + 1) as usize)
                        .cloned()
                        .collect();
                    Ok(result)
                }
                _ => Err(DbError::WrongType),
            },
            _ => Ok(Vec::new()),
        }
    }

    pub async fn llen(&self, key: &str) -> Result<usize> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::List(list) => Ok(list.len()),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(0),
        }
    }

    // Set operations

    pub async fn sadd(&self, key: &str, members: Vec<Vec<u8>>) -> Result<usize> {
        let mut data = self.data.write().await;

        match data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::Set(set) => {
                    let mut count = 0;
                    for member in members {
                        if set.insert(member) {
                            count += 1;
                        }
                    }
                    Ok(count)
                }
                _ => Err(DbError::WrongType),
            },
            _ => {
                let mut set = HashSet::new();
                let count = members.len();
                for member in members {
                    set.insert(member);
                }
                data.insert(
                    key.to_string(),
                    Entry {
                        value: Value::Set(set),
                        expires_at: None,
                    },
                );
                Ok(count)
            }
        }
    }

    pub async fn smembers(&self, key: &str) -> Result<Vec<Vec<u8>>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Set(set) => Ok(set.iter().cloned().collect()),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(Vec::new()),
        }
    }

    pub async fn sismember(&self, key: &str, member: &[u8]) -> Result<bool> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Set(set) => Ok(set.contains(member)),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(false),
        }
    }

    pub async fn scard(&self, key: &str) -> Result<usize> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Set(set) => Ok(set.len()),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(0),
        }
    }

    // Hash operations

    pub async fn hset(&self, key: &str, field: String, value: Vec<u8>) -> Result<bool> {
        let mut data = self.data.write().await;

        match data.get_mut(key) {
            Some(entry) if !entry.is_expired() => match &mut entry.value {
                Value::Hash(hash) => Ok(hash.insert(field, value).is_none()),
                _ => Err(DbError::WrongType),
            },
            _ => {
                let mut hash = HashMap::new();
                hash.insert(field, value);
                data.insert(
                    key.to_string(),
                    Entry {
                        value: Value::Hash(hash),
                        expires_at: None,
                    },
                );
                Ok(true)
            }
        }
    }

    pub async fn hget(&self, key: &str, field: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => Ok(hash.get(field).cloned()),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(None),
        }
    }

    pub async fn hgetall(&self, key: &str) -> Result<HashMap<String, Vec<u8>>> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => Ok(hash.clone()),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(HashMap::new()),
        }
    }

    pub async fn hlen(&self, key: &str) -> Result<usize> {
        let data = self.data.read().await;

        match data.get(key) {
            Some(entry) if !entry.is_expired() => match &entry.value {
                Value::Hash(hash) => Ok(hash.len()),
                _ => Err(DbError::WrongType),
            },
            _ => Ok(0),
        }
    }
}

/// Normalize a Redis-style index (supports negative indices)
fn normalize_index(index: i64, len: i64) -> i64 {
    if index < 0 {
        (len + index).max(0)
    } else {
        index.min(len - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_set() {
        let db = Db::new();
        db.set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        let value = db.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let db = Db::new();
        let value = db.get("nonexistent").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_del() {
        let db = Db::new();
        db.set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        let deleted = db.del("key1").await.unwrap();
        assert!(deleted);

        let value = db.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_expire() {
        let db = Db::new();
        db.set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        db.expire("key1", Duration::from_millis(100))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(150)).await;

        let value = db.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_ttl() {
        let db = Db::new();
        db.set("key1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        db.expire("key1", Duration::from_secs(10)).await.unwrap();

        let ttl = db.ttl("key1").await.unwrap();
        assert!(ttl > 0 && ttl <= 10);
    }

    #[tokio::test]
    async fn test_lpush_rpush() {
        let db = Db::new();
        db.lpush("mylist", vec![b"world".to_vec()]).await.unwrap();
        db.lpush("mylist", vec![b"hello".to_vec()]).await.unwrap();
        db.rpush("mylist", vec![b"!".to_vec()]).await.unwrap();

        let range = db.lrange("mylist", 0, -1).await.unwrap();
        assert_eq!(
            range,
            vec![b"hello".to_vec(), b"world".to_vec(), b"!".to_vec()]
        );
    }

    #[tokio::test]
    async fn test_lpop_rpop() {
        let db = Db::new();
        db.rpush(
            "mylist",
            vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()],
        )
        .await
        .unwrap();

        let popped = db.lpop("mylist", 1).await.unwrap();
        assert_eq!(popped, Some(vec![b"one".to_vec()]));

        let popped = db.rpop("mylist", 1).await.unwrap();
        assert_eq!(popped, Some(vec![b"three".to_vec()]));
    }

    #[tokio::test]
    async fn test_lrange() {
        let db = Db::new();
        db.rpush(
            "mylist",
            vec![
                b"one".to_vec(),
                b"two".to_vec(),
                b"three".to_vec(),
                b"four".to_vec(),
            ],
        )
        .await
        .unwrap();

        let range = db.lrange("mylist", 1, 2).await.unwrap();
        assert_eq!(range, vec![b"two".to_vec(), b"three".to_vec()]);

        let range = db.lrange("mylist", 0, -1).await.unwrap();
        assert_eq!(range.len(), 4);
    }

    #[tokio::test]
    async fn test_sadd_smembers() {
        let db = Db::new();
        let count = db
            .sadd("myset", vec![b"one".to_vec(), b"two".to_vec()])
            .await
            .unwrap();
        assert_eq!(count, 2);

        let members = db.smembers("myset").await.unwrap();
        assert_eq!(members.len(), 2);
        assert!(members.contains(&b"one".to_vec()));
        assert!(members.contains(&b"two".to_vec()));
    }

    #[tokio::test]
    async fn test_sismember() {
        let db = Db::new();
        db.sadd("myset", vec![b"hello".to_vec()]).await.unwrap();

        let is_member = db.sismember("myset", b"hello").await.unwrap();
        assert!(is_member);

        let is_member = db.sismember("myset", b"world").await.unwrap();
        assert!(!is_member);
    }

    #[tokio::test]
    async fn test_hset_hget() {
        let db = Db::new();
        db.hset("myhash", "field1".to_string(), b"value1".to_vec())
            .await
            .unwrap();

        let value = db.hget("myhash", "field1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));
    }

    #[tokio::test]
    async fn test_hgetall() {
        let db = Db::new();
        db.hset("myhash", "field1".to_string(), b"value1".to_vec())
            .await
            .unwrap();
        db.hset("myhash", "field2".to_string(), b"value2".to_vec())
            .await
            .unwrap();

        let hash = db.hgetall("myhash").await.unwrap();
        assert_eq!(hash.len(), 2);
        assert_eq!(hash.get("field1"), Some(&b"value1".to_vec()));
        assert_eq!(hash.get("field2"), Some(&b"value2".to_vec()));
    }

    #[tokio::test]
    async fn test_wrong_type_error() {
        let db = Db::new();
        db.set("mykey".to_string(), b"value".to_vec())
            .await
            .unwrap();

        let result = db.lpush("mykey", vec![b"item".to_vec()]).await;
        assert!(matches!(result, Err(DbError::WrongType)));
    }
}
