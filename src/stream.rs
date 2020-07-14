extern crate rsex;

use std::env;
use rsex::binance::spot_ws::{BinanceWs, WsEvent};
use rsex::traits::SpotWs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut symbol = "BTCUSDT".to_string();
    if args.len() > 1 {
        symbol = args[1].to_uppercase();
    }
    if !symbol.ends_with("USDT") {
        symbol += "USDT";
    }

    let handler = |event: WsEvent| {
        match event {
            WsEvent::TickerEvent(e) => {
                println!("{:?}", e);
            },
            _ => {  }
        }
        Ok(())
    };

    let url = "wss://stream.binancezh.com:9443/ws/btcusdt@depth20";
    let mut ws = BinanceWs::new(url.into());
    ws.sub_ticker(&symbol);
    ws.connect(handler);
}
