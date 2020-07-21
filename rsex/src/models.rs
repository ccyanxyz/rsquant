// for futures
#[derive(Debug)]
pub enum PositionType {
    Long,
    Short,
    All,
}

#[derive(Debug)]
pub struct SymbolInfo {
    pub base: String,
    pub quote: String,
    pub symbol: String,
    pub price_precision: u8,
    pub amount_precision: u8,
    pub min_amount: f64,
    pub min_value: f64,
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
    pub side: String,
    pub filled: f64,
    pub status: u8,
}

#[derive(Debug)]
pub struct Orderbook {
    pub timestamp: u64,
    pub bids: Vec<Bid>,
    pub asks: Vec<Ask>,
}

#[derive(Debug)]
pub struct Trade {
    pub timestamp: u64,
    pub amount: f64,
    pub price: f64,
    pub side: String,
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
    pub timestamp: u64,
    pub bid: Bid,
    pub ask: Ask,
}

impl Ticker {
    pub fn new() -> Self {
        Ticker {
            timestamp: 0u64,
            bid: Bid {
                price: 0f64,
                amount: 0f64,
            },
            ask: Ask {
                price: 0f64,
                amount: 0f64,
            }
        }
    }
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
