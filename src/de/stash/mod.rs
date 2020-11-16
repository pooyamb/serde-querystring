use std::collections::VecDeque;

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
    pairs: VecDeque<(&'de [u8], VecDeque<Pair<'de>>)>,
    values: Option<VecDeque<Pair<'de>>>,
    pub(crate) remaining_depth: u16,
}

impl<'de> Stash<'de> {
    pub(crate) fn new(remaining_depth: u16) -> Self {
        // We also store remaining_depth here as we may go independent of main deserializer
        // from here
        Self {
            pairs: VecDeque::new(),
            values: None,
            remaining_depth,
        }
    }

    pub(crate) fn add(&mut self, parent: &'de [u8], key: &'de [u8], value: &'de [u8]) {
        if let Some((_, pairs)) = self.pairs.iter_mut().find(|item| item.0 == parent) {
            pairs.push_front(Pair { key, value });
        } else {
            let mut pairs = VecDeque::new();
            pairs.push_front(Pair { key, value });
            self.pairs.push_front((parent, pairs));
        }
    }

    pub(crate) fn next_key(&mut self) -> Result<Option<&'de [u8]>> {
        // Calling next_value before next_key is an error, so we don't check the depth there
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }

        if let Some((parent, pairs)) = self.pairs.pop_back() {
            self.values = Some(pairs);
            Ok(Some(parent))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn next_value(&mut self) -> Result<VecDeque<Pair<'de>>> {
        match self.values.take() {
            None => Err(Error::EofReached),
            Some(pairs) => Ok(pairs),
        }
    }

    pub(crate) fn next_value_map(&mut self) -> Result<PairMap<'de>> {
        match self.values.take() {
            None => Err(Error::EofReached),
            Some(pairs) => Ok(PairMap::new(self.remaining_depth - 1, pairs)),
        }
    }
}
