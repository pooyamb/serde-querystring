use serde::de;

use super::map::PairMap;
// use super::Pair;
use crate::de::Deserializer;
use crate::error::{Error, Result};

pub(crate) struct PairSeq<'de> {
    values: Vec<&'de [u8]>,
    pairs: Vec<PairMap<'de>>,
}

impl<'de> PairSeq<'de> {
    pub(crate) fn new(values: Vec<&'de [u8]>, pairs: Vec<PairMap<'de>>) -> Self {
        Self { values, pairs }
    }
}

impl<'de> de::SeqAccess<'de> for PairSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.values.pop() {
            Some(value) => seed.deserialize(&mut Deserializer::new(value)).map(Some),
            None => match self.pairs.pop() {
                Some(pair) => seed.deserialize(pair).map(Some),
                None => Ok(None),
            },
        }
    }
}
