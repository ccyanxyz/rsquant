use crate::errors::*;
use crate::models::*;

pub trait SpotRest {
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

pub trait FutureRest {
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

pub trait SpotWs {
    fn sub_orderbook(&mut self, symbol: &str);
    fn sub_kline(&mut self, symbol: &str, period: &str);
    fn sub_ticker(&mut self, symbol: &str);
    fn sub_trade(&mut self, symbol: &str);
    
    fn sub_order_update(&mut self, symbol: &str);
}

pub trait FutureWs {
    fn sub_orderbook(&self, symbol: &str);
    fn sub_kline(&self, symbol: &str, period: &str);
    fn sub_ticker(&self, symbol: &str);
    fn sub_trade(&self, symbol: &str);
    
    fn sub_order_update(&self, symbol: &str);
}
