
#[derive(Debug)]
pub enum OrderStatus {
    Cancelled,
    Filled,
    PartialFilled,
    Submitted,
    Failed,
}

#[derive(Debug)]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(Debug)]
pub enum Action {
    Buy,
    Sell,
}

// for futures
#[derive(Debug)]
pub enum PositionType {
    Long,
    Short,
    All,
}

#[derive(Debug)]
pub struct Balance {
    pub asset: String,
    pub free: f64, 
    pub locked: f64,
}

#[derive(Debug)]
pub struct Order {
    pub symbol: String,
    pub order_id: String,
    pub amount: f64,
    pub price: f64,
    pub filled: f64,
    pub status: OrderStatus,
}

#[derive(Debug)]
pub struct Orderbook {
    pub timestamp: u64,
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
}

#[derive(Debug)]
pub struct Bid {
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug)]
pub struct Ask {
    pub price: f64,
    pub amount: f64,
}

#[derive(Debug)]
pub struct Ticker {
    pub symbol: String,
    pub bid: Bid,
    pub ask: Ask,
}

#[derive(Debug)]
pub struct Kline {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

// for futures
#[derive(Debug)]
pub struct Position {
    pub symbol: String,
    pub amount: f64,
    pub price: f64,
    pub pos_type: PositionType,
}
