use std::collections::VecDeque;

use serde::de;

use super::map::PairMap;
use crate::de::Deserializer;
use crate::error::{Error, Result};

pub(crate) struct PairSeq<'de> {
    values: VecDeque<&'de [u8]>,
    pairs: VecDeque<PairMap<'de>>,
    remaining_depth: u16,
}

impl<'de> PairSeq<'de> {
    pub(crate) fn new(
        values: VecDeque<&'de [u8]>,
        pairs: VecDeque<PairMap<'de>>,
        remaining_depth: u16,
    ) -> Self {
        Self {
            values,
            pairs,
            remaining_depth,
        }
    }
}

impl<'de> de::SeqAccess<'de> for PairSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.values.pop_back() {
            Some(value) => seed
                .deserialize(&mut Deserializer::new_with_depth(
                    value,
                    self.remaining_depth,
                ))
                .map(Some),
            None => match self.pairs.pop_back() {
                Some(pair) => seed.deserialize(pair).map(Some),
                None => Ok(None),
            },
        }
    }
}
