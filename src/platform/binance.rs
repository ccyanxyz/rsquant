use crate::errors::*;
use crate::models::*;
use crate::traits::*;

use hex::encode as hex_encode;
use serde_json::Value;
use reqwest::StatusCode;
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, USER_AGENT, CONTENT_TYPE};
use ring::hmac;

const HOST: &str = "https://www.binancezh.com";

#[derive(Clone)]
pub struct BinanceRest {
    api_key: String,
    secret_key: String,
    host: String,
}

impl BinanceRest {
    pub fn new(api_key: Option<String>, secret_key: Option<String>, host: String) -> Self {
        BinanceRest {
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

    fn sign(&self, endpoint: &str, request: &str) -> String {
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.secret_key.as_bytes());
        let signature = hex_encode(hmac::sign(&key, request.as_bytes()).as_ref());
        let body: String = format!("{}&signature={}", request, signature);
        let url: String = format!("{}{}?{}", self.host, endpoint, body);
        url
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

impl Spot for BinanceRest {
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

    fn get_balance(&self) -> Result<Balance> {
        unimplemented!()
    }
    fn create_order(&self, amount: f64, price: f64, action: Action, order_type: OrderType) -> Result<String> {
        unimplemented!()
    }
    fn cancel(&self, id: &str) -> Result<bool> {
        unimplemented!()
    }
    fn cancel_all(&self, symbol: &str) -> Result<bool> {
        unimplemented!()
    }
    fn get_order(&self, id: &str) -> Result<Order> {
        unimplemented!()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_orderbook() {
        let api = BinanceRest::new(None, None, "https://www.binancezh.com".to_string());
        let ret = api.get_orderbook("BTCUSDT", 10);
        println!("{:?}", ret);
    }

    #[test]
    fn test_get_ticker() {
        let api = BinanceRest::new(None, None, "https://www.binancezh.com".to_string());
        let ret = api.get_ticker("BTCUSDT");
        println!("{:?}", ret);
    }

    #[test]
    fn test_get_kline() {
        let api = BinanceRest::new(None, None, "https://www.binancezh.com".to_string());
        let ret = api.get_kline("BTCUSDT", "1m", 500);
        println!("{:?}", ret);
        println!("{:?}", ret.unwrap().len());
    }
}
