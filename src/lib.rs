#![doc = include_str!("../README.md")]

mod decode;

#[doc(hidden)]
pub mod parsers;

#[cfg(feature = "serde")]
#[doc(hidden)]
pub mod de;

pub use parsers::{BracketsQS, DelimiterQS, DuplicateQS, UrlEncodedQS};

#[cfg(feature = "serde")]
#[doc(inline)]
pub use de::{from_bytes, from_str, Error, ErrorKind, ParseMode};
