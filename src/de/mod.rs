use std::str;

use serde::{de, forward_to_deserialize_any};

use crate::error::{Error, Result};

mod basis;
mod constants;
mod map;
mod parser;
mod stash;
mod value;

use map::MapEntry;
use value::Value;

pub(crate) struct Deserializer<'de> {
    parser: parser::Parser<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            parser: parser::Parser::new(slice),
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, _: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotSupportedAsValue)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MapEntry::new(self))
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(MapEntry::new(self))
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.parser.peek()? == Some(b'&') {
            self.parser.discard();
        }

        visitor.visit_unit()
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(MapEntry::new(self))
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string bytes byte_buf unit_struct tuple_struct option
        identifier ignored_any tuple seq newtype_struct bool
        i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
    }
}

pub fn from_str<'de, T>(input: &'de str) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(input.as_bytes());
    serde::de::Deserialize::deserialize(&mut de)
}

pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(input);
    serde::de::Deserialize::deserialize(&mut de)
}
