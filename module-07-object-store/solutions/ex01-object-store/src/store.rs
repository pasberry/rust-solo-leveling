use crate::error::{ObjectStoreError, Result};
use crate::metadata::{BucketMetadata, MetadataStore, ObjectMetadata};
use crate::storage::ContentStore;
use std::path::Path;
use tokio::io::AsyncRead;

/// Main object store combining content storage and metadata
pub struct ObjectStore {
    content: ContentStore,
    metadata: MetadataStore,
}

impl ObjectStore {
    /// Create a new object store
    pub async fn new(storage_path: impl AsRef<Path>, database_url: &str) -> Result<Self> {
        let content = ContentStore::new(storage_path).await?;
        let metadata = MetadataStore::new(database_url).await?;

        Ok(ObjectStore { content, metadata })
    }

    // Bucket operations

    /// Create a new bucket
    pub async fn create_bucket(&self, name: &str) -> Result<()> {
        validate_bucket_name(name)?;
        self.metadata.create_bucket(name).await
    }

    /// Delete a bucket and all its objects
    pub async fn delete_bucket(&self, name: &str) -> Result<bool> {
        // List all objects in the bucket
        let objects = self.metadata.list_objects(name, None).await?;

        // Delete all object content
        for obj in objects {
            let _ = self.content.delete(&obj.content_hash).await;
        }

        // Delete bucket metadata (cascade deletes objects metadata)
        self.metadata.delete_bucket(name).await
    }

    /// List all buckets
    pub async fn list_buckets(&self) -> Result<Vec<BucketMetadata>> {
        self.metadata.list_buckets().await
    }

    // Object operations

    /// Put an object
    pub async fn put_object<R: AsyncRead + Unpin>(
        &self,
        bucket: &str,
        key: &str,
        mut content: R,
        content_type: Option<String>,
    ) -> Result<ObjectMetadata> {
        validate_object_key(key)?;

        // Store content and get hash
        let content_hash = self.content.put(&mut content).await?;

        // Get size
        let data = self.content.get(&content_hash).await?;
        let size = data.len() as i64;

        // Store metadata
        self.metadata
            .put_object(bucket, key, &content_hash, size, content_type.clone())
            .await?;

        // Return metadata
        self.metadata.get_object(bucket, key).await
    }

    /// Get an object
    pub async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>> {
        let metadata = self.metadata.get_object(bucket, key).await?;
        self.content.get(&metadata.content_hash).await
    }

    /// Get object metadata
    pub async fn head_object(&self, bucket: &str, key: &str) -> Result<ObjectMetadata> {
        self.metadata.get_object(bucket, key).await
    }

    /// Delete an object
    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<bool> {
        // Get metadata first to get content hash
        if let Ok(metadata) = self.metadata.get_object(bucket, key).await {
            // Delete content (may be shared by other objects)
            let _ = self.content.delete(&metadata.content_hash).await;

            // Delete metadata
            self.metadata.delete_object(bucket, key).await
        } else {
            Ok(false)
        }
    }

    /// List objects in a bucket
    pub async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<ObjectMetadata>> {
        self.metadata.list_objects(bucket, prefix).await
    }

    /// Copy an object
    pub async fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        dest_bucket: &str,
        dest_key: &str,
    ) -> Result<ObjectMetadata> {
        // Get source metadata
        let source = self.metadata.get_object(source_bucket, source_key).await?;

        // Copy metadata (reuses content hash - deduplication!)
        self.metadata
            .put_object(
                dest_bucket,
                dest_key,
                &source.content_hash,
                source.size,
                source.content_type,
            )
            .await?;

        self.metadata.get_object(dest_bucket, dest_key).await
    }
}

/// Validate bucket name (simplified S3 rules)
fn validate_bucket_name(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 63 {
        return Err(ObjectStoreError::InvalidBucketName(
            "Bucket name must be 1-63 characters".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(ObjectStoreError::InvalidBucketName(
            "Bucket name must be lowercase alphanumeric or hyphens".to_string(),
        ));
    }

    Ok(())
}

/// Validate object key
fn validate_object_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 1024 {
        return Err(ObjectStoreError::InvalidObjectKey(
            "Object key must be 1-1024 characters".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_bucket_and_put_object() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        store.create_bucket("my-bucket").await.unwrap();

        let data = b"Hello, S3!";
        let metadata = store
            .put_object(
                "my-bucket",
                "greeting.txt",
                &data[..],
                Some("text/plain".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(metadata.key, "greeting.txt");
        assert_eq!(metadata.size, 10);
    }

    #[tokio::test]
    async fn test_get_object() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        store.create_bucket("my-bucket").await.unwrap();

        let data = b"Hello, S3!";
        store
            .put_object("my-bucket", "greeting.txt", &data[..], None)
            .await
            .unwrap();

        let retrieved = store.get_object("my-bucket", "greeting.txt").await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_list_objects() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        store.create_bucket("my-bucket").await.unwrap();

        store.put_object("my-bucket", "file1.txt", &b"data1"[..], None).await.unwrap();
        store.put_object("my-bucket", "file2.txt", &b"data2"[..], None).await.unwrap();
        store.put_object("my-bucket", "docs/file3.txt", &b"data3"[..], None).await.unwrap();

        let all = store.list_objects("my-bucket", None).await.unwrap();
        assert_eq!(all.len(), 3);

        let docs = store.list_objects("my-bucket", Some("docs/")).await.unwrap();
        assert_eq!(docs.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_object() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        store.create_bucket("my-bucket").await.unwrap();

        store.put_object("my-bucket", "file.txt", &b"data"[..], None).await.unwrap();

        assert!(store.delete_object("my-bucket", "file.txt").await.unwrap());
        assert!(!store.delete_object("my-bucket", "file.txt").await.unwrap());
    }

    #[tokio::test]
    async fn test_copy_object() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        store.create_bucket("bucket1").await.unwrap();
        store.create_bucket("bucket2").await.unwrap();

        let data = b"Hello, World!";
        store.put_object("bucket1", "original.txt", &data[..], None).await.unwrap();

        store
            .copy_object("bucket1", "original.txt", "bucket2", "copy.txt")
            .await
            .unwrap();

        let copied = store.get_object("bucket2", "copy.txt").await.unwrap();
        assert_eq!(copied, data);
    }

    #[tokio::test]
    async fn test_content_deduplication() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        store.create_bucket("my-bucket").await.unwrap();

        let data = b"duplicate content";

        let meta1 = store.put_object("my-bucket", "file1.txt", &data[..], None).await.unwrap();
        let meta2 = store.put_object("my-bucket", "file2.txt", &data[..], None).await.unwrap();

        // Same content should have same hash (deduplication)
        assert_eq!(meta1.content_hash, meta2.content_hash);
    }

    #[tokio::test]
    async fn test_bucket_validation() {
        let dir = tempdir().unwrap();
        let store = ObjectStore::new(dir.path(), "sqlite::memory:")
            .await
            .unwrap();

        // Invalid: uppercase
        assert!(store.create_bucket("MyBucket").await.is_err());

        // Invalid: underscore
        assert!(store.create_bucket("my_bucket").await.is_err());

        // Valid
        assert!(store.create_bucket("my-bucket-123").await.is_ok());
    }
}
