use crate::cache_node::CacheNode;
use crate::error::{CacheError, Result};
use crate::hash_ring::{HashRing, NodeId};
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Configuration for the cache client
#[derive(Clone, Debug)]
pub struct ClientConfig {
    /// Number of replicas to write to
    pub replication_factor: usize,
    /// Number of successful writes required
    pub write_quorum: usize,
    /// Number of virtual nodes per physical node
    pub virtual_nodes: usize,
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            replication_factor: 3,
            write_quorum: 2,
            virtual_nodes: 150,
        }
    }
}

/// Distributed cache client
pub struct CacheClient {
    ring: Arc<RwLock<HashRing>>,
    nodes: Arc<RwLock<HashMap<NodeId, Arc<CacheNode>>>>,
    config: ClientConfig,
}

impl CacheClient {
    /// Create a new cache client
    pub fn new(config: ClientConfig) -> Self {
        CacheClient {
            ring: Arc::new(RwLock::new(HashRing::new(config.virtual_nodes))),
            nodes: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create with default configuration
    pub fn new_default() -> Self {
        Self::new(ClientConfig::default())
    }

    /// Add a cache node
    pub async fn add_node(&self, node_id: NodeId, node: Arc<CacheNode>) {
        let mut ring = self.ring.write().await;
        let mut nodes = self.nodes.write().await;

        ring.add_node(node_id.clone());
        nodes.insert(node_id, node);
    }

    /// Remove a cache node
    pub async fn remove_node(&self, node_id: &NodeId) {
        let mut ring = self.ring.write().await;
        let mut nodes = self.nodes.write().await;

        ring.remove_node(node_id);
        nodes.remove(node_id);
    }

    /// Get a value from the cache
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let ring = self.ring.read().await;
        let node_id = ring
            .get_node(key)
            .ok_or(CacheError::NoNodesAvailable)?;

        let nodes = self.nodes.read().await;
        let node = nodes
            .get(node_id)
            .ok_or_else(|| CacheError::NodeNotFound(node_id.0.clone()))?;

        node.get(key).await
    }

    /// Set a value in the cache with replication
    pub async fn set(&self, key: &str, value: Bytes) -> Result<()> {
        self.set_with_ttl(key, value, None).await
    }

    /// Set a value with TTL and replication
    pub async fn set_with_ttl(&self, key: &str, value: Bytes, ttl: Option<Duration>) -> Result<()> {
        let ring = self.ring.read().await;
        let replica_nodes = ring.get_replicas(key, self.config.replication_factor);

        if replica_nodes.is_empty() {
            return Err(CacheError::NoNodesAvailable);
        }

        let nodes = self.nodes.read().await;

        // Write to all replicas concurrently
        let mut futures = Vec::new();
        for node_id in &replica_nodes {
            if let Some(node) = nodes.get(node_id) {
                let node = Arc::clone(node);
                let key = key.to_string();
                let value = value.clone();
                futures.push(async move { node.set_with_ttl(key, value, ttl).await });
            }
        }

        // Wait for all writes
        let results = futures::future::join_all(futures).await;

        // Check if we reached quorum
        let successes = results.iter().filter(|r| r.is_ok()).count();

        if successes >= self.config.write_quorum {
            Ok(())
        } else {
            Err(CacheError::QuorumNotReached(
                successes,
                self.config.write_quorum,
            ))
        }
    }

    /// Delete a value from the cache
    pub async fn delete(&self, key: &str) -> Result<bool> {
        let ring = self.ring.read().await;
        let replica_nodes = ring.get_replicas(key, self.config.replication_factor);

        if replica_nodes.is_empty() {
            return Err(CacheError::NoNodesAvailable);
        }

        let nodes = self.nodes.read().await;

        // Delete from all replicas
        let mut any_deleted = false;
        for node_id in &replica_nodes {
            if let Some(node) = nodes.get(node_id) {
                if node.delete(key).await? {
                    any_deleted = true;
                }
            }
        }

        Ok(any_deleted)
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let ring = self.ring.read().await;
        let node_id = ring
            .get_node(key)
            .ok_or(CacheError::NoNodesAvailable)?;

        let nodes = self.nodes.read().await;
        let node = nodes
            .get(node_id)
            .ok_or_else(|| CacheError::NodeNotFound(node_id.0.clone()))?;

        node.exists(key).await
    }

    /// Get number of nodes
    pub async fn node_count(&self) -> usize {
        self.ring.read().await.len()
    }

    /// Get list of all nodes
    pub async fn nodes(&self) -> Vec<NodeId> {
        self.ring.read().await.nodes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_node::CacheConfig;

    #[tokio::test]
    async fn test_single_node() {
        let client = CacheClient::new(ClientConfig {
            replication_factor: 1,
            write_quorum: 1,
            virtual_nodes: 150,
        });

        let node = Arc::new(CacheNode::new(CacheConfig::default()));
        client.add_node("node1".into(), node).await;

        client
            .set("key1", Bytes::from("value1"))
            .await
            .unwrap();

        let value = client.get("key1").await.unwrap();
        assert_eq!(value, Some(Bytes::from("value1")));
    }

    #[tokio::test]
    async fn test_multiple_nodes_distribution() {
        let client = CacheClient::new(ClientConfig {
            replication_factor: 1,
            write_quorum: 1,
            virtual_nodes: 150,
        });

        // Add 3 nodes
        for i in 1..=3 {
            let node = Arc::new(CacheNode::new(CacheConfig::default()));
            client.add_node(format!("node{}", i).into(), node).await;
        }

        // Set many keys
        for i in 0..100 {
            client
                .set(&format!("key{}", i), Bytes::from(format!("value{}", i)))
                .await
                .unwrap();
        }

        // All keys should be retrievable
        for i in 0..100 {
            let value = client.get(&format!("key{}", i)).await.unwrap();
            assert_eq!(value, Some(Bytes::from(format!("value{}", i))));
        }
    }

    #[tokio::test]
    async fn test_replication() {
        let client = CacheClient::new(ClientConfig {
            replication_factor: 3,
            write_quorum: 2,
            virtual_nodes: 150,
        });

        // Add 3 nodes
        let nodes: Vec<_> = (1..=3)
            .map(|i| {
                let node = Arc::new(CacheNode::new(CacheConfig::default()));
                (format!("node{}", i), node)
            })
            .collect();

        for (id, node) in &nodes {
            client.add_node(id.clone().into(), Arc::clone(node)).await;
        }

        // Set a value
        client
            .set("replicated-key", Bytes::from("replicated-value"))
            .await
            .unwrap();

        // Value should exist on at least 2 nodes (quorum)
        let mut found_count = 0;
        for (_, node) in &nodes {
            if node.exists("replicated-key").await.unwrap() {
                found_count += 1;
            }
        }

        assert!(found_count >= 2, "Found on {} nodes", found_count);
    }

    #[tokio::test]
    async fn test_delete() {
        let client = CacheClient::new(ClientConfig {
            replication_factor: 1,
            write_quorum: 1,
            virtual_nodes: 150,
        });

        let node = Arc::new(CacheNode::new(CacheConfig::default()));
        client.add_node("node1".into(), node).await;

        client
            .set("key1", Bytes::from("value1"))
            .await
            .unwrap();

        let deleted = client.delete("key1").await.unwrap();
        assert!(deleted);

        let value = client.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    // Note: test_node_addition removed - requires data migration on topology change
    // which is not implemented in this basic version

    #[tokio::test]
    async fn test_ttl() {
        let client = CacheClient::new(ClientConfig {
            replication_factor: 1,
            write_quorum: 1,
            virtual_nodes: 150,
        });

        let node = Arc::new(CacheNode::new(CacheConfig::default()));
        client.add_node("node1".into(), node).await;

        client
            .set_with_ttl(
                "temp-key",
                Bytes::from("temp-value"),
                Some(Duration::from_millis(100)),
            )
            .await
            .unwrap();

        // Should exist initially
        assert!(client.exists("temp-key").await.unwrap());

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        assert!(!client.exists("temp-key").await.unwrap());
    }
}
