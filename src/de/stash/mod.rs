use std::collections::BTreeMap;

use crate::error::{Error, Result};

mod map;
mod seq;

use map::PairMap;

#[derive(Debug)]
pub(crate) struct Pair<'de> {
    pub(crate) key: &'de [u8],
    pub(crate) value: &'de [u8],
}

pub(crate) struct Stash<'de> {
    keys: BTreeMap<&'de [u8], Vec<Pair<'de>>>,
    values: Option<Vec<Pair<'de>>>,
    pub(crate) remaining_depth: u16,
}

impl<'de> Stash<'de> {
    pub(crate) fn new(remaining_depth: u16) -> Self {
        // We also store remaining_depth here as we may go independent of main deserializer
        // from here
        Self {
            keys: BTreeMap::default(),
            values: None,
            remaining_depth,
        }
    }

    pub(crate) fn add(&mut self, parent: &'de [u8], key: &'de [u8], value: &'de [u8]) {
        if let Some(pairs) = self.keys.get_mut(parent) {
            pairs.push(Pair { key, value });
        } else {
            self.keys.insert(parent, vec![Pair { key, value }]);
        }
    }

    pub(crate) fn next_key(&mut self) -> Result<Option<&'de [u8]>> {
        // Calling next_value before next_key is an error, so we don't check the depth there
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }

        // TODO: should look for a better alternative
        let parent: &[u8] = match self.keys.keys().take(1).collect::<Vec<&&[u8]>>().pop() {
            None => return Ok(None),
            Some(parent) => *parent,
        };

        match self.keys.remove_entry(parent) {
            Some((parent, pair)) => {
                self.values = Some(pair);
                Ok(Some(parent))
            }
            None => Ok(None),
        }
    }

    pub(crate) fn next_value_map(&mut self) -> Result<PairMap<'de>> {
        match self.values.take() {
            None => Err(Error::EofReached),
            Some(pairs) => Ok(PairMap::new(self.remaining_depth - 1, pairs)),
        }
    }
}
