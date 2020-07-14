use crate::binance::types;
use crate::binance::types::*;
use crate::errors::*;
use crate::models::*;
use crate::traits::*;

use flate2::read::GzDecoder;
use std::io::prelude::*;
use ws::{Handler, Handshake, Message, Result, Sender};

static WEBSOCKET_URL: &'static str = "wss://stream.binance.com:9443/ws/btcusdt@depth20";
//static WEBSOCKET_URL: &'static str = "wss://stream.binancezh.com:9443";

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
    result: Option<String>,
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

    fn deseralize(&self, s: &str) -> APIResult<WsEvent> {
        if s.find("result") != None {
            let resp: ResponseEvent = serde_json::from_str(s)?;
            return Ok(WsEvent::ResponseEvent(resp));
        }
        //let val: Value = serde_json::from_str(s)?;
        if s.find("kline") != None {
            let resp: KlineEvent = serde_json::from_str(&s)?;
            Ok(WsEvent::KlineEvent(resp.kline.into()))
        } else if s.find("lastUpdateId") != None {
            let resp: RawOrderbook = serde_json::from_str(&s)?;
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

impl<'a> SpotWs for BinanceWs<'a> {
    fn sub_kline(&mut self, symbol: &str, period: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@kline_{}\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            period,
            self.subs.len() + 1,
        ));
    }

    fn sub_orderbook(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@depth20\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        ));
    }

    fn sub_trade(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@aggTrade\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        ));
    }

    fn sub_ticker(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"method\": \"SUBSCRIBE\", \"params\": [\"{}@bookTicker\"], \"id\": {}}}",
            symbol.to_string().to_lowercase(),
            self.subs.len() + 1,
        ));
    }

    fn sub_order_update(&mut self, symbol: &str) {
        unimplemented!()
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
        //println!("{:?}", msg);
        match self.deseralize(&msg.to_string()) {
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
        //binance.sub_orderbook("btcusdt");
        binance.sub_ticker("btcusdt");
        binance.sub_kline("btcusdt", "5m");
        binance.sub_trade("btcusdt");
        binance.connect(handler);
    }
}
