use log::{debug, info, warn};
use std::{fs, {thread, time}};
use serde_json::Value;

use rsex::{
    binance::spot_rest::Binance,
    errors::APIResult,
    models::{SymbolInfo, Balance},
    traits::SpotRest,
    constant::{ORDER_TYPE_LIMIT, ORDER_ACTION_SELL},
};

use crate::{
    traits::Strategy,
    utils::{round_same, round_to},
};

#[derive(Debug, Clone)]
struct Position {
    symbol: String,
    amount: f64,
    price: f64,
    high: f64,
}

#[derive(Debug)]
pub struct MoveStopLoss {
    config: Value,
    client: Binance,
    watch: Vec<SymbolInfo>,
    positions: Vec<Position>,
    balances: Vec<Balance>,

    quote: String,
    min_value: f64,
    stoploss: f64,
    start_threshold: f64,
    withdraw_ratio: f64,
}

impl MoveStopLoss {
    fn get_symbols(&self) -> APIResult<Vec<SymbolInfo>> {
        let symbol_info = self.client.get_symbols()?;
        debug!("client.get_symbols: {:?}", symbol_info);
        let symbol_info = symbol_info
            .into_iter()
            .filter(|symbol| symbol.quote.to_lowercase() == self.quote)
            .collect();
        Ok(symbol_info)
    }

    fn init(&mut self) {

    }

    fn on_tick(&mut self) {
        let ret = self.client.get_all_balances();
        if let Ok(balances) = ret {
            self.balances = balances;
        } else {
            warn!("get_all_balances error: {:?}", ret);
            return;
        }
        self.positions = self
            .positions
            .iter()
            .map(|pos| {
                let new_pos = self.refresh_position(&pos);
                //info!("new_pos: {:?}", new_pos);
                let new_pos = match new_pos {
                    Ok(new_pos) => new_pos,
                    Err(err) => {
                        warn!("refresh_position error: {:?}", err);
                        pos.clone()
                    }
                };
                if new_pos.amount > 0f64 {
                    debug!("old_pos: {:?}, new_pos: {:?}", pos, new_pos);
                }
                let ret = self.check_move_stoploss(&new_pos);
                if let Err(err) = ret {
                    warn!("check_move_stoploss error: {:?}", err);
                }
                new_pos
            })
            .collect();
    }
}

impl Strategy for MoveStopLoss {
    fn new(config_path: &str) -> Box<dyn Strategy> {
        let file = fs::File::open(config_path).expect("file should open read only");
        let config: Value = serde_json::from_reader(file).expect("file should be proper json");
        let quote = config["quote"].as_str().unwrap();
        let apikey = config["apikey"].as_str().unwrap();
        let secret_key = config["secret_key"].as_str().unwrap();
        let host = config["host"].as_str().unwrap();
        let min_value = config["min_value"].as_f64().unwrap();
        let stoploss = config["stoploss"].as_f64().unwrap();
        let start_threshold = config["start_threshold"].as_f64().unwrap();
        let withdraw_ratio = config["withdraw_ratio"].as_f64().unwrap();

        Box::new(MoveStopLoss {
            config: config.clone(),
            client: Binance::new(Some(apikey.into()), Some(secret_key.into()), host.into()),
            watch: vec![],
            positions: vec![],
            balances: vec![],

            quote: quote.into(),
            min_value: min_value,
            stoploss: stoploss,
            start_threshold: start_threshold,
            withdraw_ratio: withdraw_ratio,
        })
    }

    fn run_forever(&mut self) {
        self.init();
        loop {
            self.on_tick();
            thread::sleep(time::Duration::from_secs(60));
        }
    }

    fn name(&self) -> String {
        "move_stoploss".into()
    }

    fn stringify(&self) -> String {
        format!("{:?}", self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_move_stoploss() {
        env_logger::init();
        let config_path = "./config.json";
        info!("config file: {}", config_path);

        let mut robot = MoveStopLoss::new(&config_path);
        info!("robot: {:?}", robot);
        robot.run_forever();
    }
}
