# Module 06: Distributed Cache

**Build a Consistent Hash-Based Distributed Cache**

## Overview

Build a distributed caching system with:
- Consistent hashing for key distribution
- Multiple cache nodes
- Replication for fault tolerance
- Client library for routing
- Node health monitoring

**Duration**: 2-3 weeks (25-30 hours)

## What You'll Build

```rust
// Client usage
let client = CacheClient::connect(vec![
    "node1:6379",
    "node2:6379",
    "node3:6379",
]).await?;

client.set("user:123", user_data).await?;
let data = client.get("user:123").await?;
```

## Architecture

```
┌─────────────────────────────────────┐
│       Client Library                │
│   Consistent Hash Ring              │
└─────────┬───────────────────────────┘
          │ Routes requests
    ┌─────┴─────┬─────────┬───────┐
    │           │         │       │
┌───▼──┐    ┌──▼───┐  ┌──▼───┐ ┌▼────┐
│Node 1│◄──►│Node 2│◄►│Node 3│◄│Node4│
└──────┘    └──────┘  └──────┘ └─────┘
Replication for fault tolerance
```

## Key Components

### 1. Consistent Hash Ring

```rust
pub struct HashRing {
    nodes: BTreeMap<u64, NodeId>,
    virtual_nodes: usize,
}

impl HashRing {
    pub fn new(virtual_nodes: usize) -> Self {
        HashRing {
            nodes: BTreeMap::new(),
            virtual_nodes,
        }
    }

    pub fn add_node(&mut self, node: NodeId) {
        for i in 0..self.virtual_nodes {
            let hash = self.hash_virtual_node(&node, i);
            self.nodes.insert(hash, node.clone());
        }
    }

    pub fn remove_node(&mut self, node: &NodeId) {
        self.nodes.retain(|_, n| n != node);
    }

    pub fn get_node(&self, key: &str) -> Option<&NodeId> {
        let hash = self.hash_key(key);

        self.nodes
            .range(hash..)
            .next()
            .or_else(|| self.nodes.iter().next())
            .map(|(_, node)| node)
    }

    pub fn get_replicas(&self, key: &str, count: usize) -> Vec<NodeId> {
        let hash = self.hash_key(key);
        let mut replicas = Vec::new();
        let mut seen = HashSet::new();

        for (_, node) in self.nodes.range(hash..).chain(self.nodes.iter()) {
            if seen.insert(node.clone()) {
                replicas.push(node.clone());
                if replicas.len() >= count {
                    break;
                }
            }
        }

        replicas
    }

    fn hash_key(&self, key: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_virtual_node(&self, node: &NodeId, index: usize) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        format!("{}:{}", node, index).hash(&mut hasher);
        hasher.finish()
    }
}
```

### 2. Cache Node

```rust
pub struct CacheNode {
    id: NodeId,
    store: Arc<RwLock<HashMap<String, CacheEntry>>>,
    peers: Arc<RwLock<Vec<NodeId>>>,
    config: NodeConfig,
}

struct CacheEntry {
    value: Bytes,
    expires_at: Option<Instant>,
}

impl CacheNode {
    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let store = self.store.read().await;

        if let Some(entry) = store.get(key) {
            if entry.is_expired() {
                return Ok(None);
            }
            Ok(Some(entry.value.clone()))
        } else {
            Ok(None)
        }
    }

    pub async fn set(
        &self,
        key: String,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let expires_at = ttl.map(|d| Instant::now() + d);

        let entry = CacheEntry { value, expires_at };

        let mut store = self.store.write().await;
        store.insert(key.clone(), entry);

        // Replicate to peers
        self.replicate(key, value, ttl).await?;

        Ok(())
    }

    async fn replicate(
        &self,
        key: String,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> Result<()> {
        let peers = self.peers.read().await;

        for peer in peers.iter().take(self.config.replication_factor - 1) {
            // Send to peer (async, best-effort)
            tokio::spawn(async move {
                if let Err(e) = Self::send_to_peer(peer, key.clone(), value.clone(), ttl).await {
                    eprintln!("Replication failed to {}: {}", peer, e);
                }
            });
        }

        Ok(())
    }
}
```

### 3. Client Library

```rust
pub struct CacheClient {
    ring: Arc<RwLock<HashRing>>,
    connections: Arc<RwLock<HashMap<NodeId, Connection>>>,
    config: ClientConfig,
}

impl CacheClient {
    pub async fn connect(nodes: Vec<String>) -> Result<Self> {
        let mut ring = HashRing::new(150);  // 150 virtual nodes per physical node
        let mut connections = HashMap::new();

        for node in nodes {
            let node_id = NodeId::from(node);
            ring.add_node(node_id.clone());

            let conn = Connection::connect(&node_id).await?;
            connections.insert(node_id, conn);
        }

        Ok(CacheClient {
            ring: Arc::new(RwLock::new(ring)),
            connections: Arc::new(RwLock::new(connections)),
            config: ClientConfig::default(),
        })
    }

    pub async fn get(&self, key: &str) -> Result<Option<Bytes>> {
        let ring = self.ring.read().await;
        let node = ring.get_node(key).ok_or(Error::NoNodesAvailable)?;

        let connections = self.connections.read().await;
        let conn = connections.get(node).ok_or(Error::NodeNotConnected)?;

        conn.get(key).await
    }

    pub async fn set(&self, key: &str, value: Bytes) -> Result<()> {
        let ring = self.ring.read().await;
        let replicas = ring.get_replicas(key, self.config.replication_factor);

        let connections = self.connections.read().await;

        // Write to all replicas
        let mut futures = Vec::new();
        for node in replicas {
            if let Some(conn) = connections.get(&node) {
                futures.push(conn.set(key, value.clone()));
            }
        }

        // Wait for quorum
        let results: Vec<_> = futures_util::future::join_all(futures).await;
        let successes = results.iter().filter(|r| r.is_ok()).count();

        if successes >= self.config.write_quorum {
            Ok(())
        } else {
            Err(Error::QuorumNotReached)
        }
    }
}
```

## Implementation Roadmap

### Phase 1: Consistent Hash Ring (Days 1-2)
- Implement hash ring with virtual nodes
- Test key distribution
- Benchmark hash performance

### Phase 2: Single Cache Node (Days 3-4)
- Build in-memory cache node
- Add TTL support
- Implement eviction (LRU)

### Phase 3: Client Library (Days 5-6)
- Implement client with routing
- Connection pooling
- Retry logic

### Phase 4: Replication (Days 7-9)
- Add replication to nodes
- Implement quorum writes
- Handle node failures

### Phase 5: Health Monitoring (Days 10-11)
- Node health checks
- Automatic node removal
- Re-balancing on topology changes

### Phase 6: Production Features (Days 12-14)
- Metrics and monitoring
- Connection pooling
- Request batching
- Read-through/write-through caching

## Performance Targets

- **Latency**: <1ms p99 for gets
- **Throughput**: >100k ops/sec per node
- **Rebalancing**: <5% keys moved on node add/remove
- **Availability**: 99.9% with 3+ nodes

## Success Criteria

- ✅ Consistent hashing distributes keys evenly
- ✅ Replication provides fault tolerance
- ✅ Client routes to correct nodes
- ✅ Performance meets targets
- ✅ Handles node failures gracefully

## Next Module

[Module 07: S3-like Object Store →](../module-07-object-store/)
