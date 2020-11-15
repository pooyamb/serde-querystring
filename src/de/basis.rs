use std::ops;

use serde::de;

use crate::error::Result;

pub(crate) enum ReaderNumber {
    F64(f64),
    U64(u64),
    I64(i64),
}

impl ReaderNumber {
    pub(crate) fn visit<'de, V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            ReaderNumber::F64(x) => visitor.visit_f64(x),
            ReaderNumber::U64(x) => visitor.visit_u64(x),
            ReaderNumber::I64(x) => visitor.visit_i64(x),
        }
    }
}

pub(crate) enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T> ops::Deref for Reference<'b, 'c, T>
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
