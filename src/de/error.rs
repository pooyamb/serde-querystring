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

    // The slice causing the error
    pub slice: String,

    // The index of first problematic byte in the slice
    pub index: Option<usize>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Error {
            kind,
            message: String::new(),
            slice: String::new(),
            index: None,
        }
    }

    pub(crate) fn message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub(crate) fn slice(mut self, value: &[u8]) -> Self {
        self.slice = String::from_utf8_lossy(value).to_string();
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
        f.write_fmt(format_args!(
            "Error {:?}: {} in `{}`",
            self.kind, self.message, self.slice
        ))
    }
}
