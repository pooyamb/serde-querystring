use std::collections::VecDeque;

use serde::{de, forward_to_deserialize_any};

use crate::Error;

use super::{
    pair::{Pair, PairVec},
    value::ValueDeserializer,
};

#[derive(Default)]
pub(crate) struct Stash<'de> {
    pairs: Vec<(&'de [u8], PairVec<'de>)>,

    // Used to avoid allocation while percent decoding
    scratch: Vec<u8>,

    // Used to hold the next value temporary while deserializing as a map
    temp_values: Option<PairVec<'de>>,
}

impl<'de> Stash<'de> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, mut pair: Pair<'de>) {
        let key = pair.next_key().unwrap();

        if let Some((_, pairs)) = self.pairs.iter_mut().find(|item| item.0 == key) {
            pairs.push_front(pair)
        } else {
            let mut pairs = VecDeque::new();
            pairs.push_front(pair);
            self.pairs.push((key, PairVec::new(pairs)));
        }
    }
}

impl<'de> de::Deserializer<'de> for Stash<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::MapAccess<'de> for Stash<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        if let Some((key, values)) = self.pairs.pop() {
            self.temp_values = Some(values);

            // Deserialize the key part directly
            seed.deserialize(ValueDeserializer::new(key, &mut self.scratch))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        debug_assert!(self.temp_values.is_some());

        if let Some(values) = self.temp_values.take() {
            seed.deserialize(values.into_deserializer(&mut self.scratch))
        } else {
            // It should be unreachable as serde guanartees it by calling next_key_seed first
            unreachable!()
        }
    }

    /// Returns the number of entries remaining in the map, if known.
    #[inline]
    fn size_hint(&self) -> Option<usize> {
        Some(self.pairs.len())
    }
}
