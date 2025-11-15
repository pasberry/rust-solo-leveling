use crate::error::{Result, TradingError};
use crate::types::*;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::time::SystemTime;

/// Order book for a single symbol with price-time priority matching
pub struct OrderBook {
    symbol: String,
    /// Buy orders: price (descending) -> queue of orders
    bids: BTreeMap<Decimal, VecDeque<Order>>,
    /// Sell orders: price (ascending) -> queue of orders
    asks: BTreeMap<Decimal, VecDeque<Order>>,
    /// All orders by ID for quick lookup
    orders: HashMap<OrderId, Order>,
    /// Last traded price
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

    /// Add an order and match it against the book
    pub fn add_order(&mut self, mut order: Order) -> Vec<Trade> {
        let mut trades = Vec::new();

        // Match the order
        match order.order_type {
            OrderType::Market => {
                trades.extend(self.match_market_order(&mut order));
            }
            OrderType::Limit => {
                trades.extend(self.match_limit_order(&mut order));
            }
        }

        // Add remaining quantity to book if not fully filled
        if order.remaining_quantity() > 0 && order.order_type == OrderType::Limit {
            self.insert_order(order.clone());
        }

        // Store order
        self.orders.insert(order.id, order);

        trades
    }

    fn match_limit_order(&mut self, order: &mut Order) -> Vec<Trade> {
        let mut trades = Vec::new();
        let price = match order.price {
            Some(p) => p,
            None => return trades,
        };

        let order_side = order.side;

        // Get the opposite side
        let opposite_side = match order_side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let mut prices_to_remove = Vec::new();
        let prices: Vec<Decimal> = opposite_side.keys().copied().collect();

        for book_price in prices {
            // Check if we can match
            let can_match = match order_side {
                Side::Buy => book_price <= price,   // Buy if ask <= our bid
                Side::Sell => book_price >= price,  // Sell if bid >= our ask
            };

            if !can_match {
                break; // No more matches possible (sorted order)
            }

            if let Some(level_orders) = opposite_side.get_mut(&book_price) {
                while let Some(mut passive_order) = level_orders.pop_front() {
                    let quantity = std::cmp::min(order.remaining_quantity(), passive_order.remaining_quantity());

                    order.filled_quantity += quantity;
                    passive_order.filled_quantity += quantity;

                    // Update statuses
                    order.status = if order.remaining_quantity() == 0 {
                        OrderStatus::Filled
                    } else {
                        OrderStatus::PartiallyFilled
                    };

                    passive_order.status = if passive_order.remaining_quantity() == 0 {
                        OrderStatus::Filled
                    } else {
                        OrderStatus::PartiallyFilled
                    };

                    self.last_price = Some(book_price);

                    let trade = Trade {
                        id: TradeId::new(),
                        symbol: self.symbol.clone(),
                        price: book_price,
                        quantity,
                        buyer_order_id: match order_side {
                            Side::Buy => order.id,
                            Side::Sell => passive_order.id,
                        },
                        seller_order_id: match order_side {
                            Side::Sell => order.id,
                            Side::Buy => passive_order.id,
                        },
                        timestamp: SystemTime::now(),
                    };
                    trades.push(trade);

                    // Update passive order in storage
                    self.orders.insert(passive_order.id, passive_order.clone());

                    // If passive order still has quantity, put it back
                    if passive_order.remaining_quantity() > 0 {
                        level_orders.push_front(passive_order);
                        break;
                    }

                    // If incoming order is filled, stop matching
                    if order.remaining_quantity() == 0 {
                        break;
                    }
                }

                if level_orders.is_empty() {
                    prices_to_remove.push(book_price);
                }
            }

            if order.remaining_quantity() == 0 {
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
        let order_side = order.side;

        let opposite_side = match order_side {
            Side::Buy => &mut self.asks,
            Side::Sell => &mut self.bids,
        };

        let mut prices_to_remove = Vec::new();
        let prices: Vec<Decimal> = opposite_side.keys().copied().collect();

        for book_price in prices {
            if let Some(level_orders) = opposite_side.get_mut(&book_price) {
                while let Some(mut passive_order) = level_orders.pop_front() {
                    let quantity = std::cmp::min(order.remaining_quantity(), passive_order.remaining_quantity());

                    order.filled_quantity += quantity;
                    passive_order.filled_quantity += quantity;

                    order.status = if order.remaining_quantity() == 0 {
                        OrderStatus::Filled
                    } else {
                        OrderStatus::PartiallyFilled
                    };

                    passive_order.status = if passive_order.remaining_quantity() == 0 {
                        OrderStatus::Filled
                    } else {
                        OrderStatus::PartiallyFilled
                    };

                    self.last_price = Some(book_price);

                    let trade = Trade {
                        id: TradeId::new(),
                        symbol: self.symbol.clone(),
                        price: book_price,
                        quantity,
                        buyer_order_id: match order_side {
                            Side::Buy => order.id,
                            Side::Sell => passive_order.id,
                        },
                        seller_order_id: match order_side {
                            Side::Sell => order.id,
                            Side::Buy => passive_order.id,
                        },
                        timestamp: SystemTime::now(),
                    };
                    trades.push(trade);

                    self.orders.insert(passive_order.id, passive_order.clone());

                    if passive_order.remaining_quantity() > 0 {
                        level_orders.push_front(passive_order);
                        break;
                    }

                    if order.remaining_quantity() == 0 {
                        break;
                    }
                }

                if level_orders.is_empty() {
                    prices_to_remove.push(book_price);
                }
            }

            if order.remaining_quantity() == 0 {
                break;
            }
        }

        for price in prices_to_remove {
            opposite_side.remove(&price);
        }

        trades
    }

    fn insert_order(&mut self, order: Order) {
        let price = match order.price {
            Some(p) => p,
            None => return,
        };

        let side = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        side.entry(price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> Result<Order> {
        let order = self
            .orders
            .get_mut(&order_id)
            .ok_or(TradingError::OrderNotFound(order_id))?;

        if order.status == OrderStatus::Filled {
            return Err(TradingError::OrderAlreadyFilled);
        }

        order.status = OrderStatus::Canceled;

        let price = order
            .price
            .ok_or_else(|| TradingError::InvalidOrder("No price".to_string()))?;
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

        Ok(order.clone())
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
        let bids: Vec<_> = self
            .bids
            .iter()
            .rev()
            .take(levels)
            .map(|(price, orders)| {
                let quantity: u64 = orders.iter().map(|o| o.remaining_quantity()).sum();
                PriceLevel {
                    price: *price,
                    quantity,
                    order_count: orders.len(),
                }
            })
            .collect();

        let asks: Vec<_> = self
            .asks
            .iter()
            .take(levels)
            .map(|(price, orders)| {
                let quantity: u64 = orders.iter().map(|o| o.remaining_quantity()).sum();
                PriceLevel {
                    price: *price,
                    quantity,
                    order_count: orders.len(),
                }
            })
            .collect();

        MarketDepth {
            symbol: self.symbol.clone(),
            bids,
            asks,
            last_trade_price: self.last_price,
        }
    }

    pub fn get_order(&self, order_id: OrderId) -> Option<&Order> {
        self.orders.get(&order_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_limit_order_matching() {
        let mut book = OrderBook::new("AAPL".to_string());

        // Add sell order at 150.50
        let sell_order = Order::new(
            "AAPL".to_string(),
            Side::Sell,
            OrderType::Limit,
            100,
            Some(dec!(150.50)),
            "sell1".to_string(),
        );
        let trades = book.add_order(sell_order);
        assert_eq!(trades.len(), 0); // No match

        // Add buy order at 150.50 - should match
        let buy_order = Order::new(
            "AAPL".to_string(),
            Side::Buy,
            OrderType::Limit,
            50,
            Some(dec!(150.50)),
            "buy1".to_string(),
        );
        let trades = book.add_order(buy_order);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].quantity, 50);
        assert_eq!(trades[0].price, dec!(150.50));
    }

    #[test]
    fn test_market_order() {
        let mut book = OrderBook::new("AAPL".to_string());

        // Add sell limit orders
        book.add_order(Order::new(
            "AAPL".to_string(),
            Side::Sell,
            OrderType::Limit,
            100,
            Some(dec!(150.00)),
            "sell1".to_string(),
        ));

        // Market buy should match best ask
        let buy_order = Order::new(
            "AAPL".to_string(),
            Side::Buy,
            OrderType::Market,
            50,
            None,
            "buy1".to_string(),
        );
        let trades = book.add_order(buy_order);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].price, dec!(150.00));
    }

    #[test]
    fn test_price_time_priority() {
        let mut book = OrderBook::new("AAPL".to_string());

        // Add two sell orders at same price
        let sell1 = Order::new(
            "AAPL".to_string(),
            Side::Sell,
            OrderType::Limit,
            100,
            Some(dec!(150.00)),
            "sell1".to_string(),
        );
        let sell1_id = sell1.id;
        book.add_order(sell1);

        let sell2 = Order::new(
            "AAPL".to_string(),
            Side::Sell,
            OrderType::Limit,
            100,
            Some(dec!(150.00)),
            "sell2".to_string(),
        );
        book.add_order(sell2);

        // Buy order should match first order (time priority)
        let buy_order = Order::new(
            "AAPL".to_string(),
            Side::Buy,
            OrderType::Limit,
            50,
            Some(dec!(150.00)),
            "buy1".to_string(),
        );
        let trades = book.add_order(buy_order);
        assert_eq!(trades.len(), 1);
        assert_eq!(trades[0].seller_order_id, sell1_id);
    }

    #[test]
    fn test_cancel_order() {
        let mut book = OrderBook::new("AAPL".to_string());

        let order = Order::new(
            "AAPL".to_string(),
            Side::Buy,
            OrderType::Limit,
            100,
            Some(dec!(150.00)),
            "buy1".to_string(),
        );
        let order_id = order.id;
        book.add_order(order);

        let canceled = book.cancel_order(order_id).unwrap();
        assert_eq!(canceled.status, OrderStatus::Canceled);

        // Order should not be in the book
        assert_eq!(book.get_depth(10).bids.len(), 0);
    }

    #[test]
    fn test_market_depth() {
        let mut book = OrderBook::new("AAPL".to_string());

        // Add multiple levels
        for i in 0..5 {
            book.add_order(Order::new(
                "AAPL".to_string(),
                Side::Buy,
                OrderType::Limit,
                100,
                Some(dec!(150) - Decimal::from(i)),
                format!("buy{}", i),
            ));

            book.add_order(Order::new(
                "AAPL".to_string(),
                Side::Sell,
                OrderType::Limit,
                100,
                Some(dec!(151) + Decimal::from(i)),
                format!("sell{}", i),
            ));
        }

        let depth = book.get_depth(3);
        assert_eq!(depth.bids.len(), 3);
        assert_eq!(depth.asks.len(), 3);
        assert_eq!(depth.bids[0].price, dec!(150)); // Best bid
        assert_eq!(depth.asks[0].price, dec!(151)); // Best ask
    }
}
