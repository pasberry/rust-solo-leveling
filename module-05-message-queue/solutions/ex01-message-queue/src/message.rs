use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// A message in the queue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Unique message ID
    pub id: String,

    /// Queue name this message belongs to
    pub queue: String,

    /// Message payload (raw bytes)
    pub payload: Vec<u8>,

    /// Unix timestamp when message was created (milliseconds)
    pub created_at: u64,

    /// Number of delivery attempts
    pub attempts: u32,

    /// Optional metadata for routing, filtering, etc.
    pub metadata: HashMap<String, String>,
}

impl Message {
    /// Create a new message
    pub fn new(queue: impl Into<String>, payload: Vec<u8>) -> Self {
        Message {
            id: Uuid::new_v4().to_string(),
            queue: queue.into(),
            payload,
            created_at: current_timestamp(),
            attempts: 0,
            metadata: HashMap::new(),
        }
    }

    /// Create a message with metadata
    pub fn with_metadata(
        queue: impl Into<String>,
        payload: Vec<u8>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Message {
            id: Uuid::new_v4().to_string(),
            queue: queue.into(),
            payload,
            created_at: current_timestamp(),
            attempts: 0,
            metadata,
        }
    }

    /// Increment the attempt counter
    pub fn increment_attempts(&mut self) {
        self.attempts += 1;
    }
}

/// Status of a message in the log
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageStatus {
    /// Message is pending delivery
    Pending,

    /// Message has been delivered to a consumer
    Delivered,

    /// Message was successfully acknowledged
    Acknowledged,

    /// Message delivery failed
    Failed,

    /// Message moved to dead letter queue
    DeadLettered,
}

/// A log entry combining message and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub message: Message,
    pub status: MessageStatus,
    pub updated_at: u64,
}

impl LogEntry {
    pub fn new(message: Message, status: MessageStatus) -> Self {
        LogEntry {
            message,
            status,
            updated_at: current_timestamp(),
        }
    }

    pub fn with_status(mut self, status: MessageStatus) -> Self {
        self.status = status;
        self.updated_at = current_timestamp();
        self
    }
}

/// Get current Unix timestamp in milliseconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new("test-queue", b"hello".to_vec());

        assert_eq!(msg.queue, "test-queue");
        assert_eq!(msg.payload, b"hello");
        assert_eq!(msg.attempts, 0);
        assert!(!msg.id.is_empty());
        assert!(msg.created_at > 0);
    }

    #[test]
    fn test_message_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("priority".to_string(), "high".to_string());

        let msg = Message::with_metadata("test-queue", b"data".to_vec(), metadata);

        assert_eq!(msg.metadata.get("priority"), Some(&"high".to_string()));
    }

    #[test]
    fn test_increment_attempts() {
        let mut msg = Message::new("test", b"data".to_vec());
        assert_eq!(msg.attempts, 0);

        msg.increment_attempts();
        assert_eq!(msg.attempts, 1);

        msg.increment_attempts();
        assert_eq!(msg.attempts, 2);
    }

    #[test]
    fn test_log_entry() {
        let msg = Message::new("test", b"data".to_vec());
        let entry = LogEntry::new(msg.clone(), MessageStatus::Pending);

        assert_eq!(entry.message.id, msg.id);
        assert_eq!(entry.status, MessageStatus::Pending);

        let acked = entry.with_status(MessageStatus::Acknowledged);
        assert_eq!(acked.status, MessageStatus::Acknowledged);
    }
}
