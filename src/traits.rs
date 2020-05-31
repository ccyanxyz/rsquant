use crate::errors::*;
use crate::models::*;

pub trait Spot {
    fn get_balance(&self, asset: &str) -> Result<Balance>;
    fn create_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        order_type: &str,
    ) -> Result<String>;
    fn cancel(&self, id: &str) -> Result<bool>;
    fn cancel_all(&self, symbol: &str) -> Result<bool>;
    fn get_order(&self, id: &str) -> Result<Order>;
    fn get_open_orders(&self, symbol: &str) -> Result<Vec<Order>>;

    fn get_orderbook(&self, symbol: &str, depth: u8) -> Result<Orderbook>;
    fn get_ticker(&self, symbol: &str) -> Result<Ticker>;
    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> Result<Vec<Kline>>;
}
