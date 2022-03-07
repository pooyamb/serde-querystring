//! # Serde-querystring
//! serde-querystring is a deserializer for query strings.
//!
//! To see what's supported and what's not, please take a look the tests in the main repo.
//!

mod de;
mod error;

pub use de::{from_bytes, from_str};
pub use error::{Error, ErrorKind};
