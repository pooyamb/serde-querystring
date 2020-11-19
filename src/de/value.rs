use serde::de::IntoDeserializer;
use serde::{de, forward_to_deserialize_any};

use super::parser::{Parser, Reference};
use super::{Error, Result};

pub(crate) struct Value<'de, 'a> {
    parser: &'a mut Parser<'de>,
    scratch: Vec<u8>,
    // Only used for vectors, as we don't support vectors of vectors
    flat: bool,
}

impl<'de, 'a> Value<'de, 'a> {
    pub(crate) fn new(parser: &'a mut Parser<'de>) -> Self {
        Self {
            parser,
            scratch: Vec::new(),
            flat: false,
        }
    }

    pub(crate) fn new_flat(parser: &'a mut Parser<'de>) -> Self {
        Self {
            parser,
            scratch: Vec::new(),
            flat: true,
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

impl<'de, 'a> de::Deserializer<'de> for &mut Value<'de, 'a> {
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
        value
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
        if self.flat {
            return Err(Error::NotSupportedAsValue);
        }
        visitor.visit_seq(SeqAccess::new(self))
    }

    #[inline]
    fn deserialize_tuple<V>(self, _: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        if self.flat {
            return Err(Error::NotSupportedAsValue);
        }
        visitor.visit_seq(SeqAccess::new(self))
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
        visitor.visit_enum(self)
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
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string bytes byte_buf unit_struct map struct
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

impl<'de, 'a> de::EnumAccess<'de> for &mut Value<'de, 'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(
            self.parser
                .parse_str(&mut self.scratch)?
                .into_deserializer(),
        )
        .map(|res| (res, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for &mut Value<'de, 'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotSupportedAsValue)
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], _visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::NotSupportedAsValue)
    }

    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::NotSupportedAsValue)
    }
}

pub(crate) struct SeqAccess<'de, 'a, 'b> {
    value: &'b mut Value<'de, 'a>,
    end: bool,
}

impl<'de, 'a, 'b> SeqAccess<'de, 'a, 'b> {
    pub(crate) fn new(value: &'b mut Value<'de, 'a>) -> Self {
        Self { value, end: false }
    }
}

impl<'de, 'a, 'b> de::SeqAccess<'de> for SeqAccess<'de, 'a, 'b> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.end {
            return Ok(None);
        }
        let res = match self.value.parser.parse_one_seq_value()? {
            Some(slice) => seed
                .deserialize(&mut Value::new_flat(&mut Parser::new(slice)))
                .map(Some),
            None => Ok(None),
        };
        match self.value.parser.peek()? {
            None | Some(b'&') | Some(b';') => {
                self.value.parser.discard();
                self.end = true
            }
            Some(b',') => self.value.parser.discard(),
            _ => {}
        };
        res
    }
}
