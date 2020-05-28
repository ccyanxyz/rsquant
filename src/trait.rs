
use crate::models::*;

type Result<T> = Result<T, String>;

pub trait Spot {
    fn get_balance(&self) -> Result<Balance>;
    
    fn create_order(&self, amount: f64, price: f64, action: Action, order_type: OrderType) -> Result<String>;

    fn cancel(&self, id: &str) -> Result<bool>;

    fn cancel_all(&self, symbol: str) -> Result<bool>;

    fn get_order(&self, id: &str) -> Result<Order>;

    fn get_depth(&self, symbol: &str, depth: u8) -> Result<Orderbook>;

    fn get_ticker(&self, symbol: &str) -> Result<Ticker>;

    fn get_klines(&self, symbol: &str, period: u16, limit: u16) -> Result<Vec<Kline>>;
}

pub trait Future {
    fn get_balance(&self) -> Result<Balance>;
    
    fn create_order(&self, amount: f64, price: f64, action: Action, order_type: OrderType) -> Result<String>;

    fn cancel(&self, id: &str) -> Result<bool>;

    fn cancel_all(&self, symbol: str) -> Result<bool>;

    fn get_order(&self, id: &str) -> Result<Order>;
    
    fn get_position(&self, symbol: &str, pos_type: PositionType) -> Result<Position>;

    fn get_depth(&self, symbol: &str, depth: u8) -> Result<Orderbook>;

    fn get_ticker(&self, symbol: &str) -> Result<Ticker>;

    fn get_klines(&self, symbol: &str, period: u16, limit: u16) -> Result<Vec<Kline>>;
}
