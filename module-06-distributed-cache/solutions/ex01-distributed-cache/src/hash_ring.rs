use siphasher::sip::SipHasher24;
use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};

/// A node identifier in the distributed cache
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(pub String);

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        NodeId(s.to_string())
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        NodeId(s)
    }
}

/// Consistent hash ring for distributing keys across nodes
pub struct HashRing {
    /// Virtual nodes mapped to their hash positions
    virtual_nodes: BTreeMap<u64, NodeId>,
    /// Number of virtual nodes per physical node
    replicas: usize,
    /// Set of all physical nodes
    nodes: HashSet<NodeId>,
}

impl HashRing {
    /// Create a new hash ring with the specified number of virtual nodes per node
    pub fn new(replicas: usize) -> Self {
        HashRing {
            virtual_nodes: BTreeMap::new(),
            replicas,
            nodes: HashSet::new(),
        }
    }

    /// Add a node to the hash ring
    pub fn add_node(&mut self, node: NodeId) {
        if self.nodes.contains(&node) {
            return;
        }

        for i in 0..self.replicas {
            let virtual_key = format!("{}:{}", node.0, i);
            let hash = self.hash(&virtual_key);
            self.virtual_nodes.insert(hash, node.clone());
        }

        self.nodes.insert(node);
    }

    /// Remove a node from the hash ring
    pub fn remove_node(&mut self, node: &NodeId) {
        if !self.nodes.contains(node) {
            return;
        }

        self.virtual_nodes.retain(|_, n| n != node);
        self.nodes.remove(node);
    }

    /// Get the primary node responsible for a key
    pub fn get_node(&self, key: &str) -> Option<&NodeId> {
        if self.virtual_nodes.is_empty() {
            return None;
        }

        let hash = self.hash(key);

        // Find the first node with hash >= key hash (clockwise on ring)
        self.virtual_nodes
            .range(hash..)
            .next()
            .or_else(|| self.virtual_nodes.iter().next())
            .map(|(_, node)| node)
    }

    /// Get N replica nodes for a key (including primary)
    pub fn get_replicas(&self, key: &str, count: usize) -> Vec<NodeId> {
        if self.virtual_nodes.is_empty() {
            return Vec::new();
        }

        let hash = self.hash(key);
        let mut replicas = Vec::new();
        let mut seen = HashSet::new();

        // Iterate clockwise around the ring starting from the hash
        for (_, node) in self
            .virtual_nodes
            .range(hash..)
            .chain(self.virtual_nodes.iter().map(|(h, n)| (h, n)))
        {
            if seen.insert(node.clone()) {
                replicas.push(node.clone());
                if replicas.len() >= count {
                    break;
                }
            }
        }

        replicas
    }

    /// Get all nodes in the ring
    pub fn nodes(&self) -> Vec<NodeId> {
        self.nodes.iter().cloned().collect()
    }

    /// Get the number of nodes
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the ring is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Hash a value using SipHash
    fn hash(&self, value: &str) -> u64 {
        let mut hasher = SipHasher24::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_remove_nodes() {
        let mut ring = HashRing::new(150);

        ring.add_node("node1".into());
        ring.add_node("node2".into());
        ring.add_node("node3".into());

        assert_eq!(ring.len(), 3);
        assert_eq!(ring.virtual_nodes.len(), 450); // 3 nodes * 150 replicas

        ring.remove_node(&"node2".into());
        assert_eq!(ring.len(), 2);
        assert_eq!(ring.virtual_nodes.len(), 300); // 2 nodes * 150 replicas
    }

    #[test]
    fn test_get_node() {
        let mut ring = HashRing::new(150);

        ring.add_node("node1".into());
        ring.add_node("node2".into());
        ring.add_node("node3".into());

        let node = ring.get_node("test-key");
        assert!(node.is_some());

        // Same key should always hash to same node
        let node1 = ring.get_node("test-key");
        let node2 = ring.get_node("test-key");
        assert_eq!(node1, node2);
    }

    #[test]
    fn test_get_replicas() {
        let mut ring = HashRing::new(150);

        ring.add_node("node1".into());
        ring.add_node("node2".into());
        ring.add_node("node3".into());

        let replicas = ring.get_replicas("test-key", 2);
        assert_eq!(replicas.len(), 2);

        // Replicas should be unique
        assert_ne!(replicas[0], replicas[1]);
    }

    #[test]
    fn test_distribution() {
        let mut ring = HashRing::new(150);

        ring.add_node("node1".into());
        ring.add_node("node2".into());
        ring.add_node("node3".into());

        let mut distribution = std::collections::HashMap::new();

        // Hash 10000 keys and check distribution
        for i in 0..10000 {
            let key = format!("key{}", i);
            if let Some(node) = ring.get_node(&key) {
                *distribution.entry(node.clone()).or_insert(0) += 1;
            }
        }

        // Each node should get roughly 33% of keys (within 10% variance)
        for (_, count) in distribution.iter() {
            let ratio = *count as f64 / 10000.0;
            assert!(ratio > 0.23 && ratio < 0.43, "Distribution: {}", ratio);
        }
    }

    #[test]
    fn test_minimal_disruption_on_node_change() {
        let mut ring = HashRing::new(150);

        ring.add_node("node1".into());
        ring.add_node("node2".into());
        ring.add_node("node3".into());

        // Record where keys map before change
        let mut before = std::collections::HashMap::new();
        for i in 0..1000 {
            let key = format!("key{}", i);
            if let Some(node) = ring.get_node(&key) {
                before.insert(key, node.clone());
            }
        }

        // Add a new node
        ring.add_node("node4".into());

        // Check how many keys moved
        let mut moved = 0;
        for i in 0..1000 {
            let key = format!("key{}", i);
            if let Some(node) = ring.get_node(&key) {
                if before.get(&key) != Some(node) {
                    moved += 1;
                }
            }
        }

        // With 4 nodes, ideally 25% of keys should move
        // Allow 15-35% range for variance
        let move_ratio = moved as f64 / 1000.0;
        assert!(
            move_ratio > 0.15 && move_ratio < 0.35,
            "Move ratio: {}",
            move_ratio
        );
    }
}
