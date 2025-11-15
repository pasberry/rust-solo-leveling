use crate::engine::SharedEngine;
use crate::types::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use rust_decimal_macros::dec;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

pub struct ApiServer {
    engine: SharedEngine,
}

impl ApiServer {
    pub fn new(engine: SharedEngine) -> Self {
        ApiServer { engine }
    }

    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            .route("/api/v1/orders", post(place_order))
            .route("/api/v1/orders/:symbol/:id", delete(cancel_order))
            .route("/api/v1/market-data/:symbol", get(get_market_data))
            .route("/health", get(health_check))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }

    pub async fn serve(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("API server listening on {}", addr);
        axum::serve(listener, self.router()).await?;
        Ok(())
    }
}

async fn place_order(
    State(server): State<Arc<ApiServer>>,
    Json(req): Json<NewOrderRequest>,
) -> impl IntoResponse {
    let order = Order::new(
        req.symbol,
        req.side,
        req.order_type,
        req.quantity,
        req.price,
        req.client_order_id,
    );

    let order_id = order.id;
    let mut engine = server.engine.write().await;

    match engine.add_order(order) {
        Ok(trades) => {
            let order = engine
                .get_order(&trades.first().map(|t| t.symbol.clone()).unwrap_or_default(), order_id)
                .cloned();

            let response = OrderResponse {
                order_id,
                status: order.map(|o| o.status).unwrap_or(OrderStatus::New),
                filled_quantity: trades.iter().map(|t| t.quantity).sum(),
                trades,
            };

            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            tracing::error!("Failed to place order: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(OrderResponse {
                    order_id,
                    status: OrderStatus::Rejected,
                    filled_quantity: 0,
                    trades: vec![],
                }),
            )
        }
    }
}

async fn cancel_order(
    State(server): State<Arc<ApiServer>>,
    Path((symbol, order_id)): Path<(String, u64)>,
) -> impl IntoResponse {
    let mut engine = server.engine.write().await;

    match engine.cancel_order(&symbol, OrderId(order_id)) {
        Ok(order) => (StatusCode::OK, Json(order)),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(Order::new(
                symbol,
                Side::Buy,
                OrderType::Limit,
                0,
                Some(dec!(0)),
                String::new(),
            )),
        ),
    }
}

async fn get_market_data(
    State(server): State<Arc<ApiServer>>,
    Path(symbol): Path<String>,
) -> impl IntoResponse {
    let engine = server.engine.read().await;

    match engine.get_market_depth(&symbol, 10) {
        Ok(depth) => (StatusCode::OK, Json(depth)),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(MarketDepth {
                symbol,
                bids: vec![],
                asks: vec![],
                last_trade_price: None,
            }),
        ),
    }
}

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::MatchingEngine;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_api_server_creation() {
        let mut engine = MatchingEngine::new();
        engine.add_symbol("TEST".to_string());
        let engine = Arc::new(RwLock::new(engine));
        let _server = ApiServer::new(engine);
        // Server created successfully
    }
}
