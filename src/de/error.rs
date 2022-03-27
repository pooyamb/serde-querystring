use std::error::Error as StdError;
use std::fmt;

use serde::de;

#[derive(Debug, PartialEq)]
pub enum Error {
    Custom(String),
}

impl de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: std::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Custom(c) => f.write_str(c),
        }
    }
}
