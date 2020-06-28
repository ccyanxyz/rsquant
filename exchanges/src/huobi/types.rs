use crate::models::*;
use std::convert::Into;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response<T> {
    pub status: String,
    #[serde(default)]
    pub ch: String,
    #[serde(default)]
    pub ts: u64,
    #[serde(default)]
    pub data: T,
    #[serde(default)]
    pub tick: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountInfo {
    pub id: u32,
    #[serde(rename = "type")]
    pub ty: String,
    pub subtype: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawSymbolInfo {
    #[serde(rename = "base-currency")]
    pub base: String,
    #[serde(rename = "quote-currency")]
    pub quote: String,
    #[serde(rename = "price-precision")]
    pub price_precision: u8,
    #[serde(rename = "amount-precision")]
    pub amount_precision: u8,
    #[serde(rename = "symbol-partition")]
    pub partition: String,
    pub symbol: String,
    pub state: String,
    #[serde(rename = "value-precision")]
    pub value_precision: u8,
    #[serde(rename = "min-order-amt")]
    pub min_amount: f64,
    #[serde(rename = "max-order-amt")]
    pub max_amount: f64,
    #[serde(rename = "min-order-value")]
    pub min_value: f64,
    #[serde(default, rename = "leverage-ratio")]
    pub max_leverage: f32,
}

impl From<RawSymbolInfo> for SymbolInfo {
    fn from(item: RawSymbolInfo) -> SymbolInfo {
        SymbolInfo {
            base: item.base,
            quote: item.quote,
            symbol: item.symbol,
            price_precision: item.price_precision,
            amount_precision: item.amount_precision,
            min_amount: item.min_amount,
            min_value: item.min_value,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawOrderbook {
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub ts: u64,
    pub bids: Vec<[f64; 2]>,
    pub asks: Vec<[f64; 2]>,
}

impl From<RawOrderbook> for Orderbook {
    fn from(item: RawOrderbook) -> Orderbook {
        let bids = item
            .bids
            .iter()
            .map(|bid| Bid {
                price: bid[0],
                amount: bid[1],
            })
            .collect::<Vec<Bid>>();
        let asks = item
            .asks
            .iter()
            .map(|ask| Ask {
                price: ask[0],
                amount: ask[1],
            })
            .collect::<Vec<Ask>>();
        Orderbook {
            timestamp: item.ts,
            bids: bids,
            asks: asks,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawTicker {
    pub id: u64,
    #[serde(default)]
    pub ts: u64,
    pub close: f64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub amount: f64,
    pub count: f64,
    pub vol: f64,
    pub ask: [f64; 2],
    pub bid: [f64; 2],
}

impl From<RawTicker> for Ticker {
    fn from(item: RawTicker) -> Ticker {
        Ticker {
            timestamp: item.ts,
            ask: Ask {
                price: item.ask[0],
                amount: item.ask[1],
            },
            bid: Bid {
                price: item.bid[0],
                amount: item.bid[1],
            },
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawKline {
    pub id: u64,
    pub amount: f64,
    pub count: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    #[serde(rename = "vol")]
    pub volume: f64,
}

impl From<RawKline> for Kline {
    fn from(item: RawKline) -> Kline {
        Kline {
            timestamp: item.id,
            open: item.open,
            high: item.high,
            low: item.low,
            close: item.close,
            volume: item.volume,
        }
    }
}
