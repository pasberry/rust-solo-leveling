mod messages;
mod state;
mod websocket;

use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use messages::{Event, PublishRequest, StatsResponse};
use state::AppState;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "websocket_notifications=debug,tower_http=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/api/events", post(publish_event))
        .route("/api/stats", get(get_stats))
        .route("/health", get(health_check))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.unwrap();
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket::handle_websocket(socket, state))
}

async fn publish_event(
    State(state): State<AppState>,
    Json(payload): Json<PublishRequest>,
) -> StatusCode {
    let event = Event::new(payload.channel.clone(), payload.data);
    let message = event.to_server_message();
    state.broadcast_event(payload.channel, message).await;
    StatusCode::ACCEPTED
}

async fn get_stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let (active_connections, channels) = state.get_stats().await;
    Json(StatsResponse {
        active_connections,
        channels,
    })
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}
