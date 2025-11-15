# Module 07: S3-like Object Store

**Build a Simple Object Storage Service**

## Overview

Build an object storage system with:
- HTTP API for object operations
- Streaming uploads/downloads
- Metadata management
- Bucket operations
- Multipart upload support
- Object versioning

**Duration**: 2-3 weeks (25-30 hours)

## What You'll Build

```rust
// Client usage
let client = ObjectStore::connect("http://localhost:9000").await?;

// Create bucket
client.create_bucket("my-bucket").await?;

// Upload object
let data = tokio::fs::read("large-file.mp4").await?;
client.put_object("my-bucket", "videos/file.mp4", data).await?;

// Download object
let stream = client.get_object("my-bucket", "videos/file.mp4").await?;
tokio::io::copy(&mut stream, &mut tokio::fs::File::create("output.mp4").await?).await?;

// List objects
let objects = client.list_objects("my-bucket", Some("videos/")).await?;
```

## Architecture

```
┌─────────────────────────────────────┐
│         HTTP API Server             │
│    (PUT/GET/DELETE/LIST objects)    │
└──────────┬──────────────────────────┘
           │
    ┌──────┴──────────────────────────┐
    │                                 │
┌───▼────────────┐         ┌─────────▼────────┐
│ Metadata Store │         │  Object Storage  │
│   (SQLite)     │         │  (File System)   │
└────────────────┘         └──────────────────┘
     Indexes:                  Layout:
     - Bucket → Objects        buckets/
     - Object metadata           my-bucket/
     - Versions                    ab/cd/abcd1234...
```

## Key Components

### 1. Object Metadata

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    pub bucket: String,
    pub key: String,
    pub version_id: String,
    pub size: u64,
    pub etag: String,
    pub content_type: String,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub custom_metadata: HashMap<String, String>,
    pub storage_class: StorageClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageClass {
    Standard,
    InfrequentAccess,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bucket {
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub versioning_enabled: bool,
    pub region: String,
}
```

### 2. Storage Backend

```rust
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::fs::{File, create_dir_all};
use sha2::{Sha256, Digest};

pub struct FileSystemBackend {
    root_path: PathBuf,
}

impl FileSystemBackend {
    pub fn new(root_path: PathBuf) -> Self {
        FileSystemBackend { root_path }
    }

    /// Store object on filesystem with content-addressed storage
    pub async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: impl AsyncRead + Unpin,
    ) -> Result<(String, u64)> {
        let mut hasher = Sha256::new();
        let mut size = 0u64;

        // Create temporary file
        let temp_path = self.temp_file_path();
        let mut temp_file = File::create(&temp_path).await?;

        // Stream data while hashing
        let mut reader = data;
        let mut buffer = vec![0u8; 8192];

        loop {
            let n = reader.read(&mut buffer).await?;
            if n == 0 { break; }

            hasher.update(&buffer[..n]);
            temp_file.write_all(&buffer[..n]).await?;
            size += n as u64;
        }

        temp_file.flush().await?;
        drop(temp_file);

        // Calculate content hash (etag)
        let etag = format!("{:x}", hasher.finalize());

        // Move to final location (content-addressed)
        let final_path = self.object_path(bucket, &etag);
        create_dir_all(final_path.parent().unwrap()).await?;
        tokio::fs::rename(&temp_path, &final_path).await?;

        Ok((etag, size))
    }

    pub async fn get_object(
        &self,
        bucket: &str,
        etag: &str,
    ) -> Result<File> {
        let path = self.object_path(bucket, etag);
        Ok(File::open(path).await?)
    }

    pub async fn delete_object(
        &self,
        bucket: &str,
        etag: &str,
    ) -> Result<()> {
        let path = self.object_path(bucket, etag);
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    /// Content-addressed path: buckets/{bucket}/ab/cd/abcd1234...
    fn object_path(&self, bucket: &str, etag: &str) -> PathBuf {
        let prefix = &etag[..4];
        self.root_path
            .join("buckets")
            .join(bucket)
            .join(&prefix[..2])
            .join(&prefix[2..4])
            .join(etag)
    }

    fn temp_file_path(&self) -> PathBuf {
        let uuid = uuid::Uuid::new_v4();
        self.root_path.join("temp").join(uuid.to_string())
    }
}
```

### 3. Metadata Store

```rust
use rusqlite::{Connection, params};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MetadataStore {
    conn: Arc<Mutex<Connection>>,
}

impl MetadataStore {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS buckets (
                name TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                versioning_enabled INTEGER NOT NULL,
                region TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS objects (
                bucket TEXT NOT NULL,
                key TEXT NOT NULL,
                version_id TEXT NOT NULL,
                etag TEXT NOT NULL,
                size INTEGER NOT NULL,
                content_type TEXT NOT NULL,
                last_modified TEXT NOT NULL,
                is_latest INTEGER NOT NULL,
                custom_metadata TEXT,
                PRIMARY KEY (bucket, key, version_id),
                FOREIGN KEY (bucket) REFERENCES buckets(name)
            )",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_bucket_key ON objects(bucket, key)",
            [],
        )?;

        Ok(MetadataStore {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub async fn create_bucket(&self, bucket: &Bucket) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO buckets (name, created_at, versioning_enabled, region)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                bucket.name,
                bucket.created_at.to_rfc3339(),
                bucket.versioning_enabled as i32,
                bucket.region,
            ],
        )?;
        Ok(())
    }

    pub async fn put_object_metadata(&self, metadata: &ObjectMetadata) -> Result<()> {
        let conn = self.conn.lock().await;

        // If versioning is enabled, mark old versions as not latest
        if self.is_versioning_enabled(&metadata.bucket).await? {
            conn.execute(
                "UPDATE objects SET is_latest = 0
                 WHERE bucket = ?1 AND key = ?2",
                params![metadata.bucket, metadata.key],
            )?;
        } else {
            // If versioning disabled, delete old version
            conn.execute(
                "DELETE FROM objects WHERE bucket = ?1 AND key = ?2",
                params![metadata.bucket, metadata.key],
            )?;
        }

        // Insert new version
        conn.execute(
            "INSERT INTO objects
             (bucket, key, version_id, etag, size, content_type, last_modified, is_latest, custom_metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
            params![
                metadata.bucket,
                metadata.key,
                metadata.version_id,
                metadata.etag,
                metadata.size as i64,
                metadata.content_type,
                metadata.last_modified.to_rfc3339(),
                serde_json::to_string(&metadata.custom_metadata)?,
            ],
        )?;

        Ok(())
    }

    pub async fn get_object_metadata(
        &self,
        bucket: &str,
        key: &str,
        version_id: Option<&str>,
    ) -> Result<Option<ObjectMetadata>> {
        let conn = self.conn.lock().await;

        let query = if let Some(vid) = version_id {
            "SELECT * FROM objects WHERE bucket = ?1 AND key = ?2 AND version_id = ?3"
        } else {
            "SELECT * FROM objects WHERE bucket = ?1 AND key = ?2 AND is_latest = 1"
        };

        let mut stmt = conn.prepare(query)?;
        let metadata = stmt.query_row(
            params![bucket, key, version_id.unwrap_or("")],
            |row| {
                Ok(ObjectMetadata {
                    bucket: row.get(0)?,
                    key: row.get(1)?,
                    version_id: row.get(2)?,
                    etag: row.get(3)?,
                    size: row.get::<_, i64>(4)? as u64,
                    content_type: row.get(5)?,
                    last_modified: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    is_latest: row.get::<_, i32>(7)? == 1,
                    custom_metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap(),
                    storage_class: StorageClass::Standard,
                })
            },
        ).optional()?;

        Ok(metadata)
    }

    pub async fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        max_keys: usize,
    ) -> Result<Vec<ObjectMetadata>> {
        let conn = self.conn.lock().await;

        let (query, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(pfx) = prefix {
            (
                "SELECT * FROM objects
                 WHERE bucket = ?1 AND key LIKE ?2 AND is_latest = 1
                 ORDER BY key
                 LIMIT ?3",
                vec![
                    Box::new(bucket.to_string()),
                    Box::new(format!("{}%", pfx)),
                    Box::new(max_keys as i64),
                ],
            )
        } else {
            (
                "SELECT * FROM objects
                 WHERE bucket = ?1 AND is_latest = 1
                 ORDER BY key
                 LIMIT ?2",
                vec![
                    Box::new(bucket.to_string()),
                    Box::new(max_keys as i64),
                ],
            )
        };

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(
            params.as_slice(),
            |row| {
                Ok(ObjectMetadata {
                    bucket: row.get(0)?,
                    key: row.get(1)?,
                    version_id: row.get(2)?,
                    etag: row.get(3)?,
                    size: row.get::<_, i64>(4)? as u64,
                    content_type: row.get(5)?,
                    last_modified: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                        .unwrap()
                        .with_timezone(&chrono::Utc),
                    is_latest: row.get::<_, i32>(7)? == 1,
                    custom_metadata: serde_json::from_str(&row.get::<_, String>(8)?).unwrap(),
                    storage_class: StorageClass::Standard,
                })
            },
        )?;

        let mut objects = Vec::new();
        for row in rows {
            objects.push(row?);
        }

        Ok(objects)
    }

    async fn is_versioning_enabled(&self, bucket: &str) -> Result<bool> {
        let conn = self.conn.lock().await;
        let enabled: i32 = conn.query_row(
            "SELECT versioning_enabled FROM buckets WHERE name = ?1",
            params![bucket],
            |row| row.get(0),
        )?;
        Ok(enabled == 1)
    }
}
```

### 4. HTTP API Server

```rust
use axum::{
    Router,
    routing::{get, put, delete},
    extract::{Path, Query, State},
    http::StatusCode,
    body::{Body, StreamBody},
};
use tower_http::trace::TraceLayer;

pub struct ObjectStoreServer {
    storage: Arc<FileSystemBackend>,
    metadata: Arc<MetadataStore>,
}

impl ObjectStoreServer {
    pub fn new(storage: FileSystemBackend, metadata: MetadataStore) -> Self {
        ObjectStoreServer {
            storage: Arc::new(storage),
            metadata: Arc::new(metadata),
        }
    }

    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            .route("/buckets", put(create_bucket))
            .route("/buckets/:bucket", get(list_objects))
            .route("/buckets/:bucket/:key", put(put_object))
            .route("/buckets/:bucket/:key", get(get_object))
            .route("/buckets/:bucket/:key", delete(delete_object))
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }

    pub async fn serve(self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router()).await?;
        Ok(())
    }
}

// Handler: PUT /buckets
async fn create_bucket(
    State(server): State<Arc<ObjectStoreServer>>,
    Path(bucket_name): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let bucket = Bucket {
        name: bucket_name,
        created_at: chrono::Utc::now(),
        versioning_enabled: false,
        region: "us-east-1".to_string(),
    };

    server.metadata.create_bucket(&bucket).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

// Handler: PUT /buckets/{bucket}/{key}
async fn put_object(
    State(server): State<Arc<ObjectStoreServer>>,
    Path((bucket, key)): Path<(String, String)>,
    body: Body,
) -> Result<(StatusCode, String), StatusCode> {
    // Stream body to storage
    let stream = body.into_data_stream();
    let reader = tokio_util::io::StreamReader::new(
        stream.map(|result| result.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
    );

    let (etag, size) = server.storage.put_object(&bucket, &key, reader).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Store metadata
    let metadata = ObjectMetadata {
        bucket: bucket.clone(),
        key: key.clone(),
        version_id: uuid::Uuid::new_v4().to_string(),
        size,
        etag: etag.clone(),
        content_type: "application/octet-stream".to_string(),
        last_modified: chrono::Utc::now(),
        custom_metadata: HashMap::new(),
        storage_class: StorageClass::Standard,
    };

    server.metadata.put_object_metadata(&metadata).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, etag))
}

// Handler: GET /buckets/{bucket}/{key}
async fn get_object(
    State(server): State<Arc<ObjectStoreServer>>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<StreamBody<ReaderStream<File>>, StatusCode> {
    // Get metadata
    let metadata = server.metadata.get_object_metadata(&bucket, &key, None).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Get object from storage
    let file = server.storage.get_object(&bucket, &metadata.etag).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = ReaderStream::new(file);
    Ok(StreamBody::new(stream))
}

// Handler: GET /buckets/{bucket}?prefix=...&max-keys=...
#[derive(Deserialize)]
struct ListParams {
    prefix: Option<String>,
    #[serde(default = "default_max_keys")]
    max_keys: usize,
}

fn default_max_keys() -> usize { 1000 }

async fn list_objects(
    State(server): State<Arc<ObjectStoreServer>>,
    Path(bucket): Path<String>,
    Query(params): Query<ListParams>,
) -> Result<axum::Json<Vec<ObjectMetadata>>, StatusCode> {
    let objects = server.metadata.list_objects(
        &bucket,
        params.prefix.as_deref(),
        params.max_keys,
    ).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(axum::Json(objects))
}

// Handler: DELETE /buckets/{bucket}/{key}
async fn delete_object(
    State(server): State<Arc<ObjectStoreServer>>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    let metadata = server.metadata.get_object_metadata(&bucket, &key, None).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    server.storage.delete_object(&bucket, &metadata.etag).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Mark as deleted in metadata (soft delete for versioning)

    Ok(StatusCode::NO_CONTENT)
}
```

## Implementation Roadmap

### Phase 1: Core Storage Backend (Days 1-3)
- Implement FileSystemBackend with content-addressed storage
- Add streaming upload/download
- Write tests for concurrent access
- Benchmark I/O performance

**Success criteria:**
- Can store and retrieve large files (>1GB)
- Streaming works without loading entire file in memory
- Content hashing produces correct ETags

### Phase 2: Metadata Store (Days 4-5)
- Set up SQLite schema for buckets and objects
- Implement CRUD operations
- Add indexing for fast lookups
- Test concurrent metadata operations

**Success criteria:**
- Fast object lookups (<5ms)
- Support for 1M+ objects per bucket
- ACID guarantees for metadata

### Phase 3: HTTP API Server (Days 6-8)
- Build axum server with REST endpoints
- Implement PUT/GET/DELETE/LIST operations
- Add proper error handling
- Support streaming uploads/downloads

**Success criteria:**
- All basic S3 operations work
- Can upload/download via curl
- Proper HTTP status codes

### Phase 4: Object Versioning (Days 9-10)
- Add version_id to metadata
- Support version queries
- Implement soft deletes
- Add version listing

**Success criteria:**
- Multiple versions of same object
- Can retrieve old versions
- Delete markers work correctly

### Phase 5: Multipart Upload (Days 11-13)
- Implement multipart upload protocol
- Part assembly and verification
- Cleanup of incomplete uploads
- Concurrent part uploads

**Success criteria:**
- Can upload 5GB+ files in parts
- Failed uploads cleanup properly
- Parts can be uploaded concurrently

### Phase 6: Advanced Features (Days 14-15)
- Presigned URLs for temporary access
- Metadata-only operations (HEAD)
- Range requests for partial downloads
- Storage class management

## Performance Targets

- **Upload throughput**: >500 MB/s for large files
- **Download throughput**: >800 MB/s
- **Small object latency**: <10ms
- **List operations**: <100ms for 10k objects
- **Concurrent uploads**: Handle 100+ simultaneous

## Success Criteria

- ✅ Stores objects with content-addressed storage
- ✅ Streaming upload/download for large files
- ✅ SQLite metadata for fast lookups
- ✅ RESTful HTTP API compatible with S3 basics
- ✅ Object versioning support
- ✅ Multipart upload for large files
- ✅ Performance meets targets

## Comparison with Real S3

**What you'll implement:**
- Basic PUT/GET/DELETE/LIST operations
- Content-addressed storage
- Object versioning
- Multipart uploads
- Local filesystem backend

**What S3 has that you won't:**
- Distributed storage across regions
- Lifecycle policies and transitions
- Replication across regions
- Access control (IAM)
- Event notifications
- Analytics and logging

**Learning focus:**
- Streaming I/O in Rust
- HTTP API design with axum
- Metadata management with SQLite
- Content-addressed storage patterns

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get_object() {
        let storage = FileSystemBackend::new(temp_dir());
        let data = b"Hello, object store!";

        let (etag, size) = storage.put_object("test-bucket", "test-key", &data[..]).await.unwrap();
        assert_eq!(size, data.len() as u64);

        let mut retrieved = Vec::new();
        storage.get_object("test-bucket", &etag).await.unwrap()
            .read_to_end(&mut retrieved).await.unwrap();

        assert_eq!(retrieved, data);
    }

    #[tokio::test]
    async fn test_large_file_streaming() {
        // Generate 100MB random data
        let data = vec![0u8; 100 * 1024 * 1024];
        // Test streaming without OOM
    }

    #[tokio::test]
    async fn test_concurrent_uploads() {
        // Upload 100 objects concurrently
        // Verify all succeed
    }

    #[tokio::test]
    async fn test_object_versioning() {
        // Upload same key multiple times
        // Verify all versions retrievable
    }
}
```

## Extensions & Variations

**After completing the core project, try:**

1. **Add compression**:
   - Transparent compression for objects
   - Content-type based decisions

2. **Implement caching**:
   - LRU cache for hot objects
   - CDN-like edge caching

3. **Add encryption**:
   - Server-side encryption at rest
   - Client-provided keys

4. **Build CLI client**:
   - Like AWS CLI for your store
   - Progress bars for uploads

5. **Add replication**:
   - Async replication to backup location
   - Cross-region simulation

## Resources

**Object Storage Concepts:**
- [S3 API Documentation](https://docs.aws.amazon.com/s3/)
- MinIO architecture docs
- Content-addressed storage papers

**Rust Crates:**
- `axum` - HTTP server framework
- `tokio` - Async runtime
- `rusqlite` - SQLite bindings
- `sha2` - Hashing for ETags
- `uuid` - Version IDs

**Similar Projects:**
- MinIO (Go implementation)
- SeaweedFS
- Ceph RADOS Gateway

## Next Module

[Module 08: SQLite-like Database →](../module-08-database/)
