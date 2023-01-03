use std::str;

use _serde::{de, forward_to_deserialize_any};
use lexical::{self, FromLexical};

use crate::decode::Reference;

use super::{
    error::{Error, ErrorKind},
    slices::{DecodedSlice, RawSlice, Value},
};

pub trait IntoDeserializer<'de, 's> {
    /// The type of the deserializer being converted into.
    type Deserializer: de::Deserializer<'de, Error = Error>;

    /// Convert this value into a deserializer.
    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer;
}

///////////////////////////////////////////////////////////////////////////////////////////////////

impl<'de, 's> IntoDeserializer<'de, 's> for DecodedSlice<'de> {
    type Deserializer = ValueDeserializer<'s, Self>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        ValueDeserializer(self, scratch)
    }
}

impl<'de, 's> IntoDeserializer<'de, 's> for RawSlice<'de> {
    type Deserializer = ValueDeserializer<'s, Self>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        ValueDeserializer(self, scratch)
    }
}

impl<'de, 's> IntoDeserializer<'de, 's> for Option<RawSlice<'de>> {
    type Deserializer = ValueDeserializer<'s, Self>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        ValueDeserializer(self, scratch)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub struct ValueDeserializer<'s, T>(T, &'s mut Vec<u8>);

macro_rules! deserialize_number {
    ($($method:ident => $visit:ident) *) => {
        $(
            #[inline]
            fn $method<V>(self, visitor: V) -> Result<V::Value,Error>
            where
                V: de::Visitor<'de>,
            {
                visitor.$visit(self.0.parse_number(self.1)?)
            }
        )*
    };
}

impl<'de, 's, T> de::Deserializer<'de> for ValueDeserializer<'s, T>
where
    T: Value<'de>,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        match self.0.parse_str(self.1)? {
            Reference::Borrowed(b) => visitor.visit_borrowed_str(b),
            Reference::Copied(o) => visitor.visit_str(o),
            Reference::Owned(o) => visitor.visit_string(o),
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
        visitor.visit_bool(self.0.parse_bool(self.1)?)
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
        if self.0.is_none() {
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
        match self.0.parse_bytes(self.1) {
            Reference::Borrowed(b) => visitor.visit_borrowed_bytes(b),
            Reference::Copied(c) => visitor.visit_bytes(c),
            Reference::Owned(o) => visitor.visit_byte_buf(o),
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

impl<'de, 's, T> de::EnumAccess<'de> for ValueDeserializer<'s, T>
where
    T: Value<'de>,
{
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

pub trait IntoRawSlices<'de> {
    type SizedIterator: Iterator<Item = RawSlice<'de>>;
    type UnSizedIterator: Iterator<Item = RawSlice<'de>>;

    fn into_sized_iterator(self, size: usize) -> Result<Self::SizedIterator, Error>;
    fn into_unsized_iterator(self) -> Self::UnSizedIterator;
    fn into_single_slice(self) -> RawSlice<'de>;
}

impl<'de, 's, I> IntoDeserializer<'de, 's> for I
where
    I: 'de + IntoRawSlices<'de>,
{
    type Deserializer = IterDeserializer<'s, I>;

    fn into_deserializer(self, scratch: &'s mut Vec<u8>) -> Self::Deserializer {
        IterDeserializer(self, scratch)
    }
}

pub struct IterDeserializer<'s, I>(I, &'s mut Vec<u8>);

impl<'de, 's, I> IterDeserializer<'s, I>
where
    I: 'de + IntoRawSlices<'de>,
{
    fn parse_number<T>(self) -> Result<T, Error>
    where
        T: FromLexical,
    {
        self.0.into_single_slice().parse_number(self.1)
    }

    #[inline]
    fn into_slice_deserializer(self) -> ValueDeserializer<'s, RawSlice<'de>> {
        ValueDeserializer(self.0.into_single_slice(), self.1)
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

impl<'de, 's, I> de::Deserializer<'de> for IterDeserializer<'s, I>
where
    I: 'de + IntoRawSlices<'de>,
{
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        self.into_slice_deserializer().deserialize_any(visitor)
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
        self.into_slice_deserializer().deserialize_bool(visitor)
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
        self.into_slice_deserializer()
            .deserialize_enum(name, variants, visitor)
    }

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_some(self)
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
        self.into_slice_deserializer().deserialize_bytes(visitor)
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
        visitor.visit_seq(SizedIterDeserializer(
            self.0.into_unsized_iterator(),
            self.1,
        ))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SizedIterDeserializer(
            self.0.into_sized_iterator(len)?,
            self.1,
        ))
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
        visitor.visit_seq(SizedIterDeserializer(
            self.0.into_sized_iterator(len)?,
            self.1,
        ))
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

struct SizedIterDeserializer<'s, I>(I, &'s mut Vec<u8>);

impl<'de, 's, I> de::SeqAccess<'de> for SizedIterDeserializer<'s, I>
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
        Err(Error::new(ErrorKind::InvalidType)
            .message(String::from("Tuple enums are not supported")))
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
        Err(Error::new(ErrorKind::InvalidType)
            .message(String::from("Struct enums are not supported")))
    }

    #[cold]
    fn newtype_variant_seed<T>(self, _seed: T) -> Result<T::Value, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        Err(Error::new(ErrorKind::InvalidType)
            .message(String::from("NewType enums are not supported")))
    }
}
