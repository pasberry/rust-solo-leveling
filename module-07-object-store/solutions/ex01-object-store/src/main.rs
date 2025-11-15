mod error;
mod metadata;
mod storage;
mod store;

use store::ObjectStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "object_store=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting object store demo");

    // Create object store
    let store = ObjectStore::new("./data/objects", "sqlite:./data/metadata.db").await?;

    // Create buckets
    store.create_bucket("documents").await?;
    store.create_bucket("images").await?;
    tracing::info!("Created buckets");

    // Put some objects
    let doc1 = b"This is a text document";
    store
        .put_object(
            "documents",
            "readme.txt",
            &doc1[..],
            Some("text/plain".to_string()),
        )
        .await?;
    tracing::info!("Stored readme.txt");

    let doc2 = b"# Markdown Document\n\nThis is markdown content";
    store
        .put_object(
            "documents",
            "guide.md",
            &doc2[..],
            Some("text/markdown".to_string()),
        )
        .await?;
    tracing::info!("Stored guide.md");

    // List buckets
    let buckets = store.list_buckets().await?;
    tracing::info!("Buckets:");
    for bucket in buckets {
        tracing::info!("  - {}", bucket.name);
    }

    // List objects in documents bucket
    let objects = store.list_objects("documents", None).await?;
    tracing::info!("Objects in 'documents' bucket:");
    for obj in objects {
        tracing::info!(
            "  - {} ({} bytes, type: {})",
            obj.key,
            obj.size,
            obj.content_type.unwrap_or_else(|| "unknown".to_string())
        );
    }

    // Get an object
    let content = store.get_object("documents", "readme.txt").await?;
    tracing::info!(
        "Content of readme.txt: {}",
        String::from_utf8_lossy(&content)
    );

    // Copy object
    store
        .copy_object("documents", "readme.txt", "documents", "readme-copy.txt")
        .await?;
    tracing::info!("Copied readme.txt to readme-copy.txt");

    // Demonstrate deduplication
    let doc3 = b"This is a text document"; // Same as doc1
    let meta = store
        .put_object("documents", "duplicate.txt", &doc3[..], None)
        .await?;
    let original_meta = store.head_object("documents", "readme.txt").await?;
    tracing::info!(
        "Deduplication: original hash={}, duplicate hash={}",
        original_meta.content_hash,
        meta.content_hash
    );

    tracing::info!("Demo completed successfully");

    Ok(())
}
