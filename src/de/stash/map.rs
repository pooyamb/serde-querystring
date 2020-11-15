use std::collections::VecDeque;

use serde::{de, forward_to_deserialize_any};

use super::{seq::PairSeq, Pair, Stash};
use crate::de::Deserializer;
use crate::error::{Error, Result};

pub(crate) struct PairMap<'de> {
    pairs: VecDeque<Pair<'de>>,
    value: Option<&'de [u8]>,
    stash: Stash<'de>,
}

impl<'de> PairMap<'de> {
    pub(crate) fn new(depth: u16, pairs: VecDeque<Pair<'de>>) -> Self {
        Self {
            pairs,
            value: None,
            stash: Stash::new(depth),
        }
    }

    pub(crate) fn with_one_pair(depth: u16, pair: Pair<'de>) -> Self {
        let mut pairs = VecDeque::new();
        pairs.push_front(pair);
        Self {
            pairs,
            value: None,
            stash: Stash::new(depth),
        }
    }

    pub(crate) fn parse_pair(&mut self, pair: Pair<'de>) -> Result<Option<&'de [u8]>> {
        // Parse key
        let mut key_index = 0;
        let mut key_found = false;
        while key_index < pair.key.len() {
            match pair.key[key_index] {
                b']' => {
                    key_found = true;
                    break;
                }
                _ => {
                    key_index += 1;
                }
            }
        }

        if !key_found {
            return Err(Error::InvalidMapKey);
        }

        if pair.key.len() > key_index + 1 {
            if pair.key.len() > key_index + 2 && pair.key[key_index + 1] == b'[' {
                self.stash.add(
                    &pair.key[0..key_index],
                    &pair.key[(key_index + 2)..],
                    pair.value,
                );
                Ok(None)
            } else {
                // Cases like a[b]c=2 are invalid
                Err(Error::InvalidMapKey)
            }
        } else {
            self.value = Some(pair.value);
            Ok(Some(&pair.key[0..key_index]))
        }
    }

    pub(crate) fn next_key(&mut self) -> Result<Option<&'de [u8]>> {
        loop {
            match self.pairs.pop_back() {
                Some(pair) => match self.parse_pair(pair)? {
                    Some(key) => {
                        return Ok(Some(key));
                    }
                    None => {
                        continue;
                    }
                },
                None => return Ok(None),
            }
        }
    }

    pub(crate) fn next_value(&mut self) -> Result<&'de [u8]> {
        match self.value.take() {
            Some(value) => Ok(value),
            None => Err(Error::InvalidMapValue),
        }
    }

    pub(crate) fn into_seq(mut self) -> Result<PairSeq<'de>> {
        let mut values = VecDeque::new();
        while let Some(_) = self.next_key()? {
            values.push_front(self.next_value()?);
        }

        // TODO: support ordered sequence
        let mut pairs = VecDeque::new();
        while let Some(key) = self.stash.next_key()? {
            let mut map = self.stash.next_value_map()?;
            if key.is_empty() {
                // We should check the key
                while let Some(pair) = map.pairs.pop_back() {
                    pairs.push_front(PairMap::with_one_pair(10, pair));
                }
            } else {
                pairs.push_front(map)
            }
        }
        Ok(PairSeq::new(values, pairs, self.stash.remaining_depth))
    }
}

impl<'de> de::Deserializer<'de> for PairMap<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.pairs.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        mut self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(&mut self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(&mut self.into_seq()?)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct tuple
        tuple_struct map struct identifier ignored_any
    }
}

impl<'de> de::MapAccess<'de> for PairMap<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        // Calling next_value before next_key is an error, so we don't check the depth there
        if self.stash.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }

        if let Some(key) = self.next_key()? {
            return seed.deserialize(&mut Deserializer::new(key)).map(Some);
        }

        // Visit stash
        let key = self.stash.next_key()?;

        match key {
            Some(key) => seed.deserialize(&mut Deserializer::new(&key)).map(Some),
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.next_value() {
            Ok(value) => seed.deserialize(&mut Deserializer::new_with_depth(
                value,
                self.stash.remaining_depth - 1,
            )),
            _ => seed.deserialize(self.stash.next_value_map()?),
        }
    }
}

impl<'de> de::EnumAccess<'de> for &mut PairMap<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        // Calling next_value before next_key is an error, so we don't check the depth there
        if self.stash.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }

        let key = {
            if let Some(key) = self.next_key()? {
                key
            } else {
                // Visit stash
                let key = self.stash.next_key()?;

                match key {
                    Some(key) => key,
                    None => {
                        return Err(Error::EofReached);
                    }
                }
            }
        };

        Ok((
            seed.deserialize(&mut Deserializer::new_with_depth(
                key,
                self.stash.remaining_depth - 1,
            ))?,
            self,
        ))
    }
}

impl<'de> de::VariantAccess<'de> for &mut PairMap<'de> {
    type Error = Error;

    #[inline]
    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.next_value() {
            Ok(value) => visitor.visit_seq(&mut Deserializer::new_with_depth(
                value,
                self.stash.remaining_depth - 1,
            )),
            _ => visitor.visit_seq(&mut self.stash.next_value_map()?.into_seq()?),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self.stash.next_value_map()?)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.next_value() {
            Ok(value) => seed.deserialize(&mut Deserializer::new_with_depth(
                value,
                self.stash.remaining_depth - 1,
            )),
            _ => seed.deserialize(self.stash.next_value_map()?),
        }
    }
}
