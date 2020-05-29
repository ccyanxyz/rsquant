use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ExErrorInfo {
    pub code: i64,
    pub msg: String,
}

error_chain! {
    errors {
        ExError(response: ExErrorInfo)
    }

    foreign_links {
        ReqError(reqwest::Error);
        InvalidHeaderError(reqwest::header::InvalidHeaderValue);
        IoError(std::io::Error);
        ParseFloatError(std::num::ParseFloatError);
        UrlParserError(url::ParseError);
        Json(serde_json::Error);
        Tungstenite(tungstenite::Error);
        TimestampError(std::time::SystemTimeError);
    }
}
