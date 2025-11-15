use crate::error::{ObjectStoreError, Result};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Content-addressed storage backend
pub struct ContentStore {
    root: PathBuf,
}

impl ContentStore {
    /// Create a new content store
    pub async fn new(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).await?;

        Ok(ContentStore { root })
    }

    /// Store content and return its SHA-256 hash
    pub async fn put<R: AsyncRead + Unpin>(&self, mut reader: R) -> Result<String> {
        let mut hasher = Sha256::new();
        let mut buffer = Vec::new();

        // Read all content and compute hash
        reader.read_to_end(&mut buffer).await?;
        hasher.update(&buffer);
        let hash = hex::encode(hasher.finalize());

        // Create nested directory structure (first 2 chars / next 2 chars / hash)
        let dir = self.hash_to_path(&hash);
        if let Some(parent) = dir.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write content to disk
        fs::write(&dir, &buffer).await?;

        Ok(hash)
    }

    /// Retrieve content by hash
    pub async fn get(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.hash_to_path(hash);

        if !path.exists() {
            return Err(ObjectStoreError::ObjectNotFound(hash.to_string()));
        }

        Ok(fs::read(&path).await?)
    }

    /// Check if content exists
    pub async fn exists(&self, hash: &str) -> bool {
        self.hash_to_path(hash).exists()
    }

    /// Delete content by hash
    pub async fn delete(&self, hash: &str) -> Result<bool> {
        let path = self.hash_to_path(hash);

        if !path.exists() {
            return Ok(false);
        }

        fs::remove_file(&path).await?;
        Ok(true)
    }

    /// Stream content to a writer
    pub async fn stream_to<W: AsyncWrite + Unpin>(
        &self,
        hash: &str,
        mut writer: W,
    ) -> Result<u64> {
        let path = self.hash_to_path(hash);

        if !path.exists() {
            return Err(ObjectStoreError::ObjectNotFound(hash.to_string()));
        }

        let content = fs::read(&path).await?;
        writer.write_all(&content).await?;
        Ok(content.len() as u64)
    }

    /// Convert hash to filesystem path
    /// Format: root/ab/cd/abcd1234...
    fn hash_to_path(&self, hash: &str) -> PathBuf {
        let prefix1 = &hash[0..2];
        let prefix2 = &hash[2..4];
        self.root.join(prefix1).join(prefix2).join(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_put_get() {
        let dir = tempdir().unwrap();
        let store = ContentStore::new(dir.path()).await.unwrap();

        let data = b"Hello, World!";
        let hash = store.put(&data[..]).await.unwrap();

        let retrieved = store.get(&hash).await.unwrap();
        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_content_addressing() {
        let dir = tempdir().unwrap();
        let store = ContentStore::new(dir.path()).await.unwrap();

        let data1 = b"Hello, World!";
        let data2 = b"Hello, World!";

        let hash1 = store.put(&data1[..]).await.unwrap();
        let hash2 = store.put(&data2[..]).await.unwrap();

        // Same content should produce same hash
        assert_eq!(hash1, hash2);
    }

    #[tokio::test]
    async fn test_exists() {
        let dir = tempdir().unwrap();
        let store = ContentStore::new(dir.path()).await.unwrap();

        let data = b"test data";
        let hash = store.put(&data[..]).await.unwrap();

        assert!(store.exists(&hash).await);
        assert!(!store.exists("nonexistent").await);
    }

    #[tokio::test]
    async fn test_delete() {
        let dir = tempdir().unwrap();
        let store = ContentStore::new(dir.path()).await.unwrap();

        let data = b"test data";
        let hash = store.put(&data[..]).await.unwrap();

        assert!(store.delete(&hash).await.unwrap());
        assert!(!store.exists(&hash).await);
        assert!(!store.delete(&hash).await.unwrap());
    }
}
