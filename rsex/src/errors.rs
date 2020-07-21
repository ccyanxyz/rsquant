use core::fmt;
use std::error::Error;

pub type APIResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone)]
pub enum ExError {
    ApiError(String),
    RateLimitExceeded(String),
    IpBanned(String),
}

impl fmt::Display for ExError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clone() {
            ExError::ApiError(why) => write!(f, "ApiError: {}", why),
            ExError::RateLimitExceeded(why) => write!(f, "RateLimitExceeded: {}", why),
            ExError::IpBanned(why) => write!(f, "IpBanned: {}", why),
        }
    }
}

impl Error for ExError {
    fn description(&self) -> &str {
        "ExError"
    }
}
