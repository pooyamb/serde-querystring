use std::{collections::VecDeque, ops};

use serde::de;

use super::value::ValueDeserializer;
use crate::{
    error::{Error, Result},
    ErrorKind,
};

struct Key<'de> {
    slice: &'de [u8],
    index: usize,
}

impl<'de> Key<'de> {
    pub fn new(slice: &'de [u8]) -> Self {
        Self { slice, index: 0 }
    }

    fn next(&mut self) -> Option<&'de [u8]> {
        if self.index == 0 {
            self.index = self.slice.len();
            Some(self.slice)
        } else {
            None
        }
    }
}

pub(crate) struct Value<'de> {
    slice: Option<&'de [u8]>,
}

impl<'de> Value<'de> {
    pub fn new(slice: Option<&'de [u8]>) -> Self {
        Self { slice }
    }

    pub fn to_deserializer<'a>(&self, scrach: &'a mut Vec<u8>) -> ValueDeserializer<'de, 'a> {
        ValueDeserializer::new(&self.slice.unwrap_or_default(), scrach)
    }
}

pub(crate) struct Pair<'de> {
    key: Key<'de>,
    value: Value<'de>,
}

impl<'de> Pair<'de> {
    pub fn new(key: &'de [u8], value: Option<&'de [u8]>) -> Self {
        Self {
            key: Key::new(key),
            value: Value::new(value),
        }
    }

    pub fn next_key(&mut self) -> Option<&'de [u8]> {
        self.key.next()
    }

    pub fn value_ref(&self) -> &Value<'de> {
        &self.value
    }
}

pub(crate) struct PairVec<'de> {
    values: VecDeque<Pair<'de>>,
}

impl<'de> PairVec<'de> {
    pub(crate) fn new(values: VecDeque<Pair<'de>>) -> Self {
        Self { values }
    }

    /// Only used when we only want one single value out of the vector,
    /// it's valid because of how we push at least one item to the vector
    fn force_pop(&mut self) -> Pair<'de> {
        if let Some(pair) = self.values.pop_front() {
            pair
        } else {
            unreachable!()
        }
    }

    pub(crate) fn into_deserializer<'a>(
        self,
        scratch: &'a mut Vec<u8>,
    ) -> PairVecDeserializer<'de, 'a> {
        PairVecDeserializer { vec: self, scratch }
    }
}

impl<'de> ops::Deref for PairVec<'de> {
    type Target = VecDeque<Pair<'de>>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<'de> ops::DerefMut for PairVec<'de> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}

pub(crate) struct PairVecDeserializer<'de, 'a> {
    vec: PairVec<'de>,
    scratch: &'a mut Vec<u8>,
}

impl<'de, 'a> PairVecDeserializer<'de, 'a> {
    #[cold]
    fn set_error_key(&mut self, err: Error, last_item: &Pair<'de>) -> Error {
        err.key(last_item.key.slice)
    }

    #[cold]
    fn invalid_length_error(&mut self, len: usize) -> Error {
        Error::new(ErrorKind::InvalidLength)
            .message(format!(
                "invalid length {}, expected a sequence of size {}",
                self.vec.len(),
                len
            ))
            .key(self.vec.force_pop().key.slice)
    }
}

macro_rules! forward_to_deserialize_single {
    ($($func:ident)*) => {
        $(
            #[inline]
            fn $func<V>(mut self, visitor: V) -> Result<V::Value>
            where
                V: de::Visitor<'de>,
            {
                let last_item = self.vec.force_pop();
                last_item
                    .value_ref()
                    .to_deserializer(&mut self.scratch)
                    .$func(visitor)
                    .map_err(|e| self.set_error_key(e, &last_item))
            }
        )*
    };
}

impl<'de, 'a> de::Deserializer<'de> for PairVecDeserializer<'de, 'a> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_item = self.vec.force_pop();
        last_item
            .value_ref()
            .to_deserializer(&mut self.scratch)
            .deserialize_option(visitor)
            .map_err(|e| self.set_error_key(e, &last_item))
    }

    #[inline]
    fn deserialize_option<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        let last_item = self.vec.force_pop();
        last_item
            .value_ref()
            .to_deserializer(&mut self.scratch)
            .deserialize_option(visitor)
            .map_err(|e| self.set_error_key(e, &last_item))
    }

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
        visitor.visit_seq(self)
    }

    #[inline]
    fn deserialize_tuple<V>(mut self, size: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.vec.len() == size {
            visitor.visit_seq(self)
        } else {
            Err(self.invalid_length_error(size))
        }
    }

    #[inline]
    fn deserialize_tuple_struct<V>(mut self, _: &str, size: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.vec.len() == size {
            visitor.visit_seq(self)
        } else {
            Err(self.invalid_length_error(size))
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
        let last_item = self.vec.force_pop();
        visitor
            .visit_enum(last_item.value_ref().to_deserializer(&mut self.scratch))
            .map_err(|e| self.set_error_key(e, &last_item))
    }

    #[inline]
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        // We can't do this in the single value level
        // because we can't tell if there is nothing provided at all, at that level

        visitor.visit_unit()
    }

    forward_to_deserialize_single! {
        deserialize_i8 deserialize_i16 deserialize_i32 deserialize_i64
        deserialize_i128 deserialize_u8 deserialize_u16 deserialize_u32
        deserialize_u64 deserialize_u128 deserialize_f32 deserialize_f64
        deserialize_char deserialize_str deserialize_string deserialize_bytes
        deserialize_byte_buf deserialize_bool
    }

    serde::forward_to_deserialize_any! {
        <W: Visitor<'de>>
        identifier unit_struct ignored_any map struct
    }
}

impl<'de, 'a> de::SeqAccess<'de> for PairVecDeserializer<'de, 'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if let Some(p) = self.vec.pop_back() {
            seed.deserialize(p.value_ref().to_deserializer(&mut self.scratch))
                .map(Some)
        } else {
            Ok(None)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.vec.len())
    }
}
