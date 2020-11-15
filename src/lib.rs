use std::str;

mod de;
pub mod error;

use error::Result;

pub fn from_str<'de, T>(input: &'de str) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = de::Deserializer::new(input.as_bytes());
    serde::de::Deserialize::deserialize(&mut de)
}

pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = de::Deserializer::new(input);
    serde::de::Deserialize::deserialize(&mut de)
}
