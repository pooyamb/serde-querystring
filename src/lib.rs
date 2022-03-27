mod decode;
mod parsers;

pub use parsers::{
    BracketsQueryString, DuplicateQueryString, SeparatorQueryString, SimpleQueryString,
};

#[cfg(feature = "serde")]
pub mod de;
