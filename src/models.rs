
pub enum OrderStatus {
    Cancelled,
    Filled,
    PartialFilled,
    Submitted,
    Failed,
}

pub enum OrderType {
    Limit,
    Market,
}

pub enum Action {
    Buy,
    Sell,
}

// for futures
pub enum PositionType {
    Long,
    Short,
    All,
}

pub struct Balance {
    pub asset: String,
    pub free: f64, 
    pub locked: f64,
}

pub struct Order {
    pub symbol: String,
    pub order_id: String,
    pub amount: f64,
    pub price: f64,
    pub filled: f64,
    pub status: OrderStatus,
}

pub struct Orderbook {
    pub timestamp: u64,
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
}

pub struct Bid {
    pub price: f64,
    pub amount: f64,
}

pub struct Ask {
    pub price: f64,
    pub amount: f64,
}

pub struct Ticker {
    pub bid: Bid,
    pub ask: Ask,
}

pub struct Kline {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}


// for futures
pub struct Position {
    pub symbol: String,
    pub amount: f64,
    pub price: f64,
    pub pos_type: PositionType,
}
