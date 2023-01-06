mod decode;

pub mod parsers;

#[cfg(feature = "serde")]
pub mod de;

pub use parsers::{BracketsQS, DelimiterQS, DuplicateQS, UrlEncodedQS};

#[cfg(feature = "serde")]
pub use de::{from_bytes, from_str, Error, ErrorKind, ParseMode};
