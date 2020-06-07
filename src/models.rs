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

// binance ws event models

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bids {
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub qty: f64,

    // Never serialized.
    #[serde(skip)]
    ignore: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asks {
    #[serde(with = "string_or_float")]
    pub price: f64,
    #[serde(with = "string_or_float")]
    pub qty: f64,

    // Never serialized.
    #[serde(skip)]
    ignore: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    pub last_update_id: u64,
    pub bids: Vec<Bids>,
    pub asks: Vec<Asks>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdateEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    m: u64,
    t: u64,
    b: u64,
    s: u64,
    #[serde(rename = "T")]
    t_ignore: bool,
    #[serde(rename = "W")]
    w_ignore: bool,
    #[serde(rename = "D")]
    d_ignore: bool,
    #[serde(rename = "B")]
    pub balance: Vec<EventBalance>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "f")]
    pub free: String,
    #[serde(rename = "l")]
    pub locked: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderTradeEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub new_client_order_id: String,
    #[serde(rename = "S")]
    pub side: String,
    #[serde(rename = "o")]
    pub order_type: String,
    #[serde(rename = "f")]
    pub time_in_force: String,
    #[serde(rename = "q")]
    pub qty: String,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(skip, rename = "P")]
    pub p_ignore: String,
    #[serde(skip, rename = "F")]
    pub f_ignore: String,
    #[serde(skip)]
    pub g: i32,
    #[serde(skip, rename = "C")]
    pub c_ignore: Option<String>,
    #[serde(rename = "x")]
    pub execution_type: String,
    #[serde(rename = "X")]
    pub order_status: String,
    #[serde(rename = "r")]
    pub order_reject_reason: String,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "l")]
    pub qty_last_filled_trade: String,
    #[serde(rename = "z")]
    pub accumulated_qty_filled_trades: String,
    #[serde(rename = "L")]
    pub price_last_filled_trade: String,
    #[serde(rename = "n")]
    pub commission: String,
    #[serde(skip, rename = "N")]
    pub asset_commisioned: Option<String>,
    #[serde(rename = "T")]
    pub trade_order_time: u64,
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(skip, rename = "I")]
    pub i_ignore: u64,
    #[serde(skip)]
    pub w: bool,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    #[serde(skip, rename = "M")]
    pub m_ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TradesEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "a")]
    pub aggregated_trade_id: u64,
    #[serde(rename = "p")]
    pub price: String,
    #[serde(rename = "q")]
    pub qty: String,
    #[serde(rename = "f")]
    pub first_break_trade_id: u64,
    #[serde(rename = "l")]
    pub last_break_trade_id: u64,
    #[serde(rename = "T")]
    pub trade_order_time: u64,
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    #[serde(skip, rename = "M")]
    pub m_ignore: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BookTickerEvent {
    #[serde(rename = "u")]
    pub update_id: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "b")]
    pub best_bid: String,
    #[serde(rename = "B")]
    pub best_bid_qty: String,
    #[serde(rename = "a")]
    pub best_ask: String,
    #[serde(rename = "A")]
    pub best_ask_qty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DayTickerEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price_change: String,
    #[serde(rename = "P")]
    pub price_change_percent: String,
    #[serde(rename = "w")]
    pub average_price: String,
    #[serde(rename = "x")]
    pub prev_close: String,
    #[serde(rename = "c")]
    pub current_close: String,
    #[serde(rename = "Q")]
    pub current_close_qty: String,
    #[serde(rename = "b")]
    pub best_bid: String,
    #[serde(rename = "B")]
    pub best_bid_qty: String,
    #[serde(rename = "a")]
    pub best_ask: String,
    #[serde(rename = "A")]
    pub best_ask_qty: String,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "q")]
    pub quote_volume: String,
    #[serde(rename = "O")]
    pub open_time: u64,
    #[serde(rename = "C")]
    pub close_time: u64,
    #[serde(rename = "F")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "n")]
    pub num_trades: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: BKline,
}

#[derive(Debug, Clone)]
pub struct KlineSummary {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: i64,
    pub quote_asset_volume: f64,
    pub number_of_trades: i64,
    pub taker_buy_base_asset_volume: f64,
    pub taker_buy_quote_asset_volume: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BKline {
    #[serde(rename = "t")]
    pub start_time: i64,
    #[serde(rename = "T")]
    pub end_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "f")]
    pub first_trade_id: i32,
    #[serde(rename = "L")]
    pub last_trade_id: i32,
    #[serde(rename = "o")]
    pub open: String,
    #[serde(rename = "c")]
    pub close: String,
    #[serde(rename = "h")]
    pub high: String,
    #[serde(rename = "l")]
    pub low: String,
    #[serde(rename = "v")]
    pub volume: String,
    #[serde(rename = "n")]
    pub number_of_trades: i32,
    #[serde(rename = "x")]
    pub is_final_bar: bool,
    #[serde(rename = "q")]
    pub quote_volume: String,
    #[serde(rename = "V")]
    pub active_buy_volume: String,
    #[serde(rename = "Q")]
    pub active_volume_buy_quote: String,
    #[serde(skip, rename = "B")]
    pub ignore_me: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepthOrderBookEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "U")]
    pub first_update_id: u64,
    #[serde(rename = "u")]
    pub final_update_id: u64,
    #[serde(rename = "b")]
    pub bids: Vec<Bids>,
    #[serde(rename = "a")]
    pub asks: Vec<Asks>,
}

mod string_or_float {
    use std::fmt;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => s.parse().map_err(de::Error::custom),
            StringOrFloat::Float(i) => Ok(i),
        }
    }
}
