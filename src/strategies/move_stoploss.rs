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
struct Record {
    symbol: String,
    buy_price: f64,
    sell_price: f64,
    amount: f64,
    profit: f64,
}

#[derive(Debug)]
pub struct MoveStopLoss {
    config: Value,
    client: Binance,
    watch: Vec<SymbolInfo>,
    positions: Vec<Position>,
    balances: Vec<Balance>,
    history: Vec<Record>,
    total_profit: f64,

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
            .map(|coin| coin.as_str().unwrap().to_owned() + &self.quote)
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
                high: 0f64,
            })
            .collect();
    }

    fn refresh_position(&self, pos: &Position) -> APIResult<Position> {
        let mut coin = pos.symbol.clone();
        let len = self.quote.len();
        for _ in 0..len {
            coin.pop();
        }
        //let balance = self.client.get_balance(&coin)?;
        let ticker = self.client.get_ticker(&pos.symbol)?;
        let ret = self.balances.iter().find(|balance| balance.asset == coin);
        let balance = match ret {
            Some(balance) => balance,
            None => {
                warn!("{:?} not found in self.balances", coin);
                return Ok(pos.clone());
            }
        };
        if balance.free == pos.amount {
            if pos.amount * pos.price < self.min_value {
                return Ok(pos.clone());
            } else {
                let high = if ticker.bid.price > pos.high {
                    ticker.bid.price
                } else {
                    pos.high
                };
                return Ok(Position {
                    symbol: pos.symbol.clone(),
                    price: pos.price,
                    amount: pos.amount,
                    high: high,
                });
            }
        }
        if balance.free * ticker.bid.price < self.min_value {
            return Ok(Position {
                symbol: pos.symbol.clone(),
                price: 0f64,
                amount: balance.free,
                high: 0f64,
            });
        }
        // get avg_price
        let orders = self.client.get_history_orders(&pos.symbol)?;
        let mut amount = 0f64;
        let mut avg_price = 0f64;
        for order in &orders {
            if order.side == "BUY" {
                avg_price =
                    (amount * avg_price + order.filled * order.price) / (amount + order.filled);
                amount += order.amount;
            } else if order.side == "SELL" {
                avg_price =
                    (amount * avg_price - order.filled * order.price) / (amount - order.filled);
                amount -= order.amount;
            }

            if amount == balance.free {
                break;
            }
        }
        // ignore low value position
        if amount * avg_price < self.min_value {
            amount = 0f64;
            avg_price = 0f64;
        }
        // get highest price since hold
        let high = if ticker.bid.price > avg_price {
            ticker.bid.price
        } else {
            avg_price
        };
        Ok(Position {
            symbol: pos.symbol.clone(),
            amount: amount,
            price: avg_price,
            high: high,
        })
    }

    fn calc_withdraw_ratio(&self, profit_ratio: f64) -> f64 {
        if profit_ratio < self.start_threshold {
            return self.withdraw_ratio;
        }
        // y = (10*x-10*a)/10*x
        return (round_to(profit_ratio * 10f64, 0) - round_to(self.start_threshold * 10f64, 0) + 1f64) / round_to(profit_ratio * 10f64, 0); 
    }

    fn check_move_stoploss(&mut self, pos: &Position) -> APIResult<()> {
        if pos.amount * pos.price < self.min_value {
            return Ok(());
        }
        // get current price
        let ticker = self.client.get_ticker(&pos.symbol)?;
        let diff_ratio = (ticker.bid.price - pos.price) / pos.price;
        let high_ratio = (pos.high - pos.price) / pos.price;

        let stoploss_price = round_same(ticker.bid.price, pos.price * (1f64 + self.stoploss));
        // calc withdraw ratio
        let withdraw_ratio = self.calc_withdraw_ratio(diff_ratio);
        let withdraw_price = round_same(
            ticker.bid.price,
            pos.price * (1f64 + withdraw_ratio * high_ratio),
        );
        info!(
            "pos: {:?}, now_price: {:?}, profit_ratio: {:?}, stoploss_price: {:?}, withdraw_ratio: {:?}, withdraw_price: {:?}",
            pos,
			ticker.bid.price,
            round_to(diff_ratio, 4),
            stoploss_price,
            withdraw_ratio,
            withdraw_price
        );
        info!("total_profit: {:?}, history: {:?}", self.total_profit, self.history);

        let profit = round_to((ticker.bid.price - pos.price) * pos.amount, 2);

        // stoploss
        if diff_ratio <= self.stoploss {
            // sell all
            let price = round_same(ticker.bid.price, ticker.bid.price * 0.95);
            let oid = self.client.create_order(
                &pos.symbol,
                price,
                pos.amount,
                ORDER_ACTION_SELL,
                ORDER_TYPE_LIMIT,
            );
            info!(
                "{:?} stoploss triggered, sell {:?} at {:?}, order_id: {:?}",
                pos.symbol, price, pos.amount, oid
            );
            self.history.push(Record {
                symbol: pos.symbol.clone(),
                buy_price: pos.price,
                sell_price: ticker.bid.price,
                amount: pos.amount,
                profit: profit,
            });
            self.total_profit += profit;
        }
        if high_ratio >= self.start_threshold {
            if diff_ratio <= high_ratio * withdraw_ratio {
                // sell all
                let price = round_same(ticker.bid.price, ticker.bid.price * 0.95);
                let oid = self.client.create_order(
                    &pos.symbol,
                    price,
                    pos.amount,
                    ORDER_ACTION_SELL,
                    ORDER_TYPE_LIMIT,
                );
                info!(
                    "{:?} profit withdraw triggered, sell {:?} at {:?}, order_id: {:?}",
                    pos.symbol, price, pos.amount, oid
                );
                self.history.push(Record {
                    symbol: pos.symbol.clone(),
                    buy_price: pos.price,
                    sell_price: ticker.bid.price,
                    amount: pos.amount,
                    profit: profit,
                });
                self.total_profit += profit;
            }
        }
        Ok(())
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
            .positions.clone()
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
            history: vec![],

            total_profit: 0f64,
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
