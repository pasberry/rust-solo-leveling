mod error;
mod log;
mod message;
mod queue;

use queue::Queue;
use message::Message;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "message_queue=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    std::fs::create_dir_all(&data_dir)?;

    tracing::info!("Starting message queue demo");

    // Create a queue
    let queue = Arc::new(Queue::open("orders", &data_dir).await?);
    tracing::info!("Queue 'orders' opened");

    // Spawn a consumer
    let consumer_queue = Arc::clone(&queue);
    let consumer_handle = tokio::spawn(async move {
        let mut consumer = consumer_queue.subscribe("worker-1").await.unwrap();
        tracing::info!("Consumer 'worker-1' started");

        while let Ok(Some(msg)) = consumer.receive().await {
            let payload = String::from_utf8_lossy(msg.payload());
            tracing::info!("Consumer received message: {}", payload);

            // Simulate processing
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            // Acknowledge message
            msg.ack().await.unwrap();
            tracing::info!("Message acknowledged");
        }
    });

    // Wait for consumer to be ready
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Publish some messages
    for i in 0..5 {
        let msg = Message::new("orders", format!("Order #{}", i + 1).into_bytes());
        queue.publish(msg).await?;
        tracing::info!("Published order #{}", i + 1);
    }

    // Wait for processing
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    tracing::info!("Queue depth: {}", queue.depth().await);

    // Gracefully shutdown
    drop(queue);
    consumer_handle.abort();

    tracing::info!("Demo completed");

    Ok(())
}
