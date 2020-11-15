use serde::de;
use serde::de::IntoDeserializer;

use super::stash::Stash;
use super::Deserializer;
use crate::error::{Error, Result};

pub(crate) struct MapEntry<'de, 'a> {
    pub(crate) de: &'a mut Deserializer<'de>,
    value: Option<&'de [u8]>,
    stash: Stash<'de>,
}

impl<'de, 'a> MapEntry<'de, 'a> {
    pub(crate) fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self {
            stash: Stash::new(de.remaining_depth - 1),
            value: None,
            de,
        }
    }

    fn parse_children_pair(&mut self, start_index: usize) -> Result<usize> {
        if start_index + 1 > self.de.parser.slice.len() {
            return Err(Error::EofReached);
        }

        let mut key_index = start_index + 1;
        while key_index < self.de.parser.slice.len() {
            match self.de.parser.slice[key_index] {
                b'=' => {
                    break;
                }
                _ => {
                    key_index += 1;
                }
            }
        }

        if key_index == self.de.parser.slice.len() {
            // We faced early finish
            return Err(Error::InvalidMapKey);
        }

        if key_index == start_index + 1 {
            return Err(Error::InvalidMapKey);
        }

        let mut value_index = key_index + 1;
        while value_index < self.de.parser.slice.len() {
            match self.de.parser.slice[value_index] {
                b';' | b'&' => {
                    break;
                }
                _ => {
                    value_index += 1;
                }
            }
        }

        self.stash.add(
            &self.de.parser.slice[self.de.parser.index..start_index],
            &self.de.parser.slice[(start_index + 1)..key_index],
            &self.de.parser.slice[(key_index + 1)..value_index],
        );
        Ok(value_index)
    }

    fn parse_pair<'s>(&'s mut self) -> Result<Option<&'de [u8]>> {
        // Parse key
        let mut key_found = false;
        let mut key_index = self.de.parser.index;
        while key_index < self.de.parser.slice.len() {
            match self.de.parser.slice[key_index] {
                b'=' => {
                    key_found = true;
                    break;
                }
                b'[' => {
                    // It's a subkey
                    let end_index = self.parse_children_pair(key_index)?;
                    self.de.parser.index = end_index + 1;
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
        let key = &self.de.parser.slice[self.de.parser.index..key_index];

        let mut value_index = key_index + 1;
        while value_index < self.de.parser.slice.len() {
            match self.de.parser.slice[value_index] {
                b';' | b'&' => {
                    break;
                }
                _ => {
                    value_index += 1;
                }
            }
        }
        self.value = Some(&self.de.parser.slice[(key_index + 1)..value_index]);

        self.de.parser.index = value_index + 1;

        Ok(Some(key))
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapEntry<'de, 'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if !self.de.parser.done() {
            let key = self.parse_pair()?;
            if let Some(key) = key {
                return seed.deserialize(&mut Deserializer::new(key)).map(Some);
            }
        }

        // Visit stash
        let key = self.stash.next_key()?;
        match key {
            Some(key) => {
                let mut de = Deserializer::new(&key);
                seed.deserialize(&mut de).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => {
                let mut de = Deserializer::new_with_depth(value, self.de.remaining_depth - 1);
                seed.deserialize(&mut de)
            }
            None => seed.deserialize(self.stash.next_value_map()?),
        }
    }
}

impl<'de, 'a> de::EnumAccess<'de> for MapEntry<'de, 'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        // For enums in the form of variant=value
        if !self.de.parser.done() {
            if let Some(key) = self.parse_pair()? {
                return seed
                    .deserialize(&mut Deserializer::new(key))
                    .map(|res| (res, self));
            }
        }

        // Visit stash
        let key = self.stash.next_key()?;
        match key {
            Some(key) => {
                let mut de = Deserializer::new(&key);
                seed.deserialize(&mut de).map(|res| (res, self))
            }
            None => {
                // Just visit one single token if available, it is here to cover enum unit values
                let key = std::str::from_utf8(self.de.parser.parse_token()?)
                    .map_err(|_| Error::InvalidString)?;
                seed.deserialize(key.into_deserializer())
                    .map(|res| (res, self))
            }
        }
    }
}

impl<'de, 'a> de::VariantAccess<'de> for MapEntry<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn tuple_variant<V>(mut self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.value.take() {
            Some(value) => {
                let mut de = Deserializer::new_with_depth(value, self.de.remaining_depth - 1);
                visitor.visit_seq(&mut de)
            }
            None => visitor.visit_seq(self.stash.next_value_map()?.into_seq()?),
        }
    }

    fn struct_variant<V>(mut self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // We already visited the key in enumaccess above
        let _ = self.stash.next_key()?;
        visitor.visit_map(self.stash.next_value_map()?)
    }

    fn newtype_variant_seed<T>(mut self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => {
                let mut de = Deserializer::new_with_depth(value, self.de.remaining_depth - 1);
                seed.deserialize(&mut de)
            }
            None => seed.deserialize(self.stash.next_value_map()?),
        }
    }
}
