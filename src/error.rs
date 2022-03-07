use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    UnexpectedType,
    InvalidLength,
    InvalidEncoding,
    InvalidNumber,
    InvalidBoolean,
    Other,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,

    // The key causing the error
    pub key: String,

    // The value causing the error
    pub value: Option<String>,

    // The index of first problematic byte in the value
    // If the value is None, index will be None too
    pub index: Option<usize>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            message: String::new(),
            key: String::new(),
            value: None,
            index: None,
        }
    }

    pub(crate) fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub(crate) fn key(mut self, key: &[u8]) -> Self {
        self.key = String::from_utf8_lossy(key).to_string();
        self
    }

    pub(crate) fn value(mut self, value: &[u8]) -> Self {
        self.value = Some(String::from_utf8_lossy(value).to_string());
        self
    }

    pub(crate) fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::new(ErrorKind::Other).message(msg.to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(v) => match self.index {
                Some(i) => f.write_fmt(format_args!(
                    "Error {:?}: {} in '{}={}' at index {}",
                    self.kind,
                    self.message,
                    self.key,
                    v,
                    i + self.key.len() + 2
                )),
                None => f.write_fmt(format_args!(
                    "Error {:?}: {} in '{}={}'",
                    self.kind, self.message, self.key, v
                )),
            },
            None => f.write_fmt(format_args!(
                "Error {:?}: {} for key '{}'",
                self.kind, self.message, self.key
            )),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
