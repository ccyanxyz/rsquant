use crate::errors::*;
use chrono::prelude::*;
use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> Result<u64> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_secs() * 1000 + u64::from(since_epoch.subsec_nanos()) / 1_000_000)
}

pub fn get_utc_ts() -> Result<String> {
    let dt = Utc::now();
    Ok(dt.format("%Y-%m-%dT%H:%M:%S").to_string())
}

pub fn percent_encode(url: &str) -> Result<String> {
    const FRAGMENT: &AsciiSet = &CONTROLS.add(b' ').add(b':');
    Ok(utf8_percent_encode(url, FRAGMENT).to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    //#[test]
    fn test_get_utc_ts() {
        let ret = get_utc_ts();
        println!("utc: {:?}", ret);
    }
}
