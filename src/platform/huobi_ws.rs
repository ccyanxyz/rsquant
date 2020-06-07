use std::io::prelude::*;
use ws::{Handler, Sender, Handshake, Result, Message};
use flate2::read::GzDecoder;

struct HuobiWs {
    host: String,
    subs: Vec<String>,
    out: Option<Sender>,
}

impl HuobiWs {
    pub fn new(host: &str) -> Self {
        HuobiWs {
            host: host.into(),
            subs: vec![],
            out: None,
        }
    }

    pub fn connect(&mut self) {
        ws::connect(self.host.clone(), |out| {
            HuobiWs {
                host: self.host.clone(),
                subs: self.subs.clone(),
                out: Some(out),
            }
        }).unwrap();
    }

    pub fn sub_kline(&mut self, symbol: &str, period: &str) {
        self.subs.push(format!("{{\"sub\": \"market.{}.kline.{}\", \"id\": \"id1\"}}", symbol.to_string().to_lowercase(), period));
    }

    pub fn sub_orderbook(&mut self, symbol: &str) {
        self.subs.push(format!("{{\"sub\": \"market.{}.depth.{}\", \"id\": \"id1\"}}", symbol.to_string().to_lowercase(), "step0"));
    }

    pub fn sub_trade(&mut self, symbol: &str) {
        self.subs.push(format!("{{\"sub\": \"market.{}.trade.detail\", \"id\": \"id1\"}}", symbol.to_string().to_lowercase()));
    }

    pub fn sub_ticker(&mut self, symbol: &str) {
        self.subs.push(format!("{{\"sub\": \"market.{}.bbo\", \"id\": \"id1\"}}", symbol.to_string().to_lowercase()));
    }
}

impl Handler for HuobiWs {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        match &self.out {
            Some(out) => {
                self.subs.iter().for_each(|s| {
                    out.send(s.as_str());
                })
            },
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
        println!("{:?}", s);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_huobiws() {
        let mut huobi = HuobiWs::new("wss://api.huobi.pro/ws");
        huobi.sub_ticker("BTCUSDT");
        huobi.connect();
    }
}
