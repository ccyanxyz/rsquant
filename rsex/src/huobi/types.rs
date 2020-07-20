use crate::constant::*;
use crate::models::*;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
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
    #[serde(default, rename = "err-code")]
    pub err_code: String,
    #[serde(default, rename = "err-msg")]
    pub err_msg: String,
    #[serde(default, rename = "order-state")]
    pub order_state: i8,
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
            bids,
            asks,
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BalanceInfoItem {
    pub currency: String,
    #[serde(rename = "type")]
    pub ty: String,
    pub balance: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct BalanceInfo {
    pub id: u32,
    #[serde(rename = "type")]
    pub ty: String,
    pub state: String,
    pub list: Vec<BalanceInfoItem>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawOrderInfo {
    pub id: u64,
    pub symbol: String,
    #[serde(rename = "account-id")]
    pub account_id: u32,
    pub price: String,
    pub amount: String,
    #[serde(rename = "created-at")]
    pub create_at: u64,
    #[serde(rename = "type")]
    pub ty: String,
    #[serde(alias = "filled-amount", alias = "field-amount")]
    pub filled_amount: String,
    #[serde(rename = "filled-cash-amount", alias = "field-cash-amount")]
    pub filled_cash_amount: String,
    #[serde(rename = "filled-fees", alias = "field-fees")]
    pub filled_fees: String,
    #[serde(default, rename = "finished-at")]
    pub finished_at: u64,
    #[serde(default, rename = "user-id")]
    pub user_id: u32,
    pub source: String,
    pub state: String,
    #[serde(default, rename = "canceled-at")]
    pub canceled_at: u64,
}

impl From<RawOrderInfo> for Order {
    fn from(item: RawOrderInfo) -> Order {
        let status = match item.state.as_str() {
            "partial-filled" => ORDER_STATUS_PART_FILLED,
            "partial-canceled" => ORDER_STATUS_CANCELLED,
            "filled" => ORDER_STATUS_FILLED,
            "canceled" => ORDER_STATUS_CANCELLED,
            "created" => ORDER_STATUS_SUBMITTED,
            "submitted" => ORDER_STATUS_SUBMITTED,
            _ => ORDER_STATUS_FAILED,
        };
        Order {
            symbol: item.symbol,
            order_id: item.id.to_string(),
            amount: item.amount.parse::<f64>().unwrap_or(0.0),
            price: item.price.parse::<f64>().unwrap_or(0.0),
            //FIXME: check order.ty in {"SELL", "BUY"}
            side: item.ty.to_uppercase(),
            filled: item.price.parse::<f64>().unwrap_or(0.0),
            status,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct RawTrade {
    pub amount: f64,
    pub ts: u64,
    pub id: u128,
    #[serde(rename = "tradeId")]
    pub trade_id: u64,
    pub price: f64,
    pub direction: String,
}

impl From<RawTrade> for Trade {
    fn from(item: RawTrade) -> Trade {
        Trade {
            timestamp: item.ts,
            amount: item.amount,
            price: item.price,
            side: item.direction,
        }
    }
}
