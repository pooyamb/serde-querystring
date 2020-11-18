use serde::de;

use super::map::PairMap;
use super::Value;
use crate::error::{Error, Result};

pub(crate) enum ItemKind<'de> {
    Value(&'de [u8]),
    Map(PairMap<'de>),
}

pub(crate) struct PairSeq<'de> {
    items: Vec<ItemKind<'de>>,
}

impl<'de> PairSeq<'de> {
    pub(crate) fn new(items: Vec<ItemKind<'de>>) -> Self {
        Self { items }
    }
}

impl<'de> de::SeqAccess<'de> for PairSeq<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        match self.items.pop() {
            Some(ItemKind::Value(value)) => seed.deserialize(&mut Value::new(value)).map(Some),
            Some(ItemKind::Map(map)) => seed.deserialize(map).map(Some),
            None => Ok(None),
        }
    }
}
