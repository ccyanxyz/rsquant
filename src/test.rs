use serde_json::Value;
fn main() {
    let s = r#"{
      "status": "error",
      "ch": "market.btcusdt.kline.1day",
      "ts": 1499223904680,
      "data": 111
    }"#;

    let val: Value = serde_json::from_str(s).unwrap();
    println!("val: {:?}", val);

    if val["status"].as_str() == Some("error") {
        println!("shit");
    }
}
