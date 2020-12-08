use std::collections::VecDeque;

use super::parser::parse_char;
use crate::error::{Error, Result};

#[derive(Debug)]
pub struct SubKey<'de> {
    index: usize,
    slice: &'de [u8],
}

impl<'de> SubKey<'de> {
    pub(crate) fn new(slice: &'de [u8]) -> Self {
        Self { index: 0, slice }
    }

    pub(crate) fn next_key(&mut self) -> Result<&'de [u8]> {
        while self.index < self.slice.len() {
            match self.slice[self.index] {
                b']' => {
                    let key = &self.slice[0..self.index];
                    self.slice = &self.slice[(self.index + 1)..];
                    self.index = 0;
                    return Ok(&key);
                }
                b'%' => {
                    if self.index + 2 < self.slice.len() {
                        match parse_char(&self.slice[(self.index + 1)..=(self.index + 2)]) {
                            Some(b']') => {
                                let key = &self.slice[0..self.index];
                                self.slice = &self.slice[(self.index + 3)..];
                                self.index = 0;
                                return Ok(&key);
                            }
                            _ => {
                                self.index += 1;
                            }
                        }
                    } else {
                        self.index += 1;
                    }
                }
                _ => {
                    self.index += 1;
                }
            }
        }
        Err(Error::InvalidMapKey)
    }

    pub(crate) fn next_subkey(mut self) -> Result<Option<Self>> {
        // Check if the starting character is either `[` or percent encoded version of it
        if self.slice.len() > self.index {
            if self.slice.len() > self.index + 1 && self.slice[self.index] == b'[' {
                self.slice = &self.slice[1..];
                Ok(Some(self))
            } else if self.slice.len() > self.index + 2
                && self.slice[self.index] == b'%'
                && parse_char(&self.slice[(self.index + 1)..=(self.index + 2)]) == Some(b'[')
            {
                self.slice = &self.slice[3..];
                Ok(Some(self))
            } else {
                // The subkey part didn't start with opening bracket
                Err(Error::InvalidMapKey)
            }
        } else {
            Ok(None)
        }
    }
}
#[derive(Debug)]
pub(crate) struct Pair<'de> {
    pub(crate) key: SubKey<'de>,
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
            pairs.push_front(Pair {
                key: SubKey::new(subkey),
                value,
            });
        } else {
            let mut pairs = VecDeque::new();
            pairs.push_front(Pair {
                key: SubKey::new(subkey),
                value,
            });
            self.pairs.push_front((key, pairs));
        }
    }

    pub(crate) fn add_subkey(&mut self, key: &'de [u8], subkey: SubKey<'de>, value: &'de [u8]) {
        if let Some((_, pairs)) = self.pairs.iter_mut().find(|item| item.0 == key) {
            pairs.push_front(Pair { key: subkey, value });
        } else {
            let mut pairs = VecDeque::new();
            pairs.push_front(Pair { key: subkey, value });
            self.pairs.push_front((key, pairs));
        }
    }

    pub(crate) fn next_key(&mut self) -> Option<&'de [u8]> {
        if let Some((key, pairs)) = self.pairs.pop_back() {
            self.values = Some(pairs);
            Some(key)
        } else {
            None
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
