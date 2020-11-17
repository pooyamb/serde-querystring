use std::fmt;

// TODO: Better error handling, possibly with position reporting
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    InvalidMapKey,
    InvalidMapValue,
    InvalidString,
    InvalidCharacter,
    InvalidNumber,
    InvalidIdent,
    EofReached,
    ExpectedSeprator,
    MaximumDepthReached,
    NotSupportedAsValue,
    Custom(String),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
