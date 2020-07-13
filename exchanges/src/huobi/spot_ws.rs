use crate::errors::*;
use crate::huobi::types::*;
use crate::models::*;
use flate2::read::GzDecoder;
use std::io::prelude::*;
use ws::{Handler, Handshake, Message, Result, Sender};

#[derive(Debug)]
pub enum WsEvent {
    OrderbookEvent(Orderbook),
    KlineEvent(Kline),
    TickerEvent(Ticker),
    TradeEvent(Vec<Trade>),
    ResponseEvent(ResponseEvent),
    PingEvent(Ping),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseEvent {
    id: String,
    status: String,
    subbed: String,
    ts: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Ping {
    ping: i64,
}

pub struct HuobiWs<'a> {
    host: String,
    subs: Vec<String>,
    out: Option<Sender>,

    handler: Box<dyn FnMut(WsEvent) -> Result<()> + 'a>,
}

impl<'a> HuobiWs<'a> {
    pub fn new(host: &str) -> Self {
        HuobiWs {
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
        ws::connect(self.host.clone(), |out| HuobiWs {
            host: self.host.clone(),
            subs: self.subs.clone(),
            out: Some(out),
            handler: Box::new(handler.clone()),
        })
        .unwrap();
    }

    pub fn sub_kline(&mut self, symbol: &str, period: &str) {
        self.subs.push(format!(
            "{{\"sub\": \"market.{}.kline.{}\", \"id\": \"id1\"}}",
            symbol.to_string().to_lowercase(),
            period
        ));
    }

    pub fn sub_orderbook(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"sub\": \"market.{}.depth.{}\", \"id\": \"id1\"}}",
            symbol.to_string().to_lowercase(),
            "step0"
        ));
    }

    pub fn sub_trade(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"sub\": \"market.{}.trade.detail\", \"id\": \"id1\"}}",
            symbol.to_string().to_lowercase()
        ));
    }

    pub fn sub_ticker(&mut self, symbol: &str) {
        self.subs.push(format!(
            "{{\"sub\": \"market.{}.bbo\", \"id\": \"id1\"}}",
            symbol.to_string().to_lowercase()
        ));
    }

    pub fn deseralize(&self, s: &str) -> APIResult<WsEvent> {
        if s.find("ping") != None {
            let ping: Ping = serde_json::from_str(s)?;
            match &self.out {
                Some(out) => {
                    let msg = format!("{{\"pong\":{}}}", ping.ping);
                    let _ = out.send(msg.as_str());
                }
                None => {
                    println!("self.out is None");
                }
            }
            return Ok(WsEvent::PingEvent(ping));
        }
        if s.find("tick") == None {
            let resp: ResponseEvent = serde_json::from_str(s)?;
            return Ok(WsEvent::ResponseEvent(resp));
        }
        //let val: Value = serde_json::from_str(s)?;
        if s.find("kline") != None {
            let resp: Response<RawKline> = serde_json::from_str(&s)?;
            Ok(WsEvent::KlineEvent(resp.tick.into()))
        } else if s.find("depth") != None {
            let resp: Response<RawOrderbook> = serde_json::from_str(&s)?;
            Ok(WsEvent::OrderbookEvent(resp.tick.into()))
        } else if s.find("bbo") != None {
            let resp: Response<RawTicker> = serde_json::from_str(&s)?;
            Ok(WsEvent::TickerEvent(resp.tick.into()))
        } else if s.find("trade.detail") != None {
            let resp: Response<Response<Vec<RawTrade>>> = serde_json::from_str(&s)?;
            let trades = resp
                .tick
                .data
                .into_iter()
                .map(|raw_trade| raw_trade.into())
                .collect::<Vec<Trade>>();
            Ok(WsEvent::TradeEvent(trades))
        } else {
            Err(Box::new(ExError::ApiError("msg channel not found".into())))
        }
    }
}

impl<'a> Handler for HuobiWs<'a> {
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

    //#[test]
    fn test_huobiws() {
        env_logger::init();

        let handler = |event: WsEvent| {
            match event {
                WsEvent::OrderbookEvent(e) => {
                    println!("orderbook: {:?}", e);
                    let ts = get_timestamp();
                    let diff = ts.unwrap() - e.timestamp;
                    println!("diff: {:?}", diff);
                }
                _ => {
                    println!("event: {:?}", event);
                }
            }
            Ok(())
        };
        let mut huobi = HuobiWs::new("wss://api.huobi.pro/ws");
        huobi.sub_orderbook("BTCUSDT");
        huobi.connect(handler);
    }
}
