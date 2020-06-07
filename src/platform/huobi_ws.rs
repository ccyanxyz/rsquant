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
                host: "".into(),
                subs: vec![],
                out: Some(out),
            }
        }).unwrap();
    }

    pub fn subscribe_kline(&mut self) {
        self.subs.push(r#"{"sub": "market.ethbtc.kline.1min", "id": "id1"}"#.into());
    }

    /*
    pub fn subscribe_orderbook() {

    }
    */
}

impl Handler for HuobiWs {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        match &self.out {
            Some(out) => {
                self.subs.iter().for_each(|s| {
                    out.send(s.as_str());
                })
                //out.send(r#"{"sub": "market.ethbtc.kline.1min", "id": "id1"}"#);
            },
            None => {
                println!("self.out is None");
            }
        }
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        println!("msg: {:?}", msg);
        let slice = &msg.into_data()[..];
        let mut d = GzDecoder::new(slice);
        let mut s = String::new();
        d.read_to_string(&mut s).unwrap();
        println!("decoded: {:?}", s);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_huobiws() {
        let mut huobi = HuobiWs::new("wss://api.huobi.pro/ws");
        huobi.subscribe_kline();
        huobi.connect();
    }
}
