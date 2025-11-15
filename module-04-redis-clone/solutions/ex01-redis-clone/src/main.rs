mod command;
mod db;
mod error;
mod resp;
mod server;

use server::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "redis_clone=debug,info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Start server
    let server = Server::bind("127.0.0.1:6379").await?;
    tracing::info!("Server listening on 127.0.0.1:6379");
    tracing::info!("Compatible with redis-cli - try: redis-cli -p 6379");

    server.run().await?;

    Ok(())
}
