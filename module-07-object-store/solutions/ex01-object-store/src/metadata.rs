use crate::error::{ObjectStoreError, Result};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, FromRow};

/// Object metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ObjectMetadata {
    pub bucket: String,
    pub key: String,
    pub content_hash: String,
    pub size: i64,
    pub content_type: Option<String>,
    pub created_at: i64,
}

/// Bucket metadata
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BucketMetadata {
    pub name: String,
    pub created_at: i64,
}

/// Metadata store using SQLite
pub struct MetadataStore {
    pool: SqlitePool,
}

impl MetadataStore {
    /// Create a new metadata store
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS buckets (
                name TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS objects (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                content_type TEXT,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (bucket, key),
                FOREIGN KEY (bucket) REFERENCES buckets(name) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(MetadataStore { pool })
    }

    // Bucket operations

    /// Create a new bucket
    pub async fn create_bucket(&self, name: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp();

        sqlx::query("INSERT INTO buckets (name, created_at) VALUES (?, ?)")
            .bind(name)
            .bind(now)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint") {
                    ObjectStoreError::BucketAlreadyExists(name.to_string())
                } else {
                    ObjectStoreError::Database(e)
                }
            })?;

        Ok(())
    }

    /// Delete a bucket
    pub async fn delete_bucket(&self, name: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM buckets WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List all buckets
    pub async fn list_buckets(&self) -> Result<Vec<BucketMetadata>> {
        let buckets = sqlx::query_as::<_, BucketMetadata>("SELECT * FROM buckets ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(buckets)
    }

    /// Check if bucket exists
    pub async fn bucket_exists(&self, name: &str) -> Result<bool> {
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM buckets WHERE name = ?")
            .bind(name)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0 > 0)
    }

    // Object operations

    /// Put object metadata
    pub async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        content_hash: &str,
        size: i64,
        content_type: Option<String>,
    ) -> Result<()> {
        // Verify bucket exists
        if !self.bucket_exists(bucket).await? {
            return Err(ObjectStoreError::BucketNotFound(bucket.to_string()));
        }

        let now = chrono::Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO objects (bucket, key, content_hash, size, content_type, created_at)
            VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(bucket, key) DO UPDATE SET
                content_hash = excluded.content_hash,
                size = excluded.size,
                content_type = excluded.content_type,
                created_at = excluded.created_at
            "#,
        )
        .bind(bucket)
        .bind(key)
        .bind(content_hash)
        .bind(size)
        .bind(content_type)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get object metadata
    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<ObjectMetadata> {
        let obj = sqlx::query_as::<_, ObjectMetadata>(
            "SELECT * FROM objects WHERE bucket = ? AND key = ?",
        )
        .bind(bucket)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            ObjectStoreError::ObjectNotFound(format!("{}/{}", bucket, key))
        })?;

        Ok(obj)
    }

    /// Delete object metadata
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM objects WHERE bucket = ? AND key = ?")
            .bind(bucket)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// List objects in a bucket
    pub async fn list_objects(&self, bucket: &str, prefix: Option<&str>) -> Result<Vec<ObjectMetadata>> {
        let objects = if let Some(prefix) = prefix {
            sqlx::query_as::<_, ObjectMetadata>(
                "SELECT * FROM objects WHERE bucket = ? AND key LIKE ? ORDER BY key",
            )
            .bind(bucket)
            .bind(format!("{}%", prefix))
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, ObjectMetadata>(
                "SELECT * FROM objects WHERE bucket = ? ORDER BY key",
            )
            .bind(bucket)
            .fetch_all(&self.pool)
            .await?
        };

        Ok(objects)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_list_buckets() {
        let store = MetadataStore::new("sqlite::memory:").await.unwrap();

        store.create_bucket("bucket1").await.unwrap();
        store.create_bucket("bucket2").await.unwrap();

        let buckets = store.list_buckets().await.unwrap();
        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].name, "bucket1");
        assert_eq!(buckets[1].name, "bucket2");
    }

    #[tokio::test]
    async fn test_duplicate_bucket() {
        let store = MetadataStore::new("sqlite::memory:").await.unwrap();

        store.create_bucket("bucket1").await.unwrap();
        let result = store.create_bucket("bucket1").await;

        assert!(matches!(result, Err(ObjectStoreError::BucketAlreadyExists(_))));
    }

    #[tokio::test]
    async fn test_put_get_object() {
        let store = MetadataStore::new("sqlite::memory:").await.unwrap();

        store.create_bucket("bucket1").await.unwrap();
        store
            .put_object("bucket1", "file.txt", "hash123", 1024, Some("text/plain".to_string()))
            .await
            .unwrap();

        let obj = store.get_object("bucket1", "file.txt").await.unwrap();
        assert_eq!(obj.key, "file.txt");
        assert_eq!(obj.content_hash, "hash123");
        assert_eq!(obj.size, 1024);
    }

    #[tokio::test]
    async fn test_list_objects_with_prefix() {
        let store = MetadataStore::new("sqlite::memory:").await.unwrap();

        store.create_bucket("bucket1").await.unwrap();
        store.put_object("bucket1", "docs/a.txt", "hash1", 100, None).await.unwrap();
        store.put_object("bucket1", "docs/b.txt", "hash2", 200, None).await.unwrap();
        store.put_object("bucket1", "images/c.jpg", "hash3", 300, None).await.unwrap();

        let docs = store.list_objects("bucket1", Some("docs/")).await.unwrap();
        assert_eq!(docs.len(), 2);

        let all = store.list_objects("bucket1", None).await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_object() {
        let store = MetadataStore::new("sqlite::memory:").await.unwrap();

        store.create_bucket("bucket1").await.unwrap();
        store.put_object("bucket1", "file.txt", "hash123", 1024, None).await.unwrap();

        assert!(store.delete_object("bucket1", "file.txt").await.unwrap());
        assert!(!store.delete_object("bucket1", "file.txt").await.unwrap());
    }
}
