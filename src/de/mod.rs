mod error;
mod slices;
mod traits;

use _serde::{de, forward_to_deserialize_any};

pub use error::{Error, ErrorKind};

pub(crate) mod __implementors {
    pub(crate) use super::slices::{OptionalRawSlice, ParsedSlice, RawSlice};
    pub(crate) use super::traits::{IntoDeserializer, IntoRawSlices};
}

use crate::parsers::{BracketsQS, DelimiterQS, DuplicateQS, UrlEncodedQS};

struct QSDeserializer<I, T> {
    iter: I,
    value: Option<T>,
    scratch: Vec<u8>,
}

impl<I, T> QSDeserializer<I, T> {
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            value: None,
            scratch: Vec::new(),
        }
    }
}

impl<'de, I, E, A> de::Deserializer<'de> for QSDeserializer<I, A>
where
    I: Iterator<Item = (E, A)>,
    for<'s> E: __implementors::IntoDeserializer<'de, 's>,
    for<'s> A: __implementors::IntoDeserializer<'de, 's>,
{
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de, I, E, A> de::MapAccess<'de> for QSDeserializer<I, A>
where
    I: Iterator<Item = (E, A)>,
    for<'s> E: __implementors::IntoDeserializer<'de, 's>,
    for<'s> A: __implementors::IntoDeserializer<'de, 's>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let mut scratch = Vec::new();

        if let Some((k, v)) = self.iter.next() {
            self.value = Some(v);
            seed.deserialize(k.into_deserializer(&mut scratch))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .expect("Method next_value called before next_key");
        seed.deserialize(value.into_deserializer(&mut self.scratch))
    }
}

#[derive(Clone, Copy)]
pub enum ParseMode {
    UrlEncoded,
    Duplicate,
    Delimiter(u8),
    Brackets,
}

pub fn from_bytes<'de, T>(input: &'de [u8], config: ParseMode) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    match config {
        ParseMode::UrlEncoded => {
            // A simple key=value parser
            T::deserialize(QSDeserializer::new(UrlEncodedQS::parse(input).into_iter()))
        }
        ParseMode::Duplicate => {
            // A parser with duplicated keys interpreted as sequence
            T::deserialize(QSDeserializer::new(DuplicateQS::parse(input).into_iter()))
        }
        ParseMode::Delimiter(s) => {
            // A parser with sequences of values seperated by one character
            T::deserialize(QSDeserializer::new(
                DelimiterQS::parse(input, s).into_iter(),
            ))
        }
        ParseMode::Brackets => {
            // A PHP like interpretation of querystrings
            T::deserialize(QSDeserializer::new(BracketsQS::parse(input).into_iter()))
        }
    }
}

pub fn from_str<'de, T>(input: &'de str, config: ParseMode) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    from_bytes(input.as_bytes(), config)
}
