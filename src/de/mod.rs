mod error;
mod slices;
mod traits;

use serde::{de, forward_to_deserialize_any};

pub use error::Error;

pub(crate) mod __implementors {
    pub(crate) use super::slices::{OptionalRawSlice, ParsedSlice, RawSlice};
    pub(crate) use super::traits::{IntoDeserializer, IntoSizedIterator};
}

use crate::parsers::{BracketsQS, DelimiterQS, DuplicateQS, UrlEncodedQS};

pub struct QSDeserializer<I, T> {
    iter: I,
    value: Option<T>,
    scratch: Vec<u8>,
}

impl<I, T> QSDeserializer<I, T> {
    pub fn new<'de, E, A>(iter: I) -> Self
    where
        I: Iterator<Item = (E, A)>,
        for<'a> E: __implementors::IntoDeserializer<'de, 'a>,
        for<'a> A: __implementors::IntoDeserializer<'de, 'a>,
    {
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

pub enum Config {
    UrlEncoded,
    Duplicate,
    Delimiter(u8),
    Brackets,
}

pub fn from_bytes<'de, T>(input: &'de [u8], config: Config) -> Result<T, Error>
where
    T: de::Deserialize<'de>,
{
    match config {
        Config::UrlEncoded => {
            // A simple key=value parser
            T::deserialize(QSDeserializer::new(UrlEncodedQS::parse(input).into_iter()))
        }
        Config::Duplicate => {
            // A parser with duplicated keys interpreted as sequence
            T::deserialize(QSDeserializer::new(DuplicateQS::parse(input).into_iter()))
        }
        Config::Delimiter(s) => {
            // A parser with sequences of values seperated by one character
            T::deserialize(QSDeserializer::new(
                DelimiterQS::parse(input, s).into_iter(),
            ))
        }
        Config::Brackets => {
            // A PHP like interpretation of querystrings
            T::deserialize(QSDeserializer::new(BracketsQS::parse(input).into_iter()))
        }
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use crate::{de::QSDeserializer, parsers};

    #[test]
    fn deserialize_simple() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Simple<'a> {
            #[serde(borrow)]
            foo: &'a str,
            foobar: u32,
            bar: Option<u32>,
        }

        let slice = b"foo=bar&foobar=1337&foo=baz&bar=13";

        let qs = parsers::UrlEncodedQS::parse(slice);
        let de = QSDeserializer::new(qs.into_iter());

        assert_eq!(
            Simple::deserialize(de),
            Ok(Simple {
                foo: "baz",
                foobar: 1337,
                bar: Some(13)
            })
        )
    }

    #[test]
    fn deserialize_duplicate() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Simple<'a> {
            #[serde(borrow)]
            foo: &'a str,
            foobar: u32,
            bar: Option<u32>,
            vec: Vec<u32>,
        }

        let slice = b"foo=bar&foobar=1337&foo=baz&bar=13&\
                        vec=1337&vec=11";

        let qs = parsers::DuplicateQS::parse(slice);
        let de = QSDeserializer::new(qs.into_iter());

        assert_eq!(
            Simple::deserialize(de),
            Ok(Simple {
                foo: "baz",
                foobar: 1337,
                bar: Some(13),
                vec: vec![1337, 11]
            })
        )
    }

    #[test]
    fn deserialize_separator() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Simple<'a> {
            #[serde(borrow)]
            foo: &'a str,
            foobar: u32,
            bar: Option<u32>,
            vec: Vec<u32>,
        }

        let slice = b"foo=bar&foobar=1337&foo=baz&bar=13&\
                        vec=1337|11";

        let qs = parsers::DelimiterQS::parse(slice, b'|');
        let de = QSDeserializer::new(qs.into_iter());

        assert_eq!(
            Simple::deserialize(de),
            Ok(Simple {
                foo: "baz",
                foobar: 1337,
                bar: Some(13),
                vec: vec![1337, 11]
            })
        )
    }

    #[test]
    fn deserialize_brackets() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Simple<'a> {
            #[serde(borrow)]
            foo: &'a str,
            foobar: u32,
            bar: Option<u32>,
            vec: Vec<u32>,
        }

        let slice = b"foo=bar&foobar=1337&foo=baz&bar=13&\
                        vec[1]=1337&vec=11";

        let qs = parsers::BracketsQS::parse(slice);
        let de = QSDeserializer::new(qs.into_iter());

        assert_eq!(
            Simple::deserialize(de),
            Ok(Simple {
                foo: "baz",
                foobar: 1337,
                bar: Some(13),
                vec: vec![11, 1337]
            })
        );

        #[derive(Debug, Deserialize, PartialEq)]
        struct OneField<'a> {
            #[serde(borrow)]
            bar: &'a str,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Sample2<'a> {
            #[serde(borrow)]
            foo: OneField<'a>,
            #[serde(borrow)]
            qux: OneField<'a>,
        }

        let slice = b"foo[bar]=baz&qux[bar]=foobar";

        let qs = parsers::BracketsQS::parse(slice);
        let de = QSDeserializer::new(qs.into_iter());

        assert_eq!(
            Sample2::deserialize(de),
            Ok(Sample2 {
                foo: OneField { bar: "baz" },
                qux: OneField { bar: "foobar" }
            })
        )
    }
}
