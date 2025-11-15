use std::sync::Arc;
use tokio::sync::RwLock;
use trading_system::{api::ApiServer, engine::MatchingEngine};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trading_system=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting trading system");

    // Create matching engine and add symbols
    let mut engine = MatchingEngine::new();
    engine.add_symbol("AAPL".to_string());
    engine.add_symbol("GOOGL".to_string());
    engine.add_symbol("MSFT".to_string());
    engine.add_symbol("TSLA".to_string());

    let engine = Arc::new(RwLock::new(engine));

    // Start API server
    let server = ApiServer::new(engine);
    server.serve("127.0.0.1:8080").await?;

    Ok(())
}
