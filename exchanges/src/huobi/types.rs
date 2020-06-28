use crate::models;
use std::convert::Into;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response<T> {
    pub status: String,
    pub data: T,
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
    #[serde(skip, rename = "leverage-ratio")]
    pub max_leverage: u8,
}

impl From<RawSymbolInfo> for models::SymbolInfo {
    fn from(item: RawSymbolInfo) -> models::SymbolInfo {
        models::SymbolInfo {
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
/*
impl Into<models::SymbolInfo> for RawSymbolInfo {
    fn into(&self) -> models::SymbolInfo {
        models::SymbolInfo {
            base: self.base,
            quote: self.quote,
            symbol: self.symbol,
            price_precision: self.price_precision,
            amount_precision: self.amount_precision,
            min_amount: self.min_amount,
            min_value: self.min_value,
        }
    }
}
*/
