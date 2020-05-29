use crate::errors::*;
use crate::models::*;
use crate::traits::*;
use crate::utils::*;
use crate::constant::*;

use hex::encode as hex_encode;
use std::collections::BTreeMap;
use serde_json::Value;
use reqwest::StatusCode;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT, CONTENT_TYPE};
use ring::hmac;

#[derive(Clone)]
pub struct Binance {
    api_key: String,
    secret_key: String,
    host: String,
}

impl Binance {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        Binance {
            api_key: api_key.unwrap_or_else(|| "".into()),
            secret_key: secret_key.unwrap_or_else(|| "".into()),
            host,
        }
    }

    pub fn get(&self, endpoint: &str, request: &str) -> Result<String> {
        let mut url: String = format!("{}{}", self.host, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }
        let response = reqwest::blocking::get(url.as_str())?;
        self.handler(response)
    }

    pub fn get_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign(endpoint, request);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;
        self.handler(resp)
    }

    pub fn post_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign(endpoint, request);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;
        self.handler(resp)
    }

    pub fn delete_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign(endpoint, request);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .delete(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;
        self.handler(resp)
    }

    fn sign(&self, endpoint: &str, request: &str) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.secret_key.as_bytes());
        let signature = hex_encode(hmac::sign(&key, request.as_bytes()).as_ref());
        let body: String = format!("{}&signature={}", request, signature);
        let url: String = format!("{}{}?{}", self.host, endpoint, body);
        url
    }

    fn build_signed_request(&self, mut params: BTreeMap<String, String>) -> Result<String> {
        params.insert("recvWindow".into(), "5000".to_string());

        if let Ok(ts) = get_timestamp() {
            params.insert("timestamp".into(), ts.to_string());
            let mut req = String::new();
            for (k, v) in &params {
                let param = format!("{}={}&", k, v);
                req.push_str(param.as_ref());
            }
            req.pop();
            Ok(req)
        } else {
            bail!("Failed to get timestamp")
        }
    }

    fn build_headers(&self, content_type: bool) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("rsquant"));
        if content_type {
            headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
        }
        headers.insert(HeaderName::from_static("x-mbx-apikey"), HeaderValue::from_str(self.api_key.as_str())?);
        Ok(headers)
    }

    fn handler(&self, resp: Response) -> Result<String> {
        match resp.status() {
            StatusCode::OK => {
                let body = resp.text()?;
                Ok(body)
            }
            StatusCode::INTERNAL_SERVER_ERROR => {
                bail!("Internal Server Error");
            }
            StatusCode::SERVICE_UNAVAILABLE => {
                bail!("Service Unavailable");
            }
            StatusCode::UNAUTHORIZED => {
                bail!("Unauthorized");
            }
            StatusCode::BAD_REQUEST => {
                let err: Value = resp.json()?;
                let err_info = ExErrorInfo{
                    code: err["code"].as_i64().unwrap_or(-1),
                    msg: err["msg"].as_str().unwrap_or("unwrap msg failed").to_string(),
                };
                Err(ErrorKind::ExError(err_info).into())
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}

impl Spot for Binance {
    fn get_orderbook(&self, symbol: &str, depth: u8) -> Result<Orderbook> {
        let uri = "/api/v3/depth";
        let params = format!("symbol={}&limit={}", symbol, depth);
        let ret = self.get(uri, &params)?;
        let val: Value = serde_json::from_str(&ret)?;
        // TODO: faster way to do this?
        let asks = val["asks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|ask| {
                Ask {
                    price: ask[0].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    amount: ask[1].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                }
            })
            .collect::<Vec<Ask>>();
        let bids = val["bids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|bid| {
                Bid {
                    price: bid[0].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    amount: bid[1].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                }
            })
            .collect::<Vec<Bid>>();

        Ok(Orderbook {
            timestamp: val["lastUpdateId"].as_i64().unwrap_or(0) as u64,
            asks: asks,
            bids: bids,
        })
    }
    
    fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        let uri = "/api/v3/ticker/bookTicker";
        let params = format!("symbol={}", symbol);
        let ret = self.get(uri, &params)?;
        let val: Value = serde_json::from_str(&ret)?;

        Ok(Ticker {
            symbol: val["symbol"].as_str().unwrap().into(),
            bid: Bid {
                price: val["bidPrice"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                amount: val["bidQty"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            },
            ask: Ask {
                price: val["askPrice"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                amount: val["askQty"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            }
        })
    }

    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> Result<Vec<Kline>> {
        let uri = "/api/v3/klines";
        let params = format!("symbol={}&interval={}&limit={}", symbol, period, limit);
        let ret = self.get(uri, &params)?;
        let val: Value = serde_json::from_str(&ret)?;
        let klines = val
            .as_array()
            .unwrap()
            .iter()
            .map(|kline| {
                Kline {
                    timestamp: kline[0].as_i64().unwrap_or(0) as u64,
                    open: kline[1].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    high: kline[2].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    low: kline[3].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    close: kline[4].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    volume: kline[5].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                }
            })
            .collect::<Vec<Kline>>();

        Ok(klines)
    }

    fn get_balance(&self, asset: &str) -> Result<Balance> {
        let uri = "/api/v3/account";
        let params: BTreeMap<String, String> = BTreeMap::new();
        let req = self.build_signed_request(params)?;
        let ret = self.get_signed(uri, &req)?;
        let val: Value = serde_json::from_str(&ret)?;
        
        let balance = val["balances"]
            .as_array()
            .unwrap()
            .iter()
            .find(|balance| {
                balance["asset"].as_str().unwrap() == asset.to_string()
            });
        let balance = balance.unwrap();

        Ok(Balance {
            asset: asset.into(),
            free: balance["free"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            locked: balance["locked"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
        })
    }

    fn create_order(&self, symbol: &str, price: f64, amount: f64, action: &str, order_type: &str) -> Result<String> {
        let uri = "/api/v3/order";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("symbol".into(), symbol.into());
        params.insert("side".into(), action.into());
        params.insert("type".into(), order_type.into());
        params.insert("timeInForce".into(), "GTC".into());
        params.insert("quantity".into(), amount.to_string());
        params.insert("price".into(), price.to_string());
        let req = self.build_signed_request(params)?;
        let ret = self.post_signed(uri, &req)?;
        let val: Value = serde_json::from_str(&ret)?;

        Ok(val["orderId"].as_i64().unwrap().to_string())
    }

    fn cancel(&self, id: &str) -> Result<bool> {
        let uri = "/api/v3/order";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("orderId".into(), id.into());
        let req = self.build_signed_request(params)?;
        let _ret = self.delete_signed(uri, &req)?;
        Ok(true)
    }

    fn cancel_all(&self, symbol: &str) -> Result<bool> {
        let uri = "/api/v3/openOrders";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("symbol".into(), symbol.into());
        let req = self.build_signed_request(params)?;
        let _ret = self.delete_signed(uri, &req)?;
        Ok(true)
    }

    fn get_order(&self, id: &str) -> Result<Order> {
        let uri = "/api/v3/order";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("orderId".into(), id.into());
        let req = self.build_signed_request(params)?;
        let ret = self.get_signed(uri, &req)?;
        let val: Value = serde_json::from_str(&ret)?;

        let status: u8 = match val["status"].as_str() {
            Some("NEW") => ORDER_STATUS_SUBMITTED,
            Some("FILLED") => ORDER_STATUS_FILLED,
            Some("PARTIALLY_FILLED") => ORDER_STATUS_PART_FILLED,
            Some("CANCELLED") => ORDER_STATUS_CANCELLED,
            _ => ORDER_STATUS_FAILED,
        };
        Ok(Order {
            symbol: val["symbol"].as_str().unwrap().into(),
            order_id: val["orderId"].as_i64().unwrap().to_string(),
            price: val["price"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            amount: val["origQty"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            filled: val["executedQty"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            status: status,
        })
    }

    fn get_open_orders(&self, symbol: &str) -> Result<Vec<Order>> {
        let uri = "/api/v3/openOrders";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("symbol".into(), symbol.into());
        let req = self.build_signed_request(params)?;
        let ret = self.get_signed(uri, &req)?;
        let val: Value = serde_json::from_str(&ret)?;

        let orders = val
            .as_array()
            .unwrap()
            .iter()
            .map(|order| {
                let status: u8 = match val["status"].as_str() {
                    Some("NEW") => ORDER_STATUS_SUBMITTED,
                    Some("FILLED") => ORDER_STATUS_FILLED,
                    Some("PARTIALLY_FILLED") => ORDER_STATUS_PART_FILLED,
                    Some("CANCELLED") => ORDER_STATUS_CANCELLED,
                    _ => ORDER_STATUS_FAILED,
                };
                Order {
                    symbol: order["symbol"].as_str().unwrap().into(),
                    order_id: order["orderId"].as_i64().unwrap().to_string(),
                    price: order["price"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    amount: order["origQty"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                    filled: order["executedQty"].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
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

    const API_KEY: &'static str = "N9QAtGjFuNXDAnvMlidLzfvGargt54mKQuQbzyafO2hg5Hr8YNHV1e2Jfavi44nK";
    const SECRET_KEY: &'static str = "lCuul7mVApKczbGJBrAgqEIWTWwbQ1BTMBPJyvK19q2BNmlsd5718cAWWByNuY5N";
    const HOST: &'static str = "https://api.binance.com";

    //#[test]
    fn test_get_orderbook() {
        let api = Binance::new(None, None, "https://www.binancezh.com".to_string());
        let ret = api.get_orderbook("BTCUSDT", 10);
        println!("{:?}", ret);
    }

    //#[test]
    fn test_get_ticker() {
        let api = Binance::new(None, None, "https://www.binancezh.com".to_string());
        let ret = api.get_ticker("BTCUSDT");
        println!("{:?}", ret);
    }

    //#[test]
    fn test_get_kline() {
        let api = Binance::new(None, None, "https://www.binancezh.com".to_string());
        let ret = api.get_kline("BTCUSDT", "1m", 500);
        println!("{:?}", ret);
        println!("{:?}", ret.unwrap().len());
    }

    #[test]
    fn test_get_balance() {
        let api = Binance::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let ret = api.get_balance("BTC");
        println!("{:?}", ret);
    }

    #[test]
    fn test_create_order() {
        let api = Binance::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let ret = api.create_order("BTCUSDT".into(), 9300.0, 0.01, "BUY", "LIMIT");
        println!("{:?}", ret);
    }
}
