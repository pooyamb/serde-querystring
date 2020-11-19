use std::cmp::Ordering;
use std::collections::VecDeque;

use serde::{de, forward_to_deserialize_any};

use super::seq::{ItemKind, PairSeq};
use super::stash::{Pair, Stash};
use super::{Parser, Value};
use crate::error::{Error, Result};

pub(crate) struct PairMap<'de> {
    pairs: VecDeque<Pair<'de>>,
    value: Option<&'de [u8]>,
    stash: Stash<'de>,
    remaining_depth: u16,
}

#[derive(Copy, Clone)]
enum KeyType<'a> {
    Number(u16),
    Name(&'a [u8]),
    None,
}

impl<'a> PartialOrd for KeyType<'a> {
    fn partial_cmp(&self, other: &KeyType) -> Option<Ordering> {
        if let KeyType::Number(num1) = self {
            if let KeyType::Number(num2) = other {
                Some(num1.cmp(num2))
            } else {
                Some(Ordering::Greater)
            }
        } else if let KeyType::Number(_) = other {
            Some(Ordering::Less)
        } else {
            None
        }
    }
}

impl<'a> PartialEq for KeyType<'a> {
    fn eq(&self, other: &KeyType<'a>) -> bool {
        if let KeyType::Number(num1) = self {
            if let KeyType::Number(num2) = other {
                num1 == num2
            } else {
                false
            }
        } else if let KeyType::Name(name1) = self {
            if let KeyType::Name(name2) = other {
                name1 == name2
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl<'de> PairMap<'de> {
    pub(crate) fn new(pairs: VecDeque<Pair<'de>>, remaining_depth: u16) -> Self {
        Self {
            pairs,
            value: None,
            stash: Stash::new(),
            remaining_depth,
        }
    }

    pub(crate) fn prepend(&mut self, mut pairs: VecDeque<Pair<'de>>) {
        pairs.append(&mut self.pairs);
        self.pairs = pairs;
    }

    pub(crate) fn parse_pair(&mut self, pair: Pair<'de>) -> Result<Option<(&'de [u8], &'de [u8])>> {
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
            Ok(Some((&pair.key[0..key_index], pair.value)))
        }
    }

    pub(crate) fn next_key(&mut self) -> Result<Option<&'de [u8]>> {
        while let Some(pair) = self.pairs.pop_back() {
            match self.parse_pair(pair)? {
                Some((key, value)) => {
                    self.value = Some(value);
                    return Ok(Some(key));
                }
                None => {
                    continue;
                }
            }
        }
        Ok(None)
    }

    pub(crate) fn next_value(&mut self) -> Result<&'de [u8]> {
        match self.value.take() {
            Some(value) => Ok(value),
            None => Err(Error::InvalidMapValue),
        }
    }

    // TODO: this is overcomplicated, look for an easier way
    pub(crate) fn into_pairs(mut self) -> Result<Vec<(&'de [u8], ItemKind<'de>)>> {
        let mut values = vec![];
        while let Some(pair) = self.pairs.pop_back() {
            if let Some((key, value)) = self.parse_pair(pair)? {
                // If it is in current level just add it as a single value
                values.push((key, ItemKind::Value(value)));
            } else if let Some(key) = self.stash.next_key()? {
                // Visit the stash
                if key.is_empty() {
                    // If key is empty, then add it as a single map
                    values.push((
                        key,
                        ItemKind::Map(PairMap::new(
                            self.stash.next_value()?,
                            self.remaining_depth - 1,
                        )),
                    ));
                } else if let Some((_, item)) = values.iter_mut().find(|item| item.0 == key) {
                    // If we already saw the key and it was a map, combine it with the previous map
                    // If it was a single value, just ignore this one
                    if let ItemKind::Map(map) = item {
                        map.prepend(self.stash.next_value()?)
                    }
                } else {
                    values.push((
                        key,
                        ItemKind::Map(PairMap::new(
                            self.stash.next_value()?,
                            self.remaining_depth - 1,
                        )),
                    ));
                }
            }
        }
        Ok(values)
    }

    pub(crate) fn into_seq(self) -> Result<PairSeq<'de>> {
        let pairs = self.into_pairs()?;
        let mut items = vec![];

        for (key, value) in pairs {
            if key.is_empty() {
                items.push((KeyType::None, value))
            } else {
                let index: Result<u16> =
                    serde::de::Deserialize::deserialize(&mut Value::new(&mut Parser::new(key)));
                if let Ok(index) = index {
                    items.push((KeyType::Number(index), value));
                } else {
                    items.push((KeyType::Name(key), value));
                }
            }
        }

        // Order the items by their keys
        items.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));
        items.reverse();
        items.dedup_by(|a, b| a.0 == b.0);

        let items = items.into_iter().map(|item| item.1).collect();

        Ok(PairSeq::new(items))
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
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }

        if let Some(key) = self.next_key()? {
            return seed
                .deserialize(&mut Value::new(&mut Parser::new(key)))
                .map(Some);
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
        match self.next_value() {
            Ok(value) => seed.deserialize(&mut Value::new(&mut Parser::new(value))),
            _ => {
                // Time to visit the stash
                seed.deserialize(PairMap::new(
                    self.stash.next_value()?,
                    self.remaining_depth - 1,
                ))
            }
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
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }

        // We throw all keys away, except the last one
        let mut last_key = None;
        while let Some(key) = self.next_key()? {
            last_key = Some(key);
        }

        let key = match last_key {
            Some(key) => key,
            None => {
                // Visit stash
                while let Some(key) = self.stash.next_key()? {
                    last_key = Some(key)
                }

                match last_key {
                    Some(key) => key,
                    None => {
                        return Err(Error::EofReached);
                    }
                }
            }
        };

        Ok((
            seed.deserialize(&mut Value::new(&mut Parser::new(key)))?,
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
            Ok(value) => serde::de::Deserializer::deserialize_seq(
                &mut Value::new(&mut Parser::new(value)),
                visitor,
            ),
            _ => {
                // Time to visit the stash
                visitor.visit_seq(
                    &mut PairMap::new(self.stash.next_value()?, self.remaining_depth - 1)
                        .into_seq()?,
                )
            }
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.next_value() {
            Ok(_) => Err(Error::InvalidMapValue),
            _ => visitor.visit_map(PairMap::new(
                self.stash.next_value()?,
                self.remaining_depth - 1,
            )),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.next_value() {
            Ok(value) => seed.deserialize(&mut Value::new(&mut Parser::new(value))),
            _ => seed.deserialize(PairMap::new(
                self.stash.next_value()?,
                self.remaining_depth - 1,
            )),
        }
    }
}
