use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

static ORDER_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static TRADE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub u64);

impl OrderId {
    pub fn new() -> Self {
        OrderId(ORDER_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TradeId(pub u64);

impl TradeId {
    pub fn new() -> Self {
        TradeId(TRADE_ID_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
    pub price: Option<Decimal>,
    pub filled_quantity: u64,
    pub status: OrderStatus,
    #[serde(skip, default = "SystemTime::now")]
    pub timestamp: SystemTime,
    pub client_order_id: String,
}

impl Order {
    pub fn new(
        symbol: String,
        side: Side,
        order_type: OrderType,
        quantity: u64,
        price: Option<Decimal>,
        client_order_id: String,
    ) -> Self {
        Order {
            id: OrderId::new(),
            symbol,
            side,
            order_type,
            quantity,
            price,
            filled_quantity: 0,
            status: OrderStatus::New,
            timestamp: SystemTime::now(),
            client_order_id,
        }
    }

    pub fn remaining_quantity(&self) -> u64 {
        self.quantity.saturating_sub(self.filled_quantity)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    pub id: TradeId,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: u64,
    pub buyer_order_id: OrderId,
    pub seller_order_id: OrderId,
    #[serde(skip, default = "SystemTime::now")]
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: u64,
    pub order_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDepth {
    pub symbol: String,
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub last_trade_price: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderRequest {
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
    pub price: Option<Decimal>,
    pub client_order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub order_id: OrderId,
    pub status: OrderStatus,
    pub filled_quantity: u64,
    pub trades: Vec<Trade>,
}
