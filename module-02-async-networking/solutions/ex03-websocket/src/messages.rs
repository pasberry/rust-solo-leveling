use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ClientMessage {
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ServerMessage {
    Event {
        channel: String,
        data: serde_json::Value,
    },
    Subscribed {
        channel: String,
    },
    Unsubscribed {
        channel: String,
    },
    Error {
        message: String,
    },
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub channel: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

impl Event {
    pub fn new(channel: String, data: serde_json::Value) -> Self {
        Event {
            id: Uuid::new_v4(),
            channel,
            timestamp: Utc::now(),
            data,
        }
    }

    pub fn to_server_message(&self) -> ServerMessage {
        ServerMessage::Event {
            channel: self.channel.clone(),
            data: self.data.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub channel: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub active_connections: usize,
    pub channels: HashMap<String, usize>,
}
