use crate::errors::*;
use crate::models::*;

pub trait Spot {
    fn get_balance(&self, asset: &str) -> APIResult<Balance>;
    fn create_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        order_type: &str,
    ) -> APIResult<String>;
    fn cancel(&self, id: &str) -> APIResult<bool>;
    fn cancel_all(&self, symbol: &str) -> APIResult<bool>;
    fn get_order(&self, id: &str) -> APIResult<Order>;
    fn get_open_orders(&self, symbol: &str) -> APIResult<Vec<Order>>;

    fn get_orderbook(&self, symbol: &str, depth: u8) -> APIResult<Orderbook>;
    fn get_ticker(&self, symbol: &str) -> APIResult<Ticker>;
    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> APIResult<Vec<Kline>>;
}
