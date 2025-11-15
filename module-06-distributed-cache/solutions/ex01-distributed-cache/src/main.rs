mod cache_node;
mod client;
mod error;
mod hash_ring;

use bytes::Bytes;
use cache_node::{CacheConfig, CacheNode};
use client::{CacheClient, ClientConfig};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "distributed_cache=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting distributed cache demo");

    // Create client with replication
    let client = CacheClient::new(ClientConfig {
        replication_factor: 3,
        write_quorum: 2,
        virtual_nodes: 150,
    });

    // Add 4 cache nodes
    for i in 1..=4 {
        let node = Arc::new(CacheNode::new(CacheConfig {
            max_entries: 1000,
            default_ttl: None,
        }));
        client.add_node(format!("node{}", i).into(), node).await;
        tracing::info!("Added node{}", i);
    }

    // Set some values
    for i in 0..10 {
        let key = format!("user:{}", i);
        let value = Bytes::from(format!("User data {}", i));
        client.set(&key, value).await?;
        tracing::info!("Set {}", key);
    }

    // Get values back
    for i in 0..10 {
        let key = format!("user:{}", i);
        if let Some(value) = client.get(&key).await? {
            tracing::info!("Got {}: {}", key, String::from_utf8_lossy(&value));
        }
    }

    // Show distribution
    tracing::info!("Total nodes: {}", client.node_count().await);

    tracing::info!("Demo completed successfully");

    Ok(())
}
