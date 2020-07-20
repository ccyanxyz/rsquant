extern crate env_logger;
extern crate log;
extern crate rsex;
extern crate serde_json;

use log::{info, warn, debug};
use rsex::binance::spot_rest::Binance;
use rsex::errors::APIResult;
use rsex::models::SymbolInfo;
use rsex::traits::SpotRest;
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::{thread, time};

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
        debug!("client.get_symbols: {:?}", symbol_info);
        let symbol_info = symbol_info
            .into_iter()
            .filter(|symbol| symbol.quote.to_lowercase() == quote)
            .collect();
        Ok(symbol_info)
    }

    pub fn init(&mut self) {
        // set watch_list
        let ret = self.get_symbols();
        debug!("get_symbols: {:?}", ret);
        let symbol_info = match ret {
            Ok(symbols) => symbols,
            Err(err) => {
                warn!("get_symbols error: {:?}", err);
                vec![]
            }
        };
        debug!("symbol_info: {:?}", symbol_info);

        // symbols - ignore
        let ignore: Vec<String> = self.config["ignore"]
            .as_array()
            .unwrap()
            .into_iter()
            .map(|coin| coin.as_str().unwrap().to_owned() + self.config["quote"].as_str().unwrap())
            .collect();
        info!("ignore: {:?}", ignore);

        self.watch = symbol_info
            .into_iter()
            .filter(|info| !ignore.contains(&info.symbol.to_lowercase()))
            .collect();
        debug!("watch_list: {:?}", self.watch);

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

    pub fn refresh_position(&self, pos: &Position) -> APIResult<Position> {
        let mut coin = pos.symbol.clone();
        let len = self.config["quote"].as_str().unwrap().len();
        for _ in (0..len) {
            coin.pop();
        }
        let balance = self.client.get_balance(&coin)?;
        if balance.free == pos.amount {
            return Ok(pos.clone());
        }
        // get avg_price
        let orders = self.client.get_history_orders(&pos.symbol)?;
        let mut amount = 0f64;
        let mut avg_price = 0f64;
        for order in &orders {
            if order.side == "BUY" {
                avg_price = (amount * avg_price + order.filled * order.price) / (amount + order.filled);
                amount += order.amount;
            } else if order.side == "SELL" {
                avg_price = (amount * avg_price - order.filled * order.price) / (amount - order.filled);
                amount -= order.amount;
            }

            if amount == balance.free {
                break;
            }
        }
        let min_value = self.config["min_value"].as_i64().unwrap() as f64;
        if amount * avg_price < min_value {
            amount = 0f64;
            avg_price = 0f64;
        }
        Ok(Position {
            symbol: pos.symbol.clone(),
            amount: amount,
            price: avg_price,
        })
    }

    pub fn check_move_stoploss(&self, pos: &Position) -> APIResult<Position> {
        info!("check_move_stoploss");
        Ok(pos.clone())
    }

    pub fn on_tick(&mut self) {
        self.positions = self.positions.iter().map(|pos| {
            let new_pos = self.refresh_position(&pos);
			//info!("new_pos: {:?}", new_pos);
            let new_pos = match new_pos {
                Ok(new_pos) => new_pos,
                Err(err) => {
                    warn!("refresh_position error: {:?}", err);
                    Position {
                        symbol: pos.symbol.clone(),
                        amount: 0f64,
                        price: 0f64,
                    }
                }
            };
			if new_pos.amount > 0f64 {
            	info!("old_pos: {:?}, new_pos: {:?}", pos, new_pos);
			}
            let new_pos = self.check_move_stoploss(&new_pos);
            match new_pos {
                Ok(new_pos) => new_pos,
                Err(err) => {
                    warn!("check_move_stoploss error: {:?}", err);
                    Position {
                        symbol: pos.symbol.clone(),
                        amount: 0f64,
                        price: 0f64,
                    }
                }
            }
        }).collect();
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
