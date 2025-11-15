# Module 10: Stock Trading System (Capstone)

**Build a High-Performance Stock Trading System**

## Overview

Build a complete trading system with:
- Order book with price-time priority matching
- Order gateway (REST + WebSocket)
- Risk management engine
- Market data feed handler
- Event-driven architecture
- Real-time matching engine
- Position tracking and P&L

**Duration**: 4-5 weeks (45-50 hours)

**Difficulty**: ⭐⭐⭐⭐⭐ (Capstone - Integrates Everything)

## What You'll Build

```rust
// Client usage
let client = TradingClient::connect("ws://localhost:8080").await?;

// Subscribe to market data
client.subscribe_market_data("AAPL").await?;

// Place order
let order = NewOrder {
    symbol: "AAPL".to_string(),
    side: Side::Buy,
    order_type: OrderType::Limit,
    quantity: 100,
    price: Some(150.50),
    client_order_id: "order-123".to_string(),
};

let response = client.place_order(order).await?;
println!("Order placed: {:?}", response);

// Stream live updates
while let Some(update) = client.next_update().await {
    match update {
        Update::Trade(trade) => println!("Trade: {:?}", trade),
        Update::OrderUpdate(order) => println!("Order: {:?}", order),
        Update::MarketData(quote) => println!("Quote: {:?}", quote),
    }
}
```

## Architecture

```
┌──────────────────────────────────────────────────┐
│              Client Applications                 │
│        (REST API + WebSocket)                    │
└────────────────┬─────────────────────────────────┘
                 │
┌────────────────▼─────────────────────────────────┐
│           Order Gateway                          │
│  - Authentication                                │
│  - Order validation                              │
│  - Rate limiting                                 │
└────────────────┬─────────────────────────────────┘
                 │
    ┌────────────┼─────────────┐
    │            │             │
┌───▼────┐  ┌───▼────┐  ┌────▼────┐
│  Risk  │  │Matching│  │ Market  │
│ Engine │◄─┤ Engine │─►│  Data   │
└────────┘  └───┬────┘  └─────────┘
                │
         ┌──────┴──────┐
         │             │
    ┌────▼───┐    ┌───▼────┐
    │Position│    │ Event  │
    │Tracker │    │  Bus   │
    └────────┘    └────────┘
```

## Core Components

### 1. Order Book (Matching Engine)

```rust
use std::collections::{BTreeMap, HashMap, VecDeque};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct OrderId(pub u64);

#[derive(Debug, Clone)]
pub struct Order {
    pub id: OrderId,
    pub symbol: String,
    pub side: Side,
    pub order_type: OrderType,
    pub quantity: u64,
    pub price: Option<Decimal>,  // None for market orders
    pub filled_quantity: u64,
    pub status: OrderStatus,
    pub timestamp: std::time::Instant,
    pub client_order_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderStatus {
    New,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

pub struct OrderBook {
    symbol: String,
    bids: BTreeMap<Decimal, VecDeque<Order>>,  // Price → Orders (highest first)
    asks: BTreeMap<Decimal, VecDeque<Order>>,  // Price → Orders (lowest first)
    orders: HashMap<OrderId, Order>,
    last_price: Option<Decimal>,
}

impl OrderBook {
    pub fn new(symbol: String) -> Self {
        OrderBook {
            symbol,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            orders: HashMap::new(),
            last_price: None,
        }
    }

    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Match market orders immediately
        if order.order_type == OrderType::Market {
            trades.extend(self.match_market_order(&mut order));
        } else {
            trades.extend(self.match_limit_order(&mut order));
        }

        // Add remaining quantity to book
        if order.quantity > order.filled_quantity {
            self.insert_order(order.clone());
        }

        self.orders.insert(order.id, order);
        trades
    }

    fn match_limit_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let price = order.price.expect("Limit order must have price");

        let opposite_side = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        // For buy orders: match with asks at or below our price
        // For sell orders: match with bids at or above our price
        let can_match = |book_price: &Decimal| match order.side {
            Side::Buy => book_price <= &price,
            Side::Sell => book_price >= &price,
        };

        let mut prices_to_remove = Vec::new();

        for (book_price, level_orders) in opposite_side.iter_mut() {
            if !can_match(book_price) {
                break;  // No more matches possible
            }

            while let Some(mut passive_order) = level_orders.pop_front() {
                let trade = self.execute_trade(order, &mut passive_order, *book_price);
                trades.push(trade);

                if passive_order.filled_quantity < passive_order.quantity {
                    level_orders.push_front(passive_order);
                    break;
                }

                if order.filled_quantity >= order.quantity {
                    break;
                }
            }

            if level_orders.is_empty() {
                prices_to_remove.push(*book_price);
            }

            if order.filled_quantity >= order.quantity {
                break;
            }
        }

        // Clean up empty price levels
        for price in prices_to_remove {
            opposite_side.remove(&price);
        }

        trades
    }

    fn match_market_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        let opposite_side = match order.side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let mut prices_to_remove = Vec::new();

        for (book_price, level_orders) in opposite_side.iter_mut() {
            while let Some(mut passive_order) = level_orders.pop_front() {
                let trade = self.execute_trade(order, &mut passive_order, *book_price);
                trades.push(trade);

                if passive_order.filled_quantity < passive_order.quantity {
                    level_orders.push_front(passive_order);
                    break;
                }

                if order.filled_quantity >= order.quantity {
                    break;
                }
            }

            if level_orders.is_empty() {
                prices_to_remove.push(*book_price);
            }

            if order.filled_quantity >= order.quantity {
                break;
            }
        }

        for price in prices_to_remove {
            opposite_side.remove(&price);
        }

        trades
    }

    fn execute_trade(&mut self, aggressive: &mut Order, passive: &mut Order, price: Decimal) -> Trade {
        let quantity = std::cmp::min(
            aggressive.quantity - aggressive.filled_quantity,
            passive.quantity - passive.filled_quantity,
        );

        aggressive.filled_quantity += quantity;
        passive.filled_quantity += quantity;

        // Update order statuses
        if aggressive.filled_quantity >= aggressive.quantity {
            aggressive.status = OrderStatus::Filled;
        } else {
            aggressive.status = OrderStatus::PartiallyFilled;
        }

        if passive.filled_quantity >= passive.quantity {
            passive.status = OrderStatus::Filled;
        } else {
            passive.status = OrderStatus::PartiallyFilled;
        }

        self.last_price = Some(price);

        Trade {
            id: TradeId::new(),
            symbol: self.symbol.clone(),
            price,
            quantity,
            buyer_order_id: match aggressive.side {
                Side::Buy => aggressive.id,
                Side::Sell => passive.id,
            },
            seller_order_id: match aggressive.side {
                Side::Sell => aggressive.id,
                Side::Buy => passive.id,
            },
            timestamp: std::time::Instant::now(),
        }
    }

    fn insert_order(&mut self, order: Order) {
        let price = order.price.expect("Cannot insert order without price");
        let side = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        side.entry(price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<(), Error> {
        let order = self.orders.get_mut(&order_id)
            .ok_or(Error::OrderNotFound)?;

        if order.status == OrderStatus::Filled {
            return Err(Error::OrderAlreadyFilled);
        }

        order.status = OrderStatus::Canceled;

        let price = order.price.ok_or(Error::InvalidOrder)?;
        let side = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        if let Some(level) = side.get_mut(&price) {
            level.retain(|o| o.id != order_id);
            if level.is_empty() {
                side.remove(&price);
            }
        }

        Ok(())
    }

    pub fn get_best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    pub fn get_best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    pub fn get_spread(&self) -> Option<Decimal> {
        match (self.get_best_bid(), self.get_best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn get_depth(&self, levels: usize) -> MarketDepth {
        let bids: Vec<_> = self.bids.iter()
            .rev()
            .take(levels)
            .map(|(price, orders)| {
                let quantity: u64 = orders.iter()
                    .map(|o| o.quantity - o.filled_quantity)
                    .sum();
                PriceLevel { price: *price, quantity }
            })
            .collect();

        let asks: Vec<_> = self.asks.iter()
            .take(levels)
            .map(|(price, orders)| {
                let quantity: u64 = orders.iter()
                    .map(|o| o.quantity - o.filled_quantity)
                    .sum();
                PriceLevel { price: *price, quantity }
            })
            .collect();

        MarketDepth { bids, asks }
    }
}

#[derive(Debug, Clone)]
pub struct Trade {
    pub id: TradeId,
    pub symbol: String,
    pub price: Decimal,
    pub quantity: u64,
    pub buyer_order_id: OrderId,
    pub seller_order_id: OrderId,
    pub timestamp: std::time::Instant,
}

#[derive(Debug, Clone)]
pub struct MarketDepth {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
}

#[derive(Debug, Clone)]
pub struct PriceLevel {
    pub price: Decimal,
    pub quantity: u64,
}
```

### 2. Risk Management Engine

```rust
use std::collections::HashMap;

pub struct RiskEngine {
    accounts: HashMap<String, Account>,
    limits: RiskLimits,
}

#[derive(Debug, Clone)]
pub struct Account {
    pub account_id: String,
    pub cash_balance: Decimal,
    pub positions: HashMap<String, Position>,
    pub open_orders: Vec<OrderId>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub quantity: i64,  // Can be negative for short positions
    pub avg_price: Decimal,
    pub realized_pnl: Decimal,
}

#[derive(Debug, Clone)]
pub struct RiskLimits {
    pub max_order_value: Decimal,
    pub max_position_size: u64,
    pub max_daily_loss: Decimal,
    pub max_leverage: Decimal,
}

impl RiskEngine {
    pub fn validate_order(&self, account_id: &str, order: &Order, price: Decimal) -> Result<(), RiskError> {
        let account = self.accounts.get(account_id)
            .ok_or(RiskError::AccountNotFound)?;

        // Check order value
        let order_value = price * Decimal::from(order.quantity);
        if order_value > self.limits.max_order_value {
            return Err(RiskError::OrderValueExceeded);
        }

        // Check buying power
        if order.side == Side::Buy {
            if order_value > account.cash_balance {
                return Err(RiskError::InsufficientFunds);
            }
        }

        // Check position limits
        if let Some(position) = account.positions.get(&order.symbol) {
            let new_quantity = match order.side {
                Side::Buy => position.quantity + order.quantity as i64,
                Side::Sell => position.quantity - order.quantity as i64,
            };

            if new_quantity.abs() as u64 > self.limits.max_position_size {
                return Err(RiskError::PositionLimitExceeded);
            }
        }

        Ok(())
    }

    pub fn update_position(&mut self, account_id: &str, trade: &Trade) -> Result<(), RiskError> {
        let account = self.accounts.get_mut(account_id)
            .ok_or(RiskError::AccountNotFound)?;

        let position = account.positions
            .entry(trade.symbol.clone())
            .or_insert(Position {
                symbol: trade.symbol.clone(),
                quantity: 0,
                avg_price: Decimal::ZERO,
                realized_pnl: Decimal::ZERO,
            });

        // Determine if buy or sell based on order IDs
        let is_buyer = trade.buyer_order_id == /* check if account's order */;
        let trade_value = trade.price * Decimal::from(trade.quantity);

        if is_buyer {
            // Update average price for long position
            let total_cost = position.avg_price * Decimal::from(position.quantity.abs())
                + trade_value;
            position.quantity += trade.quantity as i64;
            position.avg_price = total_cost / Decimal::from(position.quantity.abs());

            account.cash_balance -= trade_value;
        } else {
            // Selling - realize P&L
            let pnl = (trade.price - position.avg_price) * Decimal::from(trade.quantity);
            position.realized_pnl += pnl;
            position.quantity -= trade.quantity as i64;

            account.cash_balance += trade_value;
        }

        Ok(())
    }

    pub fn calculate_unrealized_pnl(&self, account_id: &str, market_prices: &HashMap<String, Decimal>) -> Decimal {
        let account = match self.accounts.get(account_id) {
            Some(acc) => acc,
            None => return Decimal::ZERO,
        };

        let mut total_pnl = Decimal::ZERO;

        for position in account.positions.values() {
            if let Some(&market_price) = market_prices.get(&position.symbol) {
                let pnl = (market_price - position.avg_price) * Decimal::from(position.quantity);
                total_pnl += pnl;
            }
        }

        total_pnl
    }
}
```

### 3. Order Gateway (WebSocket + REST)

```rust
use axum::{
    Router,
    routing::{get, post},
    extract::{State, WebSocketUpgrade, ws::WebSocket},
    response::IntoResponse,
    Json,
};
use tokio::sync::mpsc;

pub struct OrderGateway {
    matching_engine: Arc<Mutex<MatchingEngine>>,
    risk_engine: Arc<Mutex<RiskEngine>>,
    event_bus: Arc<EventBus>,
}

impl OrderGateway {
    pub fn router(self) -> Router {
        let state = Arc::new(self);

        Router::new()
            .route("/api/v1/orders", post(place_order))
            .route("/api/v1/orders/:id", delete(cancel_order))
            .route("/api/v1/positions", get(get_positions))
            .route("/api/v1/market-data/:symbol", get(get_market_data))
            .route("/ws", get(websocket_handler))
            .with_state(state)
    }

    pub async fn serve(self, addr: &str) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, self.router()).await?;
        Ok(())
    }
}

async fn place_order(
    State(gateway): State<Arc<OrderGateway>>,
    Json(new_order): Json<NewOrderRequest>,
) -> Result<Json<OrderResponse>, StatusCode> {
    // Validate order
    let order = new_order.into_order();

    // Risk check
    let price = order.price.unwrap_or_else(|| {
        // Get current market price
        gateway.matching_engine.lock().await
            .get_book(&order.symbol)
            .and_then(|book| match order.side {
                Side::Buy => book.get_best_ask(),
                Side::Sell => book.get_best_bid(),
            })
            .unwrap_or(Decimal::ZERO)
    });

    gateway.risk_engine.lock().await
        .validate_order(&new_order.account_id, &order, price)
        .map_err(|_| StatusCode::FORBIDDEN)?;

    // Submit to matching engine
    let trades = gateway.matching_engine.lock().await
        .add_order(order.clone());

    // Publish events
    for trade in trades {
        gateway.event_bus.publish(Event::Trade(trade)).await;
    }

    gateway.event_bus.publish(Event::OrderUpdate(order.clone())).await;

    Ok(Json(OrderResponse {
        order_id: order.id,
        status: order.status,
    }))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(gateway): State<Arc<OrderGateway>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, gateway))
}

async fn handle_websocket(socket: WebSocket, gateway: Arc<OrderGateway>) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel::<Event>(100);

    // Subscribe to events
    gateway.event_bus.subscribe(tx).await;

    // Send events to client
    let send_task = tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let message = serde_json::to_string(&event).unwrap();
            if sender.send(axum::extract::ws::Message::Text(message)).await.is_err() {
                break;
            }
        }
    });

    // Receive commands from client
    let receive_task = tokio::spawn(async move {
        while let Some(Ok(message)) = receiver.next().await {
            if let axum::extract::ws::Message::Text(text) = message {
                // Handle client commands (subscribe, unsubscribe, etc.)
                println!("Received: {}", text);
            }
        }
    });

    tokio::select! {
        _ = send_task => {},
        _ = receive_task => {},
    }
}
```

### 4. Event Bus

```rust
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum Event {
    OrderUpdate(Order),
    Trade(Trade),
    MarketData(Quote),
    PositionUpdate(Position),
}

pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(10000);
        EventBus { sender }
    }

    pub async fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    pub async fn subscribe(&self, tx: mpsc::Sender<Event>) {
        let mut rx = self.sender.subscribe();

        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if tx.send(event).await.is_err() {
                    break;
                }
            }
        });
    }
}
```

## Implementation Roadmap

### Phase 1: Order Book & Matching Engine (Days 1-7)
- Implement price-time priority order book
- Add market and limit order matching
- Order lifecycle management
- Test with synthetic orders

**Success criteria:**
- Correctly matches orders with price-time priority
- Handles partial fills
- Maintains book integrity

### Phase 2: REST API (Days 8-11)
- Build REST endpoints for orders
- Position and account queries
- Market data API
- Error handling and validation

**Success criteria:**
- Can place/cancel orders via REST
- Query positions and market data
- Proper HTTP status codes

### Phase 3: WebSocket Streaming (Days 12-15)
- WebSocket connection handling
- Real-time order updates
- Trade stream
- Market data subscriptions

**Success criteria:**
- Low-latency updates (<10ms)
- Handle 1000+ concurrent connections
- Graceful reconnection

### Phase 4: Risk Management (Days 16-21)
- Pre-trade risk checks
- Position tracking
- P&L calculation
- Account management

**Success criteria:**
- Blocks risky orders
- Accurate position tracking
- Real-time P&L updates

### Phase 5: Event Bus & Integration (Days 22-26)
- Event-driven architecture
- Event sourcing for audit trail
- Component integration
- End-to-end testing

**Success criteria:**
- All components communicate via events
- Full audit trail
- System works end-to-end

### Phase 6: Market Data Feed (Days 27-32)
- Market data aggregation
- Order book snapshots
- Level 2 data (market depth)
- Historical data storage

**Success criteria:**
- Accurate market data
- Low-latency updates
- Historical replay

### Phase 7: Performance & Production (Days 33-45)
- Benchmarking and profiling
- Optimize critical paths
- Load testing (10k+ orders/sec)
- Monitoring and metrics
- Documentation

## Performance Targets

- **Order latency**: <1ms (p99)
- **Throughput**: >10,000 orders/sec
- **WebSocket latency**: <5ms
- **Concurrent connections**: >1,000
- **Market data updates**: >100k/sec

## Success Criteria

- ✅ Order book with correct matching logic
- ✅ REST API for trading operations
- ✅ WebSocket for real-time updates
- ✅ Risk management and validation
- ✅ Position tracking and P&L
- ✅ Event-driven architecture
- ✅ Handles high throughput (>10k ops/sec)
- ✅ Comprehensive testing and monitoring

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_order_matching() {
        let mut book = OrderBook::new("AAPL".to_string());

        // Place buy order
        let buy = Order {
            id: OrderId(1),
            symbol: "AAPL".to_string(),
            side: Side::Buy,
            order_type: OrderType::Limit,
            quantity: 100,
            price: Some(dec!(150.00)),
            filled_quantity: 0,
            status: OrderStatus::New,
            timestamp: Instant::now(),
            client_order_id: "buy-1".to_string(),
        };

        let trades = book.add_order(buy);
        assert_eq!(trades.len(), 0);  // No match

        // Place matching sell order
        let sell = Order {
            id: OrderId(2),
            symbol: "AAPL".to_string(),
            side: Side::Sell,
            order_type: OrderType::Limit,
            quantity: 50,
            price: Some(dec!(150.00)),
            filled_quantity: 0,
            status: OrderStatus::New,
            timestamp: Instant::now(),
            client_order_id: "sell-1".to_string(),
        };

        let trades = book.add_order(sell);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 50);
        assert_eq!(trades[0].price, dec!(150.00));
    }

    #[tokio::test]
    async fn test_risk_limits() {
        let mut risk_engine = RiskEngine::new();

        // Test order value limit
        // Test position size limit
        // Test buying power
    }

    #[tokio::test]
    async fn test_full_trading_flow() {
        // Start gateway
        // Connect WebSocket client
        // Place orders
        // Verify trades
        // Check positions
    }
}
```

## Extensions & Variations

**After completing the core system, try:**

1. **Advanced order types**:
   - Stop-loss orders
   - Iceberg orders
   - Time-in-force (FOK, IOC, GTC)

2. **Market making bot**:
   - Automated market maker
   - Quote both sides
   - Inventory management

3. **Historical data & backtesting**:
   - Store all trades and quotes
   - Replay historical data
   - Backtest strategies

4. **Multi-asset support**:
   - FX pairs
   - Crypto currencies
   - Derivatives

5. **Advanced risk**:
   - VaR calculations
   - Stress testing
   - Real-time margin requirements

## Resources

**Trading Systems:**
- "Trading and Exchanges" by Larry Harris
- "Building Winning Algorithmic Trading Systems" by Kevin Davey
- Jane Street tech talks on market making

**Architecture:**
- Event sourcing patterns
- CQRS (Command Query Responsibility Segregation)
- Low-latency system design

**Rust Performance:**
- "The Rust Performance Book"
- Lock-free data structures
- SIMD optimizations

**Similar Systems:**
- Simulated exchanges (like ITCH protocol)
- Cryptocurrency exchanges (Binance, Coinbase)
- FIX protocol specification

## Conclusion

This capstone project integrates:
- **Async programming** (tokio, futures)
- **Network protocols** (REST, WebSocket)
- **Data structures** (order book, hash maps)
- **Concurrency** (Arc, Mutex, channels)
- **Event-driven architecture**
- **Performance optimization**
- **Testing and reliability**

Completing this project demonstrates mastery of Rust systems programming!

## Next Module

[Module 11: Rust + Python Interop →](../module-11-python-interop/)
