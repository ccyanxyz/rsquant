use crate::binance::types;
use crate::errors::*;
use crate::models::*;
use crate::binance::types::*;

use flate2::read::GzDecoder;
use std::io::prelude::*;
use ws::{Handler, Handshake, Message, Result, Sender};

//static WEBSOCKET_URL: &'static str = "wss://stream.binance.com:9443/ws/";
static WEBSOCKET_URL: &'static str = "wss://stream.binancezh.com:9443";

static OUTBOUND_ACCOUNT_INFO: &'static str = "outboundAccountInfo";
static EXECUTION_REPORT: &'static str = "executionReport";

static KLINE: &'static str = "kline";
static AGGREGATED_TRADE: &'static str = "aggTrade";
static DEPTH_ORDERBOOK: &'static str = "depthUpdate";
static PARTIAL_ORDERBOOK: &'static str = "lastUpdateId";

static DAYTICKER: &'static str = "24hrTicker";

#[derive(Debug)]
pub enum WsEvent {
    // public stream
    OrderbookEvent(Orderbook),
    KlineEvent(Kline),
    TickerEvent(Ticker),
    TradeEvent(Trade),
    ResponseEvent(ResponseEvent),

    // private stream
    AccountUpdateEvent(AccountUpdateEvent),
    OrderTradeEvent(OrderTradeEvent),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseEvent {
    id: u64,
    result: String,
}

pub struct BinanceWs<'a> {
    host: String,
    subs: Vec<String>,
    out: Option<Sender>,

    handler: Box<dyn FnMut(WsEvent) -> Result<()> + 'a>,
}

impl<'a> BinanceWs<'a> {
    pub fn new(host: &str) -> Self {
        BinanceWs {
            host: host.into(),
            subs: vec![],
            out: None,
            handler: Box::new(|event| {
                println!("event: {:?}", event);
                Ok(())
            }),
        }
    }

    pub fn connect<Callback: Clone>(&mut self, handler: Callback)
    where
        Callback: FnMut(WsEvent) -> Result<()> + 'a,
    {
        ws::connect(self.host.clone(), |out| BinanceWs {
            host: self.host.clone(),
            subs: self.subs.clone(),
            out: Some(out),
            handler: Box::new(handler.clone()),
        })
        .unwrap();
    }

    pub fn sub_kline(&mut self, symbol: &str, period: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@kline_{}\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            period,
            self.subs.len() + 1,
        ));
    }

    pub fn sub_orderbook(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@depth20\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        ));
    }

    pub fn sub_trade(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@aggTrade\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        ));
    }

    pub fn sub_ticker(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@bookTicker\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        ));
    }

    pub fn deseralize(&self, s: &str) -> APIResult<WsEvent> {
        if s.find("result") != None {
            let resp: ResponseEvent = serde_json::from_str(s)?;
            return Ok(WsEvent::ResponseEvent(resp));
        }
        //let val: Value = serde_json::from_str(s)?;
        if s.find("kline") != None {
            let resp: KlineEvent = serde_json::from_str(&s)?;
            Ok(WsEvent::KlineEvent(resp.kline.into()))
        } else if s.find("lastUpdateId") != None {
            let resp: DepthOrderbookEvent = serde_json::from_str(&s)?;
            Ok(WsEvent::OrderbookEvent(resp.into()))
        } else if s.find("aggTrade") != None {
            let resp: TradeEvent = serde_json::from_str(&s)?;
            Ok(WsEvent::TradeEvent(resp.into()))
        } else if s.find("A") != None && s.find("B") != None {
            let resp: BookTickerEvent = serde_json::from_str(&s)?;
            Ok(WsEvent::TickerEvent(resp.into()))
        } else {
            Err(Box::new(ExError::ApiError("msg channel not found".into())))
        }
    }
}

impl<'a> Handler for BinanceWs<'a> {
    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        match &self.out {
            Some(out) => self.subs.iter().for_each(|s| {
                let _ = out.send(s.as_str());
            }),
            None => {
                println!("self.out is None");
            }
        }
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let slice = &msg.into_data()[..];
        let mut d = GzDecoder::new(slice);
        let mut s = String::new();
        d.read_to_string(&mut s).unwrap();
        match self.deseralize(&s) {
            Ok(event) => {
                let _ = (self.handler)(event);
            }
            Err(err) => {
                println!("deseralize msg error: {:?}", err);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utils::get_timestamp;

    #[test]
    fn test_binancews() {
        env_logger::init();

        let handler = |event: WsEvent| {
            match event {
                WsEvent::OrderbookEvent(e) => {
                    println!("orderbook: {:?}", e);
                }
                _ => {
                    println!("event: {:?}", event);
                }
            }
            Ok(())
        };
        let mut binance = BinanceWs::new(WEBSOCKET_URL);
        binance.sub_orderbook("BTCUSDT");
        binance.connect(handler);
    }
}
