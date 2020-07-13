use crate::errors::*;
use chrono::prelude::*;
use percent_encoding::{define_encode_set, utf8_percent_encode, USERINFO_ENCODE_SET};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> APIResult<u64> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_secs() * 1000 + u64::from(since_epoch.subsec_nanos()) / 1_000_000)
}

pub fn get_utc_ts() -> String {
    let dt = Utc::now();
    dt.format("%Y-%m-%dT%H:%M:%S").to_string()
}

pub fn percent_encode(source: &str) -> String {
    define_encode_set! {
        pub CUSTOM_ENCODE_SET = [USERINFO_ENCODE_SET] | { '+', ',' }
    }
    utf8_percent_encode(&source, CUSTOM_ENCODE_SET).to_string()
}

pub fn to_i64(v: &Value) -> i64 {
    v.as_i64().unwrap()
}

pub fn to_f64(v: &Value) -> f64 {
    v.as_str().unwrap().parse().unwrap()
}

#[cfg(test)]
mod test {
    #![allow(dead_code)]
    use super::*;

    //#[test]
    fn test_get_utc_ts() {
        let ret = get_utc_ts();
        println!("utc: {:?}", ret);
    }
}
