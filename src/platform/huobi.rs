use crate::constant::*;
use crate::errors::*;
use crate::models::*;
use crate::traits::*;
use crate::utils::*;

use base64::encode;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::StatusCode;
use ring::{hmac, digest};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone)]
pub struct Huobi {
    api_key: String,
    secret_key: String,
    host: String,
    account_id: String,
    account_type: String,
}

impl Huobi {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        Huobi {
            api_key: api_key.unwrap_or_else(|| "".into()),
            secret_key: secret_key.unwrap_or_else(|| "".into()),
            host: host,
            account_id: "".into(),
            account_type: "spot".into(),
        }
    }

    pub fn set_account(&mut self, account_type: &str, account_id: &str) {
        self.account_id = account_id.into();
        self.account_type = account_type.into();
    }

    pub fn get_account_id(&self, account_type: &str) -> APIResult<String> {
        let uri = "/v1/account/accounts";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.get_signed(&uri, params)?;
        let val: Value = serde_json::from_str(&ret)?;
        if val["data"].is_null() {
            bail!("get_account_id error: {:?}", val);
        }

        let account_id = val["data"].as_array().unwrap().iter().find(|account| {
                account["type"].as_str().unwrap() == account_type
        });
        Ok(account_id.unwrap()["id"].as_i64().unwrap().to_string())
    }

    pub fn get(&self, endpoint: &str, request: &str) -> APIResult<String> {
        let mut url: String = format!("{}{}", self.host, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }
        let resp = reqwest::blocking::get(url.as_str())?;
        let body = resp.text()?;
        let val: Value = serde_json::from_str(body.as_str())?;
        if val["status"].as_str() == Some("error") {
            if let Some(err_msg) = val["err_msg"].as_str() {
                return Err(Box::new(ExError::ApiError(err_msg.into())));
            } else {
                return Err(Box::new(ExError::ApiError(format!("response: {:?}", val))));
            }
        }
        Ok(body)
    }

    pub fn post(&self, endpoint: &str) -> APIResult<String> {
        let url: String = format!("{}{}", self.host, endpoint);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url.as_str())
            .send()?;

        let body = resp.text()?;
        let val: Value = serde_json::from_str(body.as_str())?;
        if val["status"].as_str() == Some("error") {
            if let Some(err_msg) = val["err_msg"].as_str() {
                return Err(Box::new(ExError::ApiError(err_msg.into())));
            } else {
                return Err(Box::new(ExError::ApiError(format!("response: {:?}", val))));
            }
        }
        Ok(body)
    }

    pub fn get_signed(&self, endpoint: &str, mut params: BTreeMap<String, String>) -> APIResult<String> {
        let ts = get_utc_ts();
        params.insert("Timestamp".into(), ts);
        params.insert("AccessKeyId".into(), self.api_key.clone());
        params.insert("SignatureMethod".into(), "HmacSHA256".into());
        params.insert("SignatureVersion".into(), "2".into());

        let params_str = self.build_query_string(params);
        let split = self.host.split("//").collect::<Vec<&str>>();
        let hostname = split[1];
        let signature = self.sign(&format!("{}\n{}\n{}\n{}", "GET", hostname, endpoint, params_str));

        let req = format!("{}{}?{}&Signature={}", self.host, endpoint, params_str, percent_encode(&signature.clone()));

        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(req.as_str())
            .send()?;
        let body = resp.text()?;
        let val: Value = serde_json::from_str(body.as_str())?;
        if val["status"].as_str() == Some("error") {
            if let Some(err_msg) = val["err_msg"].as_str() {
                return Err(Box::new(ExError::ApiError(err_msg.into())));
            } else {
                return Err(Box::new(ExError::ApiError(format!("response: {:?}", val))));
            }
        }
        Ok(body)
    }

    pub fn post_signed(&self, endpoint: &str, mut params: BTreeMap<String, String>, body: &BTreeMap<String, String>) -> APIResult<String> {
        let ts = get_utc_ts();
        params.insert("Timestamp".into(), ts);
        params.insert("AccessKeyId".into(), self.api_key.clone());
        params.insert("SignatureMethod".into(), "HmacSHA256".into());
        params.insert("SignatureVersion".into(), "2".into());

        let params_str = self.build_query_string(params);
        let split = self.host.split("//").collect::<Vec<&str>>();
        let hostname = split[1];
        let signature = self.sign(&format!("{}\n{}\n{}\n{}", "POST", hostname, endpoint, params_str));

        let req = format!("{}{}?{}&Signature={}", self.host, endpoint, params_str, percent_encode(&signature.clone()));

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(req.as_str())
            .json(body)
            .send()?;
        let body = resp.text()?;
        let val: Value = serde_json::from_str(body.as_str())?;
        if val["status"].as_str() == Some("error") {
            if let Some(err_msg) = val["err_msg"].as_str() {
                return Err(Box::new(ExError::ApiError(err_msg.into())));
            } else {
                return Err(Box::new(ExError::ApiError(format!("response: {:?}", val))));
            }
        }
        Ok(body)
    }

    fn sign(&self, digest: &str) -> String {
        use data_encoding::BASE64;
        let key = hmac::SigningKey::new(&digest::SHA256, self.secret_key.as_bytes());
        let sig = hmac::sign(&key, digest.as_bytes());
        BASE64.encode(sig.as_ref())
    }

    fn build_query_string(&self, mut params: BTreeMap<String, String>) -> String {
        params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, percent_encode(&v.clone())))
            .collect::<Vec<String>>()
            .join("&")
    }
}

impl Spot for Huobi {
    fn get_orderbook(&self, symbol: &str, depth: u8) -> APIResult<Orderbook> {
        let uri = "/market/depth";
        let symbol = symbol.to_lowercase();
        let params = format!("symbol={}&depth={}&type=step0", symbol, depth);
        let ret = self.get(uri, &params)?;
        let val: Value = serde_json::from_str(&ret)?;
        // TODO: faster way to do this?
        let asks = val["tick"]["asks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|ask| Ask {
                price: ask[0].as_f64().unwrap_or(0.0),
                amount: ask[1].as_f64().unwrap_or(0.0),
            })
            .collect::<Vec<Ask>>();
        let bids = val["tick"]["bids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|bid| Bid {
                price: bid[0].as_f64().unwrap_or(0.0),
                amount: bid[1].as_f64().unwrap_or(0.0),
            })
            .collect::<Vec<Bid>>();

        Ok(Orderbook {
            timestamp: val["ts"].as_i64().unwrap_or(0) as u64,
            asks: asks,
            bids: bids,
        })
    }

    fn get_ticker(&self, symbol: &str) -> APIResult<Ticker> {
        let uri = "/market/detail/merged";
        let params = format!("symbol={}", symbol.to_lowercase());
        let ret = self.get(uri, &params)?;
        let val: Value = serde_json::from_str(&ret)?;

        Ok(Ticker {
            symbol: symbol.into(),
            bid: Bid {
                price: val["tick"]["bid"][0].as_f64().unwrap_or(0.0),
                amount: val["tick"]["bid"][1].as_f64().unwrap_or(0.0),
            },
            ask: Ask {
                price: val["tick"]["ask"][0].as_f64().unwrap_or(0.0),
                amount: val["tick"]["ask"][1].as_f64().unwrap_or(0.0),
            },
        })
    }

    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> APIResult<Vec<Kline>> {
        let uri = "/market/history/kline";
        let params = format!(
            "symbol={}&period={}&size={}",
            symbol.to_lowercase(),
            period,
            limit
        );
        let ret = self.get(uri, &params)?;
        let val: Value = serde_json::from_str(&ret)?;
        let klines = val["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|kline| Kline {
                timestamp: kline["id"].as_i64().unwrap_or(0) as u64,
                open: kline["open"].as_f64().unwrap_or(0.0),
                high: kline["high"].as_f64().unwrap_or(0.0),
                low: kline["low"].as_f64().unwrap_or(0.0),
                close: kline["close"].as_f64().unwrap_or(0.0),
                volume: kline["vol"].as_f64().unwrap_or(0.0),
            })
            .collect::<Vec<Kline>>();

        Ok(klines)
    }

    fn get_balance(&self, asset: &str) -> APIResult<Balance> {
        let uri = format!("/v1/account/accounts/{}/balance", self.account_id);
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.get_signed(&uri, params)?;
        let val: Value = serde_json::from_str(&ret)?;

        let mut balance = Balance {
            asset: asset.into(),
            free: 0.0,
            locked: 0.0,
        };
        val["data"]["list"]
            .as_array()
            .unwrap()
            .iter()
            .for_each(|item| {
                if item["currency"].as_str().unwrap() == asset.to_lowercase() {
                    if item["type"].as_str().unwrap() == "trade".to_string() {
                        balance.free = item["balance"]
                            .as_str()
                            .unwrap()
                            .parse::<f64>()
                            .unwrap_or(0.0);
                    }
                    if item["type"].as_str().unwrap() == "frozen".to_string() {
                        balance.locked = item["balance"]
                            .as_str()
                            .unwrap()
                            .parse::<f64>()
                            .unwrap_or(0.0);
                    }
                }
            });
        Ok(balance)
    }

    fn create_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        order_type: &str,
    ) -> APIResult<String> {
        let uri = "/v1/order/orders/place";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let mut body: BTreeMap<String, String> = BTreeMap::new();
        body.insert("account-id".into(), self.account_id.clone());
        body.insert("symbol".into(), symbol.to_string().to_lowercase());
        let body_type = action.to_string() + "-" + order_type;
        let body_type = body_type.to_lowercase();
        body.insert("type".into(), body_type);
        body.insert("amount".into(), amount.to_string());
        body.insert("price".into(), price.to_string());
        body.insert("source".into(), self.account_type.clone() + "-api");
        let ret = self.post_signed(uri, params, &body)?;
        let val: Value = serde_json::from_str(&ret)?;

        Ok(val["data"].as_str().unwrap().to_string())
    }

    fn cancel(&self, id: &str) -> APIResult<bool> {
        let uri = format!("/v1/order/orders/{}/submitcancel", id);
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let mut body: BTreeMap<String, String> = BTreeMap::new();
        let _ret = self.post_signed(&uri, params, &body)?;
        Ok(true)
    }

    fn cancel_all(&self, symbol: &str) -> APIResult<bool> {
        let uri = "/v1/order/orders/batchCancelOpenOrders";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let mut body: BTreeMap<String, String> = BTreeMap::new();
        body.insert("account-id".into(), self.account_id.clone());
        body.insert("symbol".into(), symbol.to_string().to_lowercase());
        let _ret = self.post_signed(uri, params, &body)?;
        Ok(true)
    }

    fn get_order(&self, id: &str) -> APIResult<Order> {
        let uri = format!("/v1/order/orders/{}", id);
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.get_signed(&uri, params)?;
        let val: Value = serde_json::from_str(&ret)?;

        let status: u8 = match val["data"]["state"].as_str() {
            Some("submitted") => ORDER_STATUS_SUBMITTED,
            Some("filled") => ORDER_STATUS_FILLED,
            Some("partial-filled") => ORDER_STATUS_PART_FILLED,
            Some("canceled") | Some("partial-canceled") => ORDER_STATUS_CANCELLED,
            _ => ORDER_STATUS_FAILED,
        };
        Ok(Order {
            symbol: val["data"]["symbol"].as_str().unwrap().into(),
            order_id: val["data"]["id"].as_i64().unwrap().to_string(),
            price: val["data"]["price"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            amount: val["data"]["amount"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap_or(0.0),
            filled: val["data"]["field-amount"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap_or(0.0),
            status: status,
        })
    }

    fn get_open_orders(&self, symbol: &str) -> APIResult<Vec<Order>> {
        let uri = "/v1/order/openOrders";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("account-id".into(), self.account_id.clone());
        params.insert("symbol".into(), symbol.to_string().to_lowercase());
        let ret = self.get_signed(uri, params)?;
        let val: Value = serde_json::from_str(&ret)?;

        let orders = val["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|order| {
                let status: u8 = match order["state"].as_str() {
                    Some("submitted") => ORDER_STATUS_SUBMITTED,
                    Some("filled") => ORDER_STATUS_FILLED,
                    Some("partial-filled") => ORDER_STATUS_PART_FILLED,
                    Some("canceled") | Some("partial-canceled") => ORDER_STATUS_CANCELLED,
                    _ => ORDER_STATUS_FAILED,
                };
                Order {
                    symbol: order["symbol"].as_str().unwrap().into(),
                    order_id: order["id"].as_i64().unwrap().to_string(),
                    price: order["price"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    amount: order["amount"]
                        .as_str()
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    filled: order["filled-amount"]
                        .as_str()
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    status: status,
                }
            })
            .collect::<Vec<Order>>();
        Ok(orders)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const HOST: &'static str = "https://api.huobi.pro";
    const API_KEY: &'static str = "e1a90fa3-ht4tgq1e4t-e02cdf4d-f7b94";
    const SECRET_KEY: &'static str = "5d80336b-c8263ba7-c58bdc4e-ff5f0";

    //#[test]
    fn test_get_orderbook() {
        let api = Huobi::new(None, None, HOST.into());
        let ret = api.get_orderbook("BTCUSDT", 10);
        println!("{:?}", ret);
    }

    //#[test]
    fn test_get_ticker() {
        let api = Huobi::new(None, None, HOST.into());
        let ret = api.get_ticker("BTCUSDT");
        println!("{:?}", ret);
    }

    //#[test]
    fn test_get_kline() {
        let api = Huobi::new(None, None, HOST.into());
        let ret = api.get_kline("BTCUSDT", "15min", 10);
        println!("{:?}", ret);
    }

    //#[test]
    fn test_get_account_id() {
        let api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let acc_id = api.get_account_id("margin");
        println!("margin_id: {:?}", acc_id);
    }

    //#[test]
    fn test_get_balance() {
        let mut api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let acc_id = api.get_account_id("super-margin").unwrap();
        api.set_account("super-margin", &acc_id);
        let ret = api.get_balance("BTC");
        println!("{:?}", ret);
    }

    #[test]
    fn test_orders() {
        let mut api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        // set account_id
        let acc_id = api.get_account_id("super-margin").unwrap();
        api.set_account("super-margin", &acc_id);

        // create_order
        let order_id = api.create_order("BTCUSDT", 10000.0, 0.01, "SELL", "LIMIT");
        println!("order_id: {:?}", order_id);
        
        // get_open_orders
        let open_orders = api.get_open_orders("BTCUSDT");
        println!("open_orders: {:?}", open_orders);

        // cancel_all
        api.cancel_all("BTCUSDT");
        
        // get_order
        let order = api.get_order(&order_id.unwrap());
        println!("order: {:?}", order);
    }
}
