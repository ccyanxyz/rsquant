extern crate env_logger;
extern crate log;
extern crate rsex;
extern crate serde_json;

use log::{info, warn};
use rsex::binance::spot_rest::Binance;
use rsex::errors::APIResult;
use rsex::models::SymbolInfo;
use rsex::traits::SpotRest;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::{thread, time};

#[derive(Debug)]
struct Config {
    watch_all: bool,
    watch_list: Vec<String>,
    ignore: Vec<String>,
    stop_loss: f64,
    start_move_stoploss: f64,
}

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    amount: f64,
    price: f64,
}

#[derive(Debug)]
struct MoveStopLoss {
    config: Value,
    client: Binance,
    watch: Vec<SymbolInfo>,
    positions: Vec<Position>,
}

impl MoveStopLoss {
    pub fn new(config_path: &str) -> Self {
        let file = fs::File::open(config_path).expect("file should open read only");
        let config: Value = serde_json::from_reader(file).expect("file should be proper json");
        let apikey = config["apikey"].as_str().unwrap();
        let secret_key = config["secret_key"].as_str().unwrap();
        let host = config["host"].as_str().unwrap();

        MoveStopLoss {
            config: config.clone(),
            client: Binance::new(Some(apikey.into()), Some(secret_key.into()), host.into()),
            watch: vec![],
            positions: vec![],
        }
    }

    pub fn get_symbols(&self) -> APIResult<Vec<SymbolInfo>> {
        let quote = self.config["quote"].as_str().unwrap();
        let symbol_info = self.client.get_symbols()?;
        let symbol_info = symbol_info
            .into_iter()
            .filter(|symbol| symbol.quote == quote)
            .collect();
        Ok(symbol_info)
    }

    pub fn init(&mut self) {
        // set watch_list
        let ret = self.get_symbols();
        let symbol_info = match ret {
            Ok(symbols) => symbols,
            Err(err) => {
                warn!("get_symbols error: {:?}", err);
                vec![]
            }
        };

        // symbols - ignore
        let ignore: Vec<String> = self.config["ignore"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|coin| coin.as_str().unwrap().to_owned() + self.config["quote"].as_str().unwrap())
            .collect();

        self.watch = symbol_info
            .into_iter()
            .filter(|info| !ignore.contains(&info.symbol))
            .collect();
        info!("watch_list: {:?}", self.watch);

        self.positions = self
            .watch
            .iter()
            .map(|info| Position {
                symbol: info.symbol.clone(),
                price: 0f64,
                amount: 0f64,
            })
            .collect();
    }

    pub fn on_tick(&self) {
        // get current positions:
        // get asset balances in set(symbols-config.ignore)
        //
        // if asset.balance > 0:
        //      if asset in positions:
        //          if asset.balance == positions[asset].amount:
        //              continue
        //          else:
        //              # bought more or sold some, recalculate price, amount
        //              positions[asset].amount = asset.balance
        //      else:
        //          positions[asset] = {price, amount}

        // iterate through each position in positions
        // check move_stoploss
        //unimplemented!()
        info!("on_tick");
    }

    pub fn run_forever(&mut self) {
        self.init();
        loop {
            self.on_tick();
            thread::sleep(time::Duration::from_secs(3));
        }
    }
}

fn main() {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "./config.json"
    };
    info!("config file: {}", config_path);

    let mut robot = MoveStopLoss::new(&config_path);
	info!("robot: {:?}", robot);
    robot.run_forever();
}
