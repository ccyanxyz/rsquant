use std::io::prelude::*;
use ws::{Handler, Sender, Handshake, Result, Message};
use flate2::read::GzDecoder;

struct HuobiWs {
    host: String,
    client: Option<Client>,
}

impl HuobiWs {
    pub fn new(host: String) -> Self {
        HuobiWs {
            host: host,
            client: None
        }
    }

    pub fn connect(&mut self) {
        ws::connect(self.host.clone(), |out| {
            Client {
                out,
            }
        }).unwrap();
    }

    pub fn subscribe(&mut self) {
        self.client.as_ref().unwrap().out.send(r#"{"sub": "market.ethbtc.kline.1min", "id": "id1"}"#);
    }
}

struct Client {
    pub out: Sender,
}

impl Handler for Client {
    fn on_open(&mut self, shake: Handshake) -> Result<()> {
        println!("on_open");
        //self.out.send(r#"{"sub": "market.ethbtc.kline.1min", "id": "id1"}"#);
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
    /*ws::connect(url, |out| {
        Client {
            out,
        }
    }).unwrap();
    println!("client finished");
    */

    let mut huobiws = HuobiWs::new(url.into());
    huobiws.connect();
    huobiws.subscribe();
}
