use crate::error::{Result, TradingError};
use crate::orderbook::OrderBook;
use crate::types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Matching engine that manages multiple order books
pub struct MatchingEngine {
    books: HashMap<String, OrderBook>,
}

impl MatchingEngine {
    pub fn new() -> Self {
        MatchingEngine {
            books: HashMap::new(),
        }
    }

    pub fn add_symbol(&mut self, symbol: String) {
        self.books.insert(symbol.clone(), OrderBook::new(symbol));
    }

    pub fn add_order(&mut self, order: Order) -> Result<Vec<Trade>> {
        let book = self
            .books
            .get_mut(&order.symbol)
            .ok_or_else(|| TradingError::SymbolNotFound(order.symbol.clone()))?;

        Ok(book.add_order(order))
    }

    pub fn cancel_order(&mut self, symbol: &str, order_id: OrderId) -> Result<Order> {
        let book = self
            .books
            .get_mut(symbol)
            .ok_or_else(|| TradingError::SymbolNotFound(symbol.to_string()))?;

        book.cancel_order(order_id)
    }

    pub fn get_market_depth(&self, symbol: &str, levels: usize) -> Result<MarketDepth> {
        let book = self
            .books
            .get(symbol)
            .ok_or_else(|| TradingError::SymbolNotFound(symbol.to_string()))?;

        Ok(book.get_depth(levels))
    }

    pub fn get_order(&self, symbol: &str, order_id: OrderId) -> Option<&Order> {
        self.books.get(symbol).and_then(|book| book.get_order(order_id))
    }
}

pub type SharedEngine = Arc<RwLock<MatchingEngine>>;

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}
