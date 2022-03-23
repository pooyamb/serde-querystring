use std::borrow::Cow;
use std::str;

use lexical::{self, FromLexical};
use serde::{de, forward_to_deserialize_any};

use crate::decode::{parse_bytes, Reference};

use super::{
    error::Error,
    slices::{OptionalRawSlice, ParsedSlice, RawSlice},
};

pub trait IntoDeserializer<'de, 's> {
    /// The type of the deserializer being converted into.
    type Deserializer: de::Deserializer<'de, Error = Error>;

    /// Convert this value into a deserializer.
    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer;
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'de, 's> IntoDeserializer<'de, 's> for ParsedSlice<'de> {
    type Deserializer = ParsedSliceDeserializer<'de>;

    fn into_deserializer(self, _: &mut Vec<u8>) -> Self::Deserializer {
        ParsedSliceDeserializer(self.0)
    }
}

pub struct ParsedSliceDeserializer<'de>(Cow<'de, [u8]>);

impl<'de> ParsedSliceDeserializer<'de> {
    fn parse_number<T>(&self) -> Result<T, Error>
    where
        T: FromLexical,
    {
        lexical::parse(&self.0).map_err(|e| Error::Custom(e.to_string()))
    }

    fn parse_bool(&self) -> Result<bool, Error> {
        match self.0.len() {
            0 => Ok(true),
            1 => match self.0[0] {
                b'1' => Ok(true),
                b'0' => Ok(false),
                _ => Err(Error::Custom("(invalid bool)".to_string())),
            },
            2 if self.0.as_ref() == b"on" => Ok(true),
            3 if self.0.as_ref() == b"off" => Ok(false),
            4 if self.0.as_ref() == b"true" => Ok(true),
            5 if self.0.as_ref() == b"false" => Ok(false),
            _ => Err(Error::Custom("(invalid bool)".to_string())),
        }
    }

    fn parse_str(self) -> Result<Cow<'de, str>, Error> {
        let res = match self.0 {
            Cow::Borrowed(b) => str::from_utf8(b)
                .map(Cow::Borrowed)
                .map_err(|e| (e, Cow::Borrowed(b))),
            Cow::Owned(o) => String::from_utf8(o)
                .map(Cow::Owned)
                .map_err(|e| (e.utf8_error(), Cow::Owned(e.into_bytes()))),
        };

        res.map_err(|(error, _slice)| {
            // Error::new(ErrorKind::InvalidEncoding)
            //     .message("invalid utf-8 sequence found in the percent decoded value".to_string())
            //     .value(&slice)
            //     .index(error.valid_up_to())
            Error::Custom(error.to_string())
        })
    }
}

macro_rules! deserialize_number {
    ($($method:ident => $visit:ident) *) => {
        $(
            #[inline]
            fn $method<V>(self, visitor: V) -> Result<V::Value,Error>
            where
                V: de::Visitor<'de>,
            {
                visitor.$visit(self.parse_number()?)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for ParsedSliceDeserializer<'de> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_str()? {
            Cow::Borrowed(b) => visitor.visit_borrowed_str(b),
            Cow::Owned(o) => visitor.visit_string(o),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
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
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    /// We don't check the bytes to be valid utf8
    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0 {
            Cow::Borrowed(b) => visitor.visit_borrowed_bytes(b),
            Cow::Owned(o) => visitor.visit_byte_buf(o),
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string unit unit_struct map struct
        tuple seq tuple_struct
    }

    deserialize_number!(
        deserialize_i8 => visit_i8
        deserialize_i16 => visit_i16
        deserialize_i32 => visit_i32
        deserialize_i64 => visit_i64

        deserialize_u8 => visit_u8
        deserialize_u16 => visit_u16
        deserialize_u32 => visit_u32
        deserialize_u64 => visit_u64

        deserialize_f32 => visit_f32
        deserialize_f64 => visit_f64
    );
}

impl<'de> de::EnumAccess<'de> for ParsedSliceDeserializer<'de> {
    type Error = Error;
    type Variant = UnitOnly;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(|res| (res, UnitOnly))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'de, 's> IntoDeserializer<'de, 's> for RawSlice<'de> {
    type Deserializer = SliceDeserializer<'de, 's>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        SliceDeserializer(self.0, scratch)
    }
}

pub struct SliceDeserializer<'de, 's>(&'de [u8], &'s mut Vec<u8>);

impl<'de, 's> SliceDeserializer<'de, 's> {
    fn parse_number<T>(&self) -> Result<T, Error>
    where
        T: FromLexical,
    {
        lexical::parse(self.0).map_err(|e| Error::Custom(e.to_string()))
    }

    fn parse_bool(&self) -> Result<bool, Error> {
        match self.0.len() {
            0 => Ok(true),
            1 => match self.0[0] {
                b'1' => Ok(true),
                b'0' => Ok(false),
                _ => Err(Error::Custom("(invalid bool)".to_string())),
            },
            2 if self.0 == b"on" => Ok(true),
            3 if self.0 == b"off" => Ok(false),
            4 if self.0 == b"true" => Ok(true),
            5 if self.0 == b"false" => Ok(false),
            _ => Err(Error::Custom("(invalid bool)".to_string())),
        }
    }

    fn parse_str(self) -> Result<Reference<'de, 's, str>, Error> {
        let slice = self.0;

        parse_bytes(slice, self.1)
            .try_map(str::from_utf8)
            .map_err(|error| {
                // Error::new(ErrorKind::InvalidEncoding)
                //     .message(
                //         "invalid utf-8 sequence found in the percent decoded value".to_string(),
                //     )
                //     .value(slice)
                //     .index(error.valid_up_to())

                Error::Custom(error.to_string())
            })
    }

    fn parse_bytes(self) -> Reference<'de, 's, [u8]> {
        parse_bytes(self.0, self.1)
    }
}

macro_rules! deserialize_number {
    ($($method:ident => $visit:ident) *) => {
        $(
            #[inline]
            fn $method<V>(self, visitor: V) -> Result<V::Value,Error>
            where
                V: de::Visitor<'de>,
            {
                visitor.$visit(self.parse_number()?)
            }
        )*
    };
}

impl<'de, 's> de::Deserializer<'de> for SliceDeserializer<'de, 's> {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_str()? {
            Reference::Borrowed(b) => visitor.visit_borrowed_str(b),
            Reference::Copied(o) => visitor.visit_str(o),
        }
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
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
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        if self.0.is_empty() {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    /// We don't check the bytes to be valid utf8
    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.parse_bytes() {
            Reference::Borrowed(b) => visitor.visit_borrowed_bytes(b),
            Reference::Copied(c) => visitor.visit_bytes(c),
        }
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string unit unit_struct map struct
        tuple seq tuple_struct
    }

    deserialize_number!(
        deserialize_i8 => visit_i8
        deserialize_i16 => visit_i16
        deserialize_i32 => visit_i32
        deserialize_i64 => visit_i64

        deserialize_u8 => visit_u8
        deserialize_u16 => visit_u16
        deserialize_u32 => visit_u32
        deserialize_u64 => visit_u64

        deserialize_f32 => visit_f32
        deserialize_f64 => visit_f64
    );
}

impl<'de, 's> de::EnumAccess<'de> for SliceDeserializer<'de, 's> {
    type Error = Error;
    type Variant = UnitOnly;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self).map(|res| (res, UnitOnly))
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'de, 's> IntoDeserializer<'de, 's> for OptionalRawSlice<'de> {
    type Deserializer = SliceDeserializer<'de, 's>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        match self.0 {
            Some(b) => SliceDeserializer(b, scratch),
            None => SliceDeserializer(b"", scratch),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'de, 's, I> IntoDeserializer<'de, 's> for I
where
    I: 'de + Iterator<Item = RawSlice<'de>>,
{
    type Deserializer = IterDeserializer<'s, I>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        IterDeserializer(self, scratch)
    }
}

pub struct IterDeserializer<'s, I>(I, &'s mut Vec<u8>);

impl<'de, 's, I> IterDeserializer<'s, I>
where
    I: Iterator<Item = RawSlice<'de>>,
{
    fn parse_number<T>(self) -> Result<T, Error>
    where
        T: FromLexical,
    {
        SliceDeserializer(
            self.0
                .last()
                .expect("Values iterator has no value inside it")
                .0,
            self.1,
        )
        .parse_number()
    }
}

impl<'de, 's, I> de::Deserializer<'de> for IterDeserializer<'s, I>
where
    I: 'de + Iterator<Item = RawSlice<'de>>,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        let v = self
            .0
            .last()
            .expect("Values iterator has no value inside it");
        SliceDeserializer(v.0, self.1).deserialize_any(visitor)
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _: &str, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    #[inline]
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        SliceDeserializer(
            self.0
                .last()
                .expect("Values iterator has no value inside it")
                .0,
            self.1,
        )
        .deserialize_bool(visitor)
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        SliceDeserializer(
            self.0
                .last()
                .expect("Values iterator has no value inside it")
                .0,
            self.1,
        )
        .deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        SliceDeserializer(
            self.0
                .last()
                .expect("Values iterator has no value inside it")
                .0,
            self.1,
        )
        .deserialize_option(visitor)
    }

    #[inline]
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_unit()
    }

    /// We don't check the bytes to be valid utf8
    #[inline]
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        SliceDeserializer(
            self.0
                .last()
                .expect("Values iterator has no value inside it")
                .0,
            self.1,
        )
        .deserialize_bytes(visitor)
    }

    #[inline]
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if len == self.0.size_hint().0 {
            visitor.visit_seq(self)
        } else {
            Err(Error::Custom("Seq length is wrong".to_string()))
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        if len == self.0.size_hint().0 {
            visitor.visit_seq(self)
        } else {
            Err(Error::Custom("Seq length is wrong".to_string()))
        }
    }

    forward_to_deserialize_any! {
        <W: Visitor<'de>>
        char str string unit unit_struct map struct identifier
    }

    deserialize_number!(
        deserialize_i8 => visit_i8
        deserialize_i16 => visit_i16
        deserialize_i32 => visit_i32
        deserialize_i64 => visit_i64

        deserialize_u8 => visit_u8
        deserialize_u16 => visit_u16
        deserialize_u32 => visit_u32
        deserialize_u64 => visit_u64

        deserialize_f32 => visit_f32
        deserialize_f64 => visit_f64
    );
}

impl<'de, 's, I> de::SeqAccess<'de> for IterDeserializer<'s, I>
where
    I: 'de + Iterator<Item = RawSlice<'de>>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.0
            .next()
            .map(|v| seed.deserialize(v.into_deserializer(self.1)))
            .transpose()
    }
}

pub struct UnitOnly;

impl<'de> de::VariantAccess<'de> for UnitOnly {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    #[cold]
    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Custom(String::from("Tuple enums are not supported")))
    }

    #[cold]
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        Err(Error::Custom(String::from("Tuple enums are not supported")))
    }

    #[cold]
    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::Custom(String::from("Tuple enums are not supported")))
    }
}
