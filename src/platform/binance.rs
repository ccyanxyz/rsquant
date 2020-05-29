use crate::errors::*;
use crate::models::*;
use crate::traits::*;

use serde_json::Value;
use reqwest::StatusCode;
use reqwest::blocking::Response;

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

    /*
    pub fn post(&self, endpoint: &str) -> Result<String> {
        let url: String = format!("{}{}", self.host, endpoint);
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url.as_str())
            .headers(self.build_headers(false)?)
            .send()?;
        self.handler(resp)
    }*/

    //pub fn get_signed(&self, endpoint: &str, request: &str) -> Result<String>



    fn handler(&self, mut resp: Response) -> Result<String> {
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
        unimplemented!()
    }
    fn get_klines(&self, symbol: &str, period: u16, limit: u16) -> Result<Vec<Kline>> {
        unimplemented!()
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

    //#[test]
    fn test_get() {
        let api = BinanceRest::new(Some("test".to_string()), Some("test".to_string()), "https://www.binancezh.com".to_string());
        let ret = api.get("/api/v3/exchangeInfo", "");
        println!("{:?}", ret);
    }

    #[test]
    fn test_get_orderbook() {
        let api = BinanceRest::new(Some("test".to_string()), Some("test".to_string()), "https://www.binancezh.com".to_string());
        let ret = api.get_orderbook("BTCUSDT", 10);
        println!("{:?}", ret);
    }
}
