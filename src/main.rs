use log::{info, warn};
use std::{env, fs};
use serde_json::Value;
use rsquant::{
    traits::Strategy,
    strategies::{MoveStopLoss, Dummy},
};

fn construct_robot(config_path: &str) -> Box<dyn Strategy> {
    let file = fs::File::open(config_path).expect("file should open read only");
    let config: Value = serde_json::from_reader(file).expect("file should be proper json");
    let strategy = config["strategy"].as_str().unwrap();

    match strategy {
        "move_stoploss" => {
            MoveStopLoss::new(config_path)
        },
        _ => {
            Dummy::new(config_path)
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

    let mut robot = construct_robot(&config_path);
    if robot.name() != "dummy" {
        info!("robot: {:?}", robot.stringify());
        robot.run_forever();
    } else {
        warn!("strategy not found!");
    }
}
