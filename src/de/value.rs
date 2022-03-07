use std::str::{self, Utf8Error};

use lexical::FromLexical;
use serde::{
    de::{self, IntoDeserializer},
    forward_to_deserialize_any,
};

use crate::error::{Error, ErrorKind, Result};

#[inline]
pub(crate) fn parse_char(bytes: &[u8]) -> Option<u8> {
    let high = char::from(bytes[0]).to_digit(16)?;
    let low = char::from(bytes[1]).to_digit(16)?;
    Some(high as u8 * 0x10 + low as u8)
}

pub(crate) struct ValueDeserializer<'de, 'a> {
    slice: &'de [u8],
    scratch: &'a mut Vec<u8>,
    index: usize,
}

impl<'de, 'a> ValueDeserializer<'de, 'a> {
    pub(crate) fn new(slice: &'de [u8], scratch: &'a mut Vec<u8>) -> Self {
        Self {
            slice,
            scratch,
            index: 0,
        }
    }

    fn deserialize_signed<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_i64(self.parse_number()?)
    }

    fn deserialize_unsigned<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_u64(self.parse_number()?)
    }

    fn deserialize_float<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_f64(self.parse_number()?)
    }

    #[cold]
    fn invalid_encoding(&self, error: Utf8Error) -> Error {
        Error::new(ErrorKind::InvalidEncoding)
            .message("invalid utf-8 sequence found in the percent decoded value".to_string())
            .value(self.slice)
            .index(self.index_before_decoding(error.valid_up_to()))
    }

    #[cold]
    fn invalid_number(&self, error: lexical::Error) -> Error {
        // Lexical doesn't provide a method to see if it's a parsing error
        // It may be better to just use enum matching for the error, but it's harder
        let err_string = error.to_string();
        if err_string.starts_with("lexical parse error") {
            // The actual message is between the first two single quotations
            let the_message = err_string.split('\'');

            if let Some(the_message) = the_message.skip(1).next() {
                if let Some(index) = error.index() {
                    return Error::new(ErrorKind::InvalidNumber)
                        .message(the_message.to_owned())
                        .value(self.slice)
                        .index(self.index_before_decoding(*index));
                }
            }
        }

        // We shouldn't reach here unless lexical change their error message
        // Or some configs change, etc
        Error::new(ErrorKind::InvalidEncoding)
            .message(err_string)
            .value(self.slice)
    }

    #[cold]
    fn invalid_boolean(&self) -> Error {
        Error::new(ErrorKind::InvalidBoolean)
            .message(String::from("invalid ident found for boolean"))
            .value(self.slice)
            .index(0)
    }

    #[cold]
    fn index_before_decoding(&self, mut index: usize) -> usize {
        let mut cursor = 0;

        while cursor < index {
            if self.slice[cursor] == b'%'
                && self.slice.len() > cursor + 2
                && parse_char(&self.slice[(cursor + 1)..(cursor + 2)]).is_some()
            {
                index += 3;
            }
            cursor += 1;
        }

        index
    }
}

/// Parsing methods
impl<'de, 'a> ValueDeserializer<'de, 'a> {
    pub(crate) fn parse_str_bytes<'s, T, F>(
        &'s mut self,
        result: F,
    ) -> Result<Reference<'de, 's, T>>
    where
        T: ?Sized + 's,
        F: for<'f> FnOnce(&Self, &'f [u8]) -> Result<&'f T>,
    {
        self.scratch.clear();

        // Index of the first byte not yet copied into the scratch space.
        let mut index = 0;

        while let Some(v) = self.slice.get(index) {
            match v {
                b'+' => {
                    self.scratch
                        .extend_from_slice(&self.slice[self.index..index]);
                    self.scratch.push(b' ');

                    index += 1;
                    self.index = index;
                }
                b'%' => {
                    // we saw percentage
                    if self.slice.len() > index + 2 {
                        match parse_char(&self.slice[(index + 1)..=(index + 2)]) {
                            Some(b) => {
                                self.scratch
                                    .extend_from_slice(&self.slice[self.index..index]);
                                self.scratch.push(b);

                                index += 3;
                                self.index = index;
                            }
                            None => {
                                // If it wasn't valid, go to the next byte
                                index += 1;
                            }
                        }
                    } else {
                        index += 1;
                    }
                }
                _ => {
                    index += 1;
                }
            }
        }

        if self.scratch.is_empty() {
            result(self, self.slice).map(Reference::Borrowed)
        } else {
            self.scratch.extend_from_slice(&self.slice[self.index..]);
            result(self, self.scratch).map(Reference::Copied)
        }
    }

    pub(crate) fn parse_str<'s>(&'s mut self) -> Result<Reference<'de, 's, str>> {
        self.parse_str_bytes(|other_self, bytes| {
            str::from_utf8(bytes).map_err(|e| other_self.invalid_encoding(e))
        })
    }

    pub(crate) fn parse_bytes<'s>(&'s mut self) -> Result<Reference<'de, 's, [u8]>> {
        self.parse_str_bytes(|_, bytes| Ok(bytes))
    }

    pub(crate) fn parse_number<'s, T>(&'s mut self) -> Result<T>
    where
        T: FromLexical,
    {
        let c = match self.parse_bytes()? {
            Reference::Borrowed(b) => b,
            Reference::Copied(c) => c,
        };
        lexical::parse(c).map_err(|e| self.invalid_number(e))
    }

    pub(crate) fn parse_bool(&mut self) -> Result<bool> {
        match self.slice.len() {
            0 => Ok(true),
            1 => match self.slice[0] {
                b'1' => Ok(true),
                b'0' => Ok(false),
                _ => Err(self.invalid_boolean()),
            },
            2 if self.slice == b"on" => Ok(true),
            3 if self.slice == b"off" => Ok(false),
            4 if self.slice == b"true" => Ok(true),
            5 if self.slice == b"false" => Ok(false),
            _ => Err(self.invalid_boolean()),
        }
    }
}

macro_rules! deserialize_in {
    ($other_method:ident, $($method:ident) *) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value>
            where
                V: de::Visitor<'de>,
            {
                self.$other_method(visitor)
            }
        )*
    };
}

impl<'de, 'a> de::Deserializer<'de> for ValueDeserializer<'de, 'a> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_str()? {
            Reference::Borrowed(b) => visitor.visit_borrowed_str(b),
            Reference::Copied(c) => visitor.visit_str(c),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _: &str, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_bool<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
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
        visitor.visit_enum(self)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.slice.len() > 0 {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    /// We don't check the bytes to be valid utf8
    #[inline]
    fn deserialize_bytes<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_bytes()? {
            Reference::Borrowed(b) => visitor.visit_borrowed_bytes(b),
            Reference::Copied(c) => visitor.visit_bytes(c),
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string unit unit_struct map struct
        identifier tuple seq tuple_struct
    }

    deserialize_in!(
        deserialize_signed,
        deserialize_i8 deserialize_i16 deserialize_i32 deserialize_i64
    );

    deserialize_in!(
        deserialize_unsigned,
        deserialize_u8 deserialize_u16 deserialize_u32 deserialize_u64
    );

    deserialize_in!(
        deserialize_float,
        deserialize_f32 deserialize_f64
    );
}

impl<'de, 'a> de::EnumAccess<'de> for ValueDeserializer<'de, 'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(mut self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.parse_str()?.into_deserializer())
            .map(|res| (res, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for ValueDeserializer<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    #[cold]
    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::new(ErrorKind::UnexpectedType)
            .message(String::from("Tuple enums are not supported")))
    }

    #[cold]
    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::new(ErrorKind::UnexpectedType)
            .message(String::from("Struct enums are not supported")))
    }

    #[cold]
    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::new(ErrorKind::UnexpectedType)
            .message(String::from("Newtype enums are not supported")))
    }
}

pub(crate) enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T> std::ops::Deref for Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Reference::Borrowed(b) => b,
            Reference::Copied(c) => c,
        }
    }
}
