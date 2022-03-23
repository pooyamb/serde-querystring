mod de;
mod decode;
mod parsers;

use de::QSDeserializer;
pub use parsers::{
    BracketsQueryString, DuplicateQueryString, SeparatorQueryString, SimpleQueryString,
};
use serde::Deserialize;

pub use de::Error;

pub enum Config {
    Simple,
    Duplicate,
    Separator(u8),
    Brackets,
}

pub fn from_bytes<'de, T>(input: &'de [u8], config: Config) -> Result<T, de::Error>
where
    T: Deserialize<'de>,
{
    match config {
        Config::Simple => {
            // A simple key=value parser
            T::deserialize(QSDeserializer::new(
                SimpleQueryString::parse(input).into_iter(),
            ))
        }
        Config::Duplicate => {
            // A parser with duplicated keys interpreted as sequence
            T::deserialize(QSDeserializer::new(
                DuplicateQueryString::parse(input).into_iter(),
            ))
        }
        Config::Separator(s) => {
            // A parser with sequences of values seperated by one character
            T::deserialize(QSDeserializer::new(
                SeparatorQueryString::parse(input, s).into_iter(),
            ))
        }
        Config::Brackets => {
            // A PHP like interpretation of querystrings
            T::deserialize(QSDeserializer::new(
                BracketsQueryString::parse(input).into_iter(),
            ))
        }
    }
}
