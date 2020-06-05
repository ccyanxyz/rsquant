use core::fmt;
use std::error::Error;

pub type APIResult<T> = Result<T, Box<std::error::Error>>;

#[derive(Debug, Clone)]
pub enum ExError {
    ApiError(String),
}

impl fmt::Display for ExError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.clone() {
            ExError::ApiError(why) => write!(f, "ApiError: {}", why),
        }
    }
}

impl Error for ExError {
    fn description(&self) -> &str {
        "ExError"
    }
}
