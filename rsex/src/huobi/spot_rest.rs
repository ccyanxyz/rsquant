use crate::errors::*;
use crate::huobi::types::*;
use crate::models::*;
use crate::utils::*;

use ring::{digest, hmac};
use serde_json::Value;
use std::collections::BTreeMap;

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
            host,
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
        let params: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.get_signed(&uri, params)?;
        let resp: Response<Vec<AccountInfo>> = serde_json::from_str(&ret)?;
        let account_id = resp.data.iter().find(|account| account.ty == account_type);
        match account_id {
            Some(acc_id) => Ok(acc_id.id.to_string()),
            None => Err(Box::new(ExError::ApiError("account_id None".into()))),
        }
    }

    pub fn get_symbols(&self) -> APIResult<Vec<SymbolInfo>> {
        let uri = "/v1/common/symbols";
        let ret = self.get(&uri, "")?;
        let resp: Response<Vec<RawSymbolInfo>> = serde_json::from_str(&ret)?;
        let symbols: Vec<SymbolInfo> = resp
            .data
            .into_iter()
            .map(|symbol| symbol.into())
            .collect::<Vec<SymbolInfo>>();
        Ok(symbols)
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
        let resp = client.post(url.as_str()).send()?;

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

    pub fn get_signed(
        &self,
        endpoint: &str,
        mut params: BTreeMap<String, String>,
    ) -> APIResult<String> {
        let ts = get_utc_ts();
        params.insert("Timestamp".into(), ts);
        params.insert("AccessKeyId".into(), self.api_key.clone());
        params.insert("SignatureMethod".into(), "HmacSHA256".into());
        params.insert("SignatureVersion".into(), "2".into());

        let params_str = self.build_query_string(params);
        let split = self.host.split("//").collect::<Vec<&str>>();
        let hostname = split[1];
        let signature = self.sign(&format!(
            "{}\n{}\n{}\n{}",
            "GET", hostname, endpoint, params_str
        ));

        let req = format!(
            "{}{}?{}&Signature={}",
            self.host,
            endpoint,
            params_str,
            percent_encode(&signature)
        );

        let client = reqwest::blocking::Client::new();
        let resp = client.get(req.as_str()).send()?;
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

    pub fn post_signed(
        &self,
        endpoint: &str,
        mut params: BTreeMap<String, String>,
        body: &BTreeMap<String, String>,
    ) -> APIResult<String> {
        let ts = get_utc_ts();
        params.insert("Timestamp".into(), ts);
        params.insert("AccessKeyId".into(), self.api_key.clone());
        params.insert("SignatureMethod".into(), "HmacSHA256".into());
        params.insert("SignatureVersion".into(), "2".into());

        let params_str = self.build_query_string(params);
        let split = self.host.split("//").collect::<Vec<&str>>();
        let hostname = split[1];
        let signature = self.sign(&format!(
            "{}\n{}\n{}\n{}",
            "POST", hostname, endpoint, params_str
        ));

        let req = format!(
            "{}{}?{}&Signature={}",
            self.host,
            endpoint,
            params_str,
            percent_encode(&signature)
        );

        let client = reqwest::blocking::Client::new();
        let resp = client.post(req.as_str()).json(body).send()?;
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

    fn build_query_string(&self, params: BTreeMap<String, String>) -> String {
        params
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, percent_encode(&v)))
            .collect::<Vec<String>>()
            .join("&")
    }

    pub fn get_orderbook(&self, symbol: &str, depth: u8) -> APIResult<Orderbook> {
        let uri = "/market/depth";
        let symbol = symbol.to_lowercase();
        let params = format!("symbol={}&depth={}&type=step0", symbol, depth);
        let ret = self.get(uri, &params)?;
        let resp: Response<RawOrderbook> = serde_json::from_str(&ret)?;
        let mut orderbook: Orderbook = resp.tick.into();
        if orderbook.timestamp == 0 {
            orderbook.timestamp = resp.ts;
        }
        Ok(orderbook)
    }

    pub fn get_ticker(&self, symbol: &str) -> APIResult<Ticker> {
        let uri = "/market/detail/merged";
        let params = format!("symbol={}", symbol.to_lowercase());
        let ret = self.get(uri, &params)?;
        let resp: Response<RawTicker> = serde_json::from_str(&ret)?;
        let mut ticker: Ticker = resp.tick.into();
        if ticker.timestamp == 0 {
            ticker.timestamp = resp.ts;
        }
        Ok(ticker)
    }

    pub fn get_kline(&self, symbol: &str, period: &str, limit: u16) -> APIResult<Vec<Kline>> {
        let uri = "/market/history/kline";
        let params = format!(
            "symbol={}&period={}&size={}",
            symbol.to_lowercase(),
            period,
            limit
        );
        let ret = self.get(uri, &params)?;
        let resp: Response<Vec<RawKline>> = serde_json::from_str(&ret)?;
        let klines = resp
            .data
            .into_iter()
            .map(|kline| kline.into())
            .collect::<Vec<Kline>>();
        Ok(klines)
    }

    pub fn get_balance(&self, asset: &str) -> APIResult<Balance> {
        let uri = format!("/v1/account/accounts/{}/balance", self.account_id);
        let params: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.get_signed(&uri, params)?;
        let resp: Response<BalanceInfo> = serde_json::from_str(&ret)?;

        let mut balance = Balance {
            asset: asset.into(),
            free: 0.0,
            locked: 0.0,
        };
        resp.data.list.iter().for_each(|item| {
            if item.currency == asset.to_lowercase() {
                if item.ty == "trade" {
                    balance.free = item.balance.parse::<f64>().expect("parse float error");
                    //.unwrap_or(0.0);
                }
                if item.ty == "frozen" {
                    balance.locked = item.balance.parse::<f64>().expect("parse float error");
                    //.unwrap_or(0.0);
                }
            }
        });
        Ok(balance)
    }

    pub fn create_order(
        &self,
        symbol: &str,
        price: f64,
        amount: f64,
        action: &str,
        order_type: &str,
    ) -> APIResult<String> {
        let uri = "/v1/order/orders/place";
        let params: BTreeMap<String, String> = BTreeMap::new();
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
        let resp: Response<String> = serde_json::from_str(&ret)?;

        Ok(resp.data)
    }

    pub fn cancel(&self, id: &str) -> APIResult<bool> {
        let uri = format!("/v1/order/orders/{}/submitcancel", id);
        let params: BTreeMap<String, String> = BTreeMap::new();
        let body: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.post_signed(&uri, params, &body)?;
        let resp: Response<String> = serde_json::from_str(&ret)?;
        if resp.status == "ok" {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn cancel_all(&self, symbol: &str) -> APIResult<bool> {
        let uri = "/v1/order/orders/batchCancelOpenOrders";
        let params: BTreeMap<String, String> = BTreeMap::new();
        let mut body: BTreeMap<String, String> = BTreeMap::new();
        body.insert("account-id".into(), self.account_id.clone());
        body.insert("symbol".into(), symbol.to_string().to_lowercase());
        let _ret = self.post_signed(uri, params, &body)?;
        Ok(true)
    }

    pub fn get_order(&self, id: &str) -> APIResult<Order> {
        let uri = format!("/v1/order/orders/{}", id);
        let params: BTreeMap<String, String> = BTreeMap::new();
        let ret = self.get_signed(&uri, params)?;
        let resp: Response<RawOrderInfo> = serde_json::from_str(&ret)?;

        Ok(resp.data.into())
    }

    pub fn get_open_orders(&self, symbol: &str) -> APIResult<Vec<Order>> {
        let uri = "/v1/order/openOrders";
        let mut params: BTreeMap<String, String> = BTreeMap::new();
        params.insert("account-id".into(), self.account_id.clone());
        params.insert("symbol".into(), symbol.to_string().to_lowercase());
        let ret = self.get_signed(uri, params)?;
        let resp: Response<Vec<RawOrderInfo>> = serde_json::from_str(&ret)?;

        let orders = resp
            .data
            .into_iter()
            .map(|raw_order| raw_order.into())
            .collect::<Vec<Order>>();

        Ok(orders)
    }
}

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    use super::*;

    const HOST: &'static str = "https://api.huobi.pro";
    const API_KEY: &'static str = "2ed1ae8e-7015f4e4-85c65e29-edrfhh5h53";
    const SECRET_KEY: &'static str = "259f957f-e568adb8-5b4e5a15-be8d6";

    //#[test]
    fn test_get_symbols() {
        let api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let symbols = api.get_symbols();
        println!("symbols: {:?}", symbols);
    }

    //#[test]
    fn test_get_account_id() {
        let api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let acc_id = api.get_account_id("margin");
        println!("margin_id: {:?}", acc_id);
    }

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
    fn test_get_balance() {
        let mut api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        let acc_id = api.get_account_id("super-margin").unwrap();
        api.set_account("super-margin", &acc_id);
        let ret = api.get_balance("USDT");
        println!("{:?}", ret);
    }

    //#[test]
    fn test_orders() {
        let mut api = Huobi::new(Some(API_KEY.into()), Some(SECRET_KEY.into()), HOST.into());
        // set account_id
        let acc_id = api.get_account_id("spot").unwrap();
        api.set_account("spot", &acc_id);

        // create_order
        let order_id = api.create_order("NEXOBTC", 0.00002500, 100.0, "SELL", "LIMIT");
        println!("order_id: {:?}", order_id);

        // get_open_orders
        let open_orders = api.get_open_orders("NEXOBTC");
        println!("open_orders: {:?}", open_orders);

        // cancel_all
        let _ = api.cancel_all("NEXOBTC");

        // get_order
        let order = api.get_order(&order_id.unwrap());
        println!("order: {:?}", order);
    }
}
