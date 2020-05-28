use crate::errors::*;
use reqwest::StatusCode;
use reqwest::blocking::Response;

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

    //pub fn get_signed(&self, endpoint: &str, request: &str) -> Result<String>

    pub fn get(&self, endpoint: &str, request: &str) -> Result<String> {
        let mut url: String = format!("{}{}", self.host, endpoint);
        if !request.is_empty() {
            url.push_str(format!("?{}", request).as_str());
        }

        let response = reqwest::blocking::get(url.as_str())?;

        self.handler(response)
    }

    pub fn get_orderbook(&self, symbol: &str, depth: u8) -> Result<String> {
        let uri = "/api/v3/depth";
        let req_str = format!("symbol={}&limit={}", symbol, depth);
        println!("req_str={:?}", req_str);
        self.get(uri, &req_str)
    }

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
                let err_content: ExErrorInfo{ code: err["code"], msg: err["msg"] };
                Err(ErrorKind::ExError(err).into())
            }
            s => {
                bail!(format!("Received response: {:?}", s));
            }
        }
    }
}

impl Spot for BinanceRest {
    pub fn get_orderbook(&self, symbol: &str, depth: u8) -> Result<String> {
        let uri = "/api/v3/depth";
        let req_str = format!("symbol={}&limit={}", symbol, depth);
        println!("req_str={:?}", req_str);
        self.get(uri, &req_str)
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
