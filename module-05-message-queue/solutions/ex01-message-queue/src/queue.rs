use crate::error::Result;
use crate::log::LogStore;
use crate::message::{Message, MessageStatus};
use std::collections::VecDeque;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, info};

/// Configuration for a queue
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// Maximum messages in memory buffer
    pub buffer_size: usize,
    /// Maximum retry attempts before moving to DLQ
    pub max_retries: u32,
    /// Enable dead letter queue
    pub enable_dlq: bool,
}

impl Default for QueueConfig {
    fn default() -> Self {
        QueueConfig {
            buffer_size: 1000,
            max_retries: 3,
            enable_dlq: true,
        }
    }
}

/// A message queue with persistence
pub struct Queue {
    name: String,
    log: Arc<Mutex<LogStore>>,
    buffer: Arc<Mutex<VecDeque<Message>>>,
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    config: QueueConfig,
    dlq: Option<Arc<Mutex<VecDeque<Message>>>>,
}

impl Queue {
    /// Create or open a queue
    pub async fn open(name: impl Into<String>, data_dir: impl AsRef<Path>) -> Result<Self> {
        let name = name.into();
        let log_path = data_dir.as_ref().join(format!("{}.log", name));

        let mut log = LogStore::open(&log_path)?;

        // Recover pending messages
        let pending = log.recover()?;
        info!(
            "Queue '{}' opened with {} pending messages",
            name,
            pending.len()
        );

        let buffer = VecDeque::from(pending);

        Ok(Queue {
            name,
            log: Arc::new(Mutex::new(log)),
            buffer: Arc::new(Mutex::new(buffer)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            config: QueueConfig::default(),
            dlq: Some(Arc::new(Mutex::new(VecDeque::new()))),
        })
    }

    /// Create a queue with custom configuration
    pub async fn with_config(
        name: impl Into<String>,
        data_dir: impl AsRef<Path>,
        config: QueueConfig,
    ) -> Result<Self> {
        let mut queue = Self::open(name, data_dir).await?;
        queue.config = config;
        Ok(queue)
    }

    /// Publish a message to the queue
    pub async fn publish(&self, mut message: Message) -> Result<()> {
        message.queue = self.name.clone();

        // Write to persistent log
        {
            let mut log = self.log.lock().await;
            log.append(&message, MessageStatus::Pending)?;
        }

        // Add to in-memory buffer
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push_back(message.clone());
        }

        // Notify subscribers
        self.notify_subscribers(message).await;

        Ok(())
    }

    /// Subscribe to the queue
    pub async fn subscribe(&self, consumer_id: impl Into<String>) -> Result<Consumer> {
        let consumer_id = consumer_id.into();
        let (tx, rx) = mpsc::channel(self.config.buffer_size);

        // Send all buffered messages to new subscriber
        {
            let buffer = self.buffer.lock().await;
            for msg in buffer.iter() {
                let _ = tx.send(msg.clone()).await;
            }
        }

        let subscriber = Subscriber {
            id: consumer_id.clone(),
            sender: tx,
        };

        self.subscribers.write().await.push(subscriber);

        info!("Consumer '{}' subscribed to queue '{}'", consumer_id, self.name);

        Ok(Consumer {
            id: consumer_id,
            queue: self.name.clone(),
            receiver: rx,
            log: Arc::clone(&self.log),
            max_retries: self.config.max_retries,
        })
    }

    /// Notify all subscribers of a new message
    async fn notify_subscribers(&self, message: Message) {
        let subscribers = self.subscribers.read().await;

        // Round-robin distribution: send to first available subscriber
        for subscriber in subscribers.iter() {
            if let Ok(()) = subscriber.sender.try_send(message.clone()) {
                debug!(
                    "Delivered message {} to subscriber {}",
                    message.id, subscriber.id
                );
                return;
            }
        }

        debug!(
            "No subscribers available for message {} in queue '{}'",
            message.id, self.name
        );
    }

    /// Get the current queue depth (messages in buffer)
    pub async fn depth(&self) -> usize {
        self.buffer.lock().await.len()
    }

    /// Get the dead letter queue messages
    pub async fn get_dlq_messages(&self) -> Vec<Message> {
        if let Some(dlq) = &self.dlq {
            dlq.lock().await.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Compact the underlying log
    pub async fn compact(&self) -> Result<()> {
        let mut log = self.log.lock().await;
        log.compact()
    }
}

/// A subscriber to a queue
struct Subscriber {
    id: String,
    sender: mpsc::Sender<Message>,
}

/// A consumer that receives messages from a queue
pub struct Consumer {
    id: String,
    queue: String,
    receiver: mpsc::Receiver<Message>,
    log: Arc<Mutex<LogStore>>,
    max_retries: u32,
}

impl Consumer {
    /// Receive the next message
    pub async fn receive(&mut self) -> Result<Option<AckMessage>> {
        match self.receiver.recv().await {
            Some(message) => {
                debug!(
                    "Consumer '{}' received message {} from queue '{}'",
                    self.id, message.id, self.queue
                );

                // Mark as delivered in log
                {
                    let mut log = self.log.lock().await;
                    log.mark_delivered(&message.id)?;
                }

                Ok(Some(AckMessage {
                    message,
                    log: Arc::clone(&self.log),
                    max_retries: self.max_retries,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get the consumer ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// A message that can be acknowledged or rejected
pub struct AckMessage {
    message: Message,
    log: Arc<Mutex<LogStore>>,
    max_retries: u32,
}

impl AckMessage {
    /// Acknowledge successful processing
    pub async fn ack(self) -> Result<()> {
        debug!("Acknowledging message {}", self.message.id);

        let mut log = self.log.lock().await;
        log.mark_acked(&self.message.id)
    }

    /// Negative acknowledge - message failed processing
    pub async fn nack(mut self) -> Result<()> {
        self.message.increment_attempts();

        debug!(
            "Negative acknowledging message {} (attempts: {})",
            self.message.id, self.message.attempts
        );

        let mut log = self.log.lock().await;

        if self.message.attempts >= self.max_retries {
            info!(
                "Message {} exceeded max retries, moving to DLQ",
                self.message.id
            );
            log.append(&self.message, MessageStatus::DeadLettered)?;
        } else {
            // Requeue for retry
            log.mark_failed(&self.message.id)?;
        }

        Ok(())
    }

    /// Get the message payload
    pub fn payload(&self) -> &[u8] {
        &self.message.payload
    }

    /// Get the message ID
    pub fn id(&self) -> &str {
        &self.message.id
    }

    /// Get the full message
    pub fn message(&self) -> &Message {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_publish_receive() {
        let dir = tempdir().unwrap();
        let queue = Queue::open("test", dir.path()).await.unwrap();
        let mut consumer = queue.subscribe("c1").await.unwrap();

        let msg = Message::new("test", b"hello".to_vec());
        queue.publish(msg.clone()).await.unwrap();

        let received = consumer.receive().await.unwrap().unwrap();
        assert_eq!(received.payload(), b"hello");

        received.ack().await.unwrap();
    }

    #[tokio::test]
    async fn test_multiple_messages() {
        let dir = tempdir().unwrap();
        let queue = Queue::open("test", dir.path()).await.unwrap();
        let mut consumer = queue.subscribe("c1").await.unwrap();

        for i in 0..10 {
            let msg = Message::new("test", format!("msg{}", i).into_bytes());
            queue.publish(msg).await.unwrap();
        }

        for i in 0..10 {
            let msg = consumer.receive().await.unwrap().unwrap();
            assert_eq!(msg.payload(), format!("msg{}", i).as_bytes());
            msg.ack().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_ack_prevents_redelivery() {
        let dir = tempdir().unwrap();

        {
            let queue = Queue::open("test", dir.path()).await.unwrap();
            let mut consumer = queue.subscribe("c1").await.unwrap();

            let msg = Message::new("test", b"data".to_vec());
            queue.publish(msg).await.unwrap();

            let received = consumer.receive().await.unwrap().unwrap();
            received.ack().await.unwrap();
        }

        // Reopen queue
        {
            let queue = Queue::open("test", dir.path()).await.unwrap();
            let depth = queue.depth().await;

            // Should have no pending messages
            assert_eq!(depth, 0);
        }
    }


    #[tokio::test]
    async fn test_persistence_across_restarts() {
        let dir = tempdir().unwrap();

        {
            let queue = Queue::open("test", dir.path()).await.unwrap();

            for i in 0..5 {
                let msg = Message::new("test", format!("msg{}", i).into_bytes());
                queue.publish(msg).await.unwrap();
            }
        }

        // Reopen and verify messages are there
        {
            let queue = Queue::open("test", dir.path()).await.unwrap();
            let depth = queue.depth().await;
            assert_eq!(depth, 5);
        }
    }

}
