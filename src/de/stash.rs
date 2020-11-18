use std::collections::VecDeque;

use crate::error::{Error, Result};

#[derive(Debug)]
pub(crate) struct Pair<'de> {
    pub(crate) key: &'de [u8],
    pub(crate) value: &'de [u8],
}

pub(crate) struct Stash<'de> {
    pairs: VecDeque<(&'de [u8], VecDeque<Pair<'de>>)>,
    values: Option<VecDeque<Pair<'de>>>,
}

impl<'de> Stash<'de> {
    pub(crate) fn new() -> Self {
        Self {
            pairs: VecDeque::new(),
            values: None,
        }
    }

    pub(crate) fn add(&mut self, key: &'de [u8], subkey: &'de [u8], value: &'de [u8]) {
        if let Some((_, pairs)) = self.pairs.iter_mut().find(|item| item.0 == key) {
            pairs.push_front(Pair { key: subkey, value });
        } else {
            let mut pairs = VecDeque::new();
            pairs.push_front(Pair { key: subkey, value });
            self.pairs.push_front((key, pairs));
        }
    }

    pub(crate) fn next_key(&mut self) -> Result<Option<&'de [u8]>> {
        if let Some((key, pairs)) = self.pairs.pop_back() {
            self.values = Some(pairs);
            Ok(Some(key))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn next_value(&mut self) -> Result<VecDeque<Pair<'de>>> {
        // Calling next_value before next_key is an error
        match self.values.take() {
            None => Err(Error::EofReached),
            Some(pairs) => Ok(pairs),
        }
    }
}
