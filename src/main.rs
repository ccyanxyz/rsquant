#[macro_use]
extern crate serde_json;

use serde_json::Value;
use rsquant::models::*;

fn main() {
    let obj = json!({
        "bids": [["210", "1"], ["209", "2"]],
        "asks": [["211", "1"], ["212", "2"]],
    });
    println!("value: {:?}", obj);
    
    let bids = obj["bids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|bid| {
            Bid {
                price: bid[0].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
                amount: bid[1].as_str().unwrap().parse::<f64>().unwrap_or(0.0),
            }
        })
        .collect::<Vec<Bid>>();

    println!("bids: {:?}", bids);
}
