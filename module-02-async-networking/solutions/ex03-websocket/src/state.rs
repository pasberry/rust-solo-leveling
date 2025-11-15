use crate::messages::ServerMessage;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

pub type ClientId = Uuid;

#[derive(Clone)]
pub struct AppState {
    pub connections: Arc<RwLock<HashMap<ClientId, ClientInfo>>>,
    pub broadcast_tx: tokio::sync::broadcast::Sender<(String, ServerMessage)>,
}

pub struct ClientInfo {
    pub id: ClientId,
    pub subscriptions: HashSet<String>,
    pub tx: mpsc::UnboundedSender<ServerMessage>,
}

impl AppState {
    pub fn new() -> Self {
        let (broadcast_tx, _) = tokio::sync::broadcast::channel(1000);
        AppState {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
        }
    }

    pub async fn register_client(&self, id: ClientId, tx: mpsc::UnboundedSender<ServerMessage>) {
        let info = ClientInfo {
            id,
            subscriptions: HashSet::new(),
            tx,
        };
        self.connections.write().await.insert(id, info);
    }

    pub async fn unregister_client(&self, id: &ClientId) {
        self.connections.write().await.remove(id);
    }

    pub async fn subscribe(&self, client_id: &ClientId, channel: String) -> bool {
        let mut connections = self.connections.write().await;
        if let Some(client) = connections.get_mut(client_id) {
            client.subscriptions.insert(channel);
            true
        } else {
            false
        }
    }

    pub async fn unsubscribe(&self, client_id: &ClientId, channel: &str) -> bool {
        let mut connections = self.connections.write().await;
        if let Some(client) = connections.get_mut(client_id) {
            client.subscriptions.remove(channel);
            true
        } else {
            false
        }
    }

    pub async fn broadcast_event(&self, channel: String, message: ServerMessage) {
        let _ = self.broadcast_tx.send((channel, message));
    }

    pub async fn get_stats(&self) -> (usize, HashMap<String, usize>) {
        let connections = self.connections.read().await;
        let mut channel_counts: HashMap<String, usize> = HashMap::new();

        for client in connections.values() {
            for channel in &client.subscriptions {
                *channel_counts.entry(channel.clone()).or_insert(0) += 1;
            }
        }

        (connections.len(), channel_counts)
    }
}
