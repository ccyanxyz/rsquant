use crate::constant::*;
use crate::errors::*;
use crate::models::*;
use crate::traits::*;
use crate::utils::*;

use base64::encode;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, CONTENT_TYPE, USER_AGENT};
use reqwest::StatusCode;
use ring::hmac;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

lazy_static! {
    static ref SPOT_URI: HashMap::<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("get_orderbook", "/market/depth");
        map.insert("get_ticker", "/market/detail/merged");
        map.insert("get_kline", "/market/history/kline");
        map.insert("get_balance", "/v1/account/accounts/account_id/balance");

        map.insert("create_order", "/api/v3/order");
        map.insert("cancel", "/api/v3/order");
        map.insert("cancel_all", "/api/v3/openOrders");
        map.insert("get_order", "/api/v3/order");
        map.insert("get_open_orders", "/api/v3/openOrders");
        map
    };
    static ref MARGIN_URI: HashMap::<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("get_orderbook", "/market/depth");
        map.insert("get_ticker", "/market/detail/merged");
        map.insert("get_kline", "/market/history/kline");
        map.insert("get_balance", "/v1/account/accounts/account_id/balance");

        map.insert("create_order", "/sapi/v1/margin/order");
        map.insert("cancel", "/sapi/v1/margin/order");
        map.insert("cancel_all", "/sapi/v1/margin/openOrders"); // maybe not exist
        map.insert("get_order", "/sapi/v1/margin/order");
        map.insert("get_open_orders", "/sapi/v1/margin/openOrders");
        map
    };
}

#[derive(Clone)]
pub struct Huobi {
    api_key: String,
    secret_key: String,
    host: String,
    is_margin: bool,
    // margin/super-margin
    margin_type: String,
    account_ids: HashMap<String, String>,
}

impl Huobi {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        Huobi {
            api_key: api_key.unwrap_or_else(|| "".into()),
            secret_key: secret_key.unwrap_or_else(|| "".into()),
            host: host,
            is_margin: false,
            margin_type: "margin".into(),
            account_ids: HashMap::new(),
        }
    }

    pub fn set_margin(&mut self, margin_type: &str) {
        self.margin_type = margin_type.into();
    }

    pub fn set_spot(&mut self) {
        self.is_margin = false;
    }

    pub fn set_account_ids(&mut self) -> Result<bool> {
        let uri = "/v1/account/accounts";
        let params: BTreeMap<String, String> = BTreeMap::new();
        let req = self.build_signed_request(params)?;
        let ret = self.get_signed(&uri, &req)?;
        let val: Value = serde_json::from_str(&ret)?;
        if val["data"].is_null() {
            bail!("get_account_ids error: {:?}", val);
        }

        val["data"].as_array().unwrap().iter().for_each(|account| {
            self.account_ids.insert(
                account["type"].as_str().unwrap().to_string(),
                account["id"].as_i64().unwrap().to_string(),
            );
        });
        Ok(true)
    }

    pub fn get(&self, endpoint: &str, request: &str) -> Result<String> {
        let mut url: String = format!("{}{}", self.host, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }
        let response = reqwest::blocking::get(url.as_str())?;
        self.handler(response)
    }

    pub fn post(&self, endpoint: &str) -> Result<String> {
        let url: String = format!("{}{}", self.host, endpoint);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url.as_str())
            .headers(self.build_headers(false)?)
            .send()?;

        self.handler(resp)
    }

    pub fn get_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign("GET", endpoint, request);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .get(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;
        self.handler(resp)
    }

    pub fn post_signed(&self, endpoint: &str, request: &str) -> Result<String> {
        let url = self.sign("POST", endpoint, request);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url.as_str())
            .headers(self.build_headers(true)?)
            .send()?;
        self.handler(resp)
    }

    fn sign(&self, method: &str, endpoint: &str, request: &str) -> String {
        let split = self.host.split("//").collect::<Vec<&str>>();
        let hostname = split[1];
        let request = percent_encode(request).unwrap();
        let payload = method.to_string() + "\n" + hostname + "\n" + endpoint + "\n" + &request;

        let key = hmac::Key::new(hmac::HMAC_SHA256, self.secret_key.as_bytes());
        let signature = encode(hmac::sign(&key, payload.as_bytes()).as_ref());
        println!("payload: {:?}", payload);
        println!("signature: {:?}", signature);
        let body: String = format!("{}&Signature={}", request, signature);
        let url: String = format!("{}{}?{}", self.host, endpoint, body);
        url
    }

    fn build_signed_request(&self, mut params: BTreeMap<String, String>) -> Result<String> {
        if let Ok(ts) = get_utc_ts() {
            params.insert("Timestamp".into(), ts);
            params.insert("AccessKeyId".into(), self.api_key.clone());
            params.insert("SignatureMethod".into(), "HmacSHA256".into());
            params.insert("SignatureVersion".into(), "2".into());
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
            headers.insert(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            );
        }
        /*headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(self.api_key.as_str())?,
        );*/
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
                let err_info = ExErrorInfo {
                    code: err["code"].as_i64().unwrap_or(-1),
                    msg: err["msg"]
                        .as_str()
                        .unwrap_or("unwrap msg failed")
                        .to_string(),
                };
                Err(ErrorKind::ExError(err_info).into())
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}

impl Spot for Huobi {
    fn get_orderbook(&self, symbol: &str, depth: u8) -> Result<Orderbook> {
        let uri = if self.is_margin {
            MARGIN_URI.get("get_orderbook").unwrap()
        } else {
            SPOT_URI.get("get_orderbook").unwrap()
        };
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

    fn get_ticker(&self, symbol: &str) -> Result<Ticker> {
        let uri = if self.is_margin {
            MARGIN_URI.get("get_ticker").unwrap()
        } else {
            SPOT_URI.get("get_ticker").unwrap()
        };
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

    fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> Result<Vec<Kline>> {
        let uri = if self.is_margin {
            MARGIN_URI.get("get_kline").unwrap()
        } else {
            SPOT_URI.get("get_kline").unwrap()
        };
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

    fn get_balance(&self, asset: &str) -> Result<Balance> {
        let uri = if self.is_margin {
            MARGIN_URI.get("get_balance").unwrap()
        } else {
            SPOT_URI.get("get_balance").unwrap()
        };
        let mut account_type = "spot".to_string();
        if self.is_margin {
            account_type = self.margin_type.clone();
        }
        match self.account_ids.get(&account_type) {
            Some(_) => {}
            None => {
                bail!("set_account_ids first");
            }
        }
        let uri = str::replace(uri, "account_id", &self.account_ids[&account_type]);
        let params: BTreeMap<String, String> = BTreeMap::new();
        let req = self.build_signed_request(params)?;
        let ret = self.get_signed(&uri, &req)?;
        let val: Value = serde_json::from_str(&ret)?;
        println!("val: {:?}", val);

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
    ) -> Result<String> {
        let uri = if self.is_margin {
            MARGIN_URI.get("create_order").unwrap()
        } else {
            SPOT_URI.get("create_order").unwrap()
        };
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
        unimplemented!()
    }

    fn cancel_all(&self, symbol: &str) -> Result<bool> {
        unimplemented!()
    }

    fn get_order(&self, id: &str) -> Result<Order> {
        let uri = if self.is_margin {
            MARGIN_URI.get("get_order").unwrap()
        } else {
            SPOT_URI.get("get_order").unwrap()
        };
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
            amount: val["origQty"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap_or(0.0),
            filled: val["executedQty"]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap_or(0.0),
            status: status,
        })
    }

    fn get_open_orders(&self, symbol: &str) -> Result<Vec<Order>> {
        let uri = if self.is_margin {
            MARGIN_URI.get("get_open_orders").unwrap()
        } else {
            SPOT_URI.get("get_open_orders").unwrap()
        };
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
                    price: order["price"]
                        .as_str()
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    amount: order["origQty"]
                        .as_str()
                        .unwrap()
                        .parse::<f64>()
                        .unwrap_or(0.0),
                    filled: order["executedQty"]
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
    fn test_set_account_ids() {
        let mut api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        api.set_account_ids();
    }

    #[test]
    fn test_get_balance() {
        let mut api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        api.set_account_ids();
        let ret = api.get_balance("BTC");
        println!("{:?}", ret);
    }
}
