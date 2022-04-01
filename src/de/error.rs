use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub enum ErrorKind {
    InvalidType,
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

    // The slice causing the error
    pub value: String,
    // Index of the byte in the value slice, causing the error
    pub index: Option<usize>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            message: String::new(),
            value: String::new(),
            index: None,
        }
    }

    pub(crate) fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub(crate) fn value(mut self, slice: &[u8]) -> Self {
        self.value = String::from_utf8_lossy(slice).to_string();
        self
    }

    pub(crate) fn index(mut self, index: usize) -> Self {
        self.index = Some(index);
        self
    }
}

impl _serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: fmt::Display,
    {
        Error::new(ErrorKind::Other).message(msg.to_string())
    }

    fn invalid_type(unexp: _serde::de::Unexpected, exp: &dyn _serde::de::Expected) -> Self {
        Error::new(ErrorKind::InvalidType)
            .message(format_args!("invalid type: {}, expected {}", unexp, exp).to_string())
    }
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "Error {:?}: {} in `{}`",
            self.kind, self.message, self.value
        ))
    }
}
