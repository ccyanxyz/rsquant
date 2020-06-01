use std::env;
use rsquant::traits::Spot;
use rsquant::platform::binance::Binance;

fn main() {
    let args: Vec<String> = env::args().collect();
    let symbol = args[1].to_uppercase();

    let api = Binance::new(None, None, "https://www.binancezh.com".into());
    let ret = api.get_ticker(&symbol);
    println!("{:?}", ret.unwrap());
}
