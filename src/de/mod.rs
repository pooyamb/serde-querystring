use std::str;

use serde::{de, forward_to_deserialize_any};

use crate::error::{Error, Result};

mod basis;
mod constants;
mod map;
mod parser;
mod stash;

use basis::Reference;
use map::MapEntry;

pub(crate) struct Deserializer<'de> {
    parser: parser::Parser<'de>,
    scratch: Vec<u8>,
    remaining_depth: u16,
}

impl<'de> Deserializer<'de> {
    pub fn new(slice: &'de [u8]) -> Self {
        Self {
            parser: parser::Parser::new(slice),
            scratch: Vec::new(),
            // We don't actually support these many levels of recursion, the actual level
            // should be around 60. But it's here just as a limit to prevent stack overflow
            remaining_depth: 64,
        }
    }

    pub fn new_with_depth(slice: &'de [u8], remaining_depth: u16) -> Self {
        Self {
            parser: parser::Parser::new(slice),
            scratch: Vec::new(),
            remaining_depth,
        }
    }

    fn deserialize_number<V>(&mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.parser.peek()? {
            Some(b) => b,
            None => {
                return Err(Error::EofReached);
            }
        };

        let res = match peek {
            b'-' => {
                self.parser.discard();
                self.parser.parse_integer(false)?.visit(visitor)
            }
            b'0'..=b'9' => self.parser.parse_integer(true)?.visit(visitor),
            _ => {
                return Err(Error::InvalidNumber);
            }
        };

        match self.parser.peek()? {
            None | Some(b'&') | Some(b';') => {
                self.parser.discard();
                res
            }
            Some(_) => Err(Error::InvalidNumber),
        }
    }
}

macro_rules! deserialize_number {
    ($method:ident) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value>
        where
            V: de::Visitor<'de>,
        {
            self.deserialize_number(visitor)
        }
    };
}

impl<'de> de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let peek = match self.parser.peek()? {
            Some(b) => b,
            None => return visitor.visit_unit(),
        };

        let value = match peek {
            b'=' => Err(Error::InvalidMapKey),
            _ => match self.parser.parse_str(&mut self.scratch)? {
                Reference::Borrowed(s) => visitor.visit_borrowed_str(s),
                Reference::Copied(s) => visitor.visit_str(s),
            },
        };
        self.parser.discard();
        value
    }

    #[inline]
    fn deserialize_struct<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }
        visitor.visit_map(MapEntry::new(self))
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _: &str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }
        visitor.visit_map(&mut MapEntry::new(self))
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.parser.parse_bool()?)
    }

    #[inline]
    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }
        self.deserialize_tuple(len, visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.remaining_depth == 0 {
            return Err(Error::MaximumDepthReached);
        }
        visitor.visit_enum(MapEntry::new(self))
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parser.peek()? {
            Some(b'&') | Some(b';') | None => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.parser.peek()? == Some(b'&') {
            self.parser.discard();
        }

        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string bytes byte_buf unit_struct
        identifier ignored_any
    }

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);

    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);
}

impl<'de> de::SeqAccess<'de> for Deserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        let mut index = self.parser.index;
        let mut saw_delimiter = false;

        while index < self.parser.slice.len() {
            match self.parser.slice[index] {
                b',' | b'&' | b';' => {
                    saw_delimiter = true;
                    break;
                }
                _ => {
                    index += 1;
                }
            }
        }

        if !saw_delimiter && index == self.parser.index {
            return Ok(None);
        }

        let slice = &self.parser.slice[self.parser.index..index];

        // TODO: it may make sense to do boundary check here
        self.parser.index = index + 1;

        let remaining_depth = if self.remaining_depth > 1 {
            self.remaining_depth - 1
        } else {
            0
        };

        seed.deserialize(&mut Deserializer::new_with_depth(
            slice,
            self.remaining_depth - 1,
        ))
        .map(Some)
    }
}
