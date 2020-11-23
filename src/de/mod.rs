use serde::{de, forward_to_deserialize_any};

use crate::error::{Error, Result};

mod map;
mod parser;
mod seq;
mod stash;
mod value;

use map::PairMap;
use parser::Parser;
use stash::Stash;
use value::Value;

pub(crate) struct Deserializer<'de> {
    parser: Parser<'de>,
    stash: Stash<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            parser: Parser::new(slice),
            stash: Stash::new(),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.parser.done()
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

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        while let Some(b) = self.parser.next()? {
            match b {
                b'&' | b';' => {}
                _ => return Err(Error::InvalidCharacter),
            }
        }

        visitor.visit_unit()
    }

    #[inline]
    fn deserialize_unit_struct<V>(self, _: &'static str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.is_finished() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _: &str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        while self.parser.next()?.is_some() {}
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64
        char str string bytes byte_buf tuple_struct enum
        identifier tuple seq bool
    }
}

impl<'de> de::MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if !self.parser.done() {
            let key = self.parser.parse_key(&mut self.stash)?;
            if let Some(key) = key {
                return seed
                    .deserialize(&mut Value::new(&mut Parser::new(key)))
                    .map(Some);
            }
        }

        // Visit stash
        let key = self.stash.next_key()?;
        match key {
            Some(key) => seed
                .deserialize(&mut Value::new(&mut Parser::new(key)))
                .map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        if !self.parser.done() {
            seed.deserialize(&mut Value::new(&mut self.parser))
        } else {
            seed.deserialize(PairMap::new(self.stash.next_value()?, 64))
        }
    }
}

pub fn from_str<'de, T>(input: &'de str) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(input.as_bytes());
    let res = serde::de::Deserialize::deserialize(&mut de)?;
    if !de.is_finished() {
        Err(Error::EofReached)
    } else {
        Ok(res)
    }
}

pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let mut de = Deserializer::new(input);
    let res = serde::de::Deserialize::deserialize(&mut de)?;
    if !de.is_finished() {
        Err(Error::EofReached)
    } else {
        Ok(res)
    }
}
