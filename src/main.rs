use std::env;
use rsquant::traits::Spot;
use rsquant::platform::binance::Binance;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut symbol = "BTCUSDT".to_string();
    if args.len() > 1 {
        symbol = args[1].to_uppercase();
    }
    if !symbol.ends_with("USDT") {
        symbol += "USDT";
    }

    let api = Binance::new(None, None, "https://www.binancezh.com".into());
    let ret = api.get_ticker(&symbol);
    println!("{:?}", ret.unwrap());
}
