use flate2::read::GzDecoder;
use std::io::prelude::*;
use ws::{Handler, Handshake, Message, Result, Sender};

struct Client {
    out: Sender,
}

impl Handler for Client {
    fn on_open(&mut self, _shake: Handshake) -> Result<()> {
        println!("on_open");
        self.out
            .send(r#"{"sub": "market.ethbtc.kline.1min", "id": "id1"}"#);
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

fn main() {
    env_logger::init();

    let url = "wss://api.huobi.pro/ws";
    ws::connect(url, |out| Client { out }).unwrap();
    println!("client finished");
}
