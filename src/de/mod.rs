use serde::{de, forward_to_deserialize_any};

use crate::error::{Error, Result};

mod map;
mod parser;
mod seq;
mod stash;
mod value;

use stash::Stash;
use value::Value;

pub(crate) struct Deserializer<'de> {
    parser: parser::Parser<'de>,
    value: Option<&'de [u8]>,
    stash: Stash<'de>,
}

impl<'de> Deserializer<'de> {
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            parser: parser::Parser::new(slice),
            stash: Stash::new(64),
            value: None,
        }
    }

    fn parse_children_pair(&mut self, start_index: usize) -> Result<usize> {
        if start_index + 1 > self.parser.slice.len() {
            return Err(Error::EofReached);
        }

        let mut key_index = start_index + 1;
        while key_index < self.parser.slice.len() {
            match self.parser.slice[key_index] {
                b'=' => {
                    break;
                }
                _ => {
                    key_index += 1;
                }
            }
        }

        if key_index == self.parser.slice.len() {
            // We faced early finish
            return Err(Error::InvalidMapKey);
        }

        if key_index == start_index + 1 {
            return Err(Error::InvalidMapKey);
        }

        let mut value_index = key_index + 1;
        while value_index < self.parser.slice.len() {
            match self.parser.slice[value_index] {
                b';' | b'&' => {
                    break;
                }
                _ => {
                    value_index += 1;
                }
            }
        }

        self.stash.add(
            &self.parser.slice[self.parser.index..start_index],
            &self.parser.slice[(start_index + 1)..key_index],
            &self.parser.slice[(key_index + 1)..value_index],
        );
        Ok(value_index)
    }

    fn parse_pair<'s>(&'s mut self) -> Result<Option<&'de [u8]>> {
        // Parse key
        let mut key_found = false;
        let mut key_index = self.parser.index;
        while key_index < self.parser.slice.len() {
            match self.parser.slice[key_index] {
                b'=' => {
                    key_found = true;
                    break;
                }
                b'[' => {
                    // It's a subkey
                    let end_index = self.parse_children_pair(key_index)?;
                    self.parser.index = end_index + 1;
                    key_index = end_index + 1;
                }
                _ => {
                    key_index += 1;
                }
            }
        }

        if !key_found {
            return Ok(None);
        }
        let key = &self.parser.slice[self.parser.index..key_index];

        let mut value_index = key_index + 1;
        while value_index < self.parser.slice.len() {
            match self.parser.slice[value_index] {
                b';' | b'&' => {
                    break;
                }
                _ => {
                    value_index += 1;
                }
            }
        }
        self.value = Some(&self.parser.slice[(key_index + 1)..value_index]);

        self.parser.index = value_index + 1;

        Ok(Some(key))
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
        visitor.visit_map(self)
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
        visitor.visit_map(self)
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

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string bytes byte_buf unit_struct tuple_struct option enum
        identifier ignored_any tuple seq newtype_struct bool
        i8 i16 i32 i64 u8 u16 u32 u64 f32 f64
    }
}

impl<'de> de::MapAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if !self.parser.done() {
            let key = self.parse_pair()?;
            if let Some(key) = key {
                return seed.deserialize(&mut Value::new(key)).map(Some);
            }
        }

        // Visit stash
        let key = self.stash.next_key()?;
        match key {
            Some(key) => seed.deserialize(&mut Value::new(key)).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(&mut Value::new(value)),
            None => seed.deserialize(self.stash.next_value_map()?),
        }
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
