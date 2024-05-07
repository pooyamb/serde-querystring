use std::{borrow::Cow, collections::BTreeMap};

use crate::decode::{parse_bytes, Reference};

struct Key<'a>(&'a [u8]);

impl<'a> Key<'a> {
    fn parse(slice: &'a [u8]) -> Self {
        let mut index = 0;
        while index < slice.len() {
            match slice[index] {
                b'&' | b'=' => break,
                _ => index += 1,
            }
        }

        Self(&slice[..index])
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn decode<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.0, scratch)
    }
}

struct Value<'a>(&'a [u8]);

impl<'a> Value<'a> {
    fn parse(slice: &'a [u8]) -> Option<Self> {
        if *slice.first()? == b'&' {
            return None;
        }

        let mut index = 1;
        while index < slice.len() {
            match slice[index] {
                b'&' => break,
                _ => index += 1,
            }
        }

        Some(Self(&slice[1..index]))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn decode<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.0, scratch)
    }

    fn slice(&self) -> &'a [u8] {
        self.0
    }
}

struct Pair<'a>(Key<'a>, Option<Value<'a>>);

impl<'a> Pair<'a> {
    fn parse(slice: &'a [u8]) -> Self {
        let key = Key::parse(slice);
        let value = Value::parse(&slice[key.len()..]);

        Self(key, value)
    }

    /// It report how many chars we should move forward after this pair, to see a new one.
    /// It might report invalid result at the end of the slice,
    /// so calling site should check the validity of resulting index
    fn skip_len(&self) -> usize {
        match &self.1 {
            Some(v) => self.0.len() + v.len() + 2,
            None => self.0.len() + 1,
        }
    }
}

/// A querystring parser with support for vectors/lists of values by repeating keys.
///
/// # Note
/// Keys are decoded when calling the `parse` method, but values are lazily decoded when you
/// call the `value` method for their keys.
///
/// # Example
/// ```rust
///# use std::borrow::Cow;
/// use serde_querystring::DuplicateQS;
///
/// let slice = b"foo=bar&foo=baz&foo&foo=";
///
/// let parser = DuplicateQS::parse(slice);
///
/// // `values` method returns ALL the values as a vector.
/// assert_eq!(
///    parser.values(b"foo"),
///    Some(vec![
///        Some("bar".as_bytes().into()),
///        Some("baz".as_bytes().into()),
///        None,
///        Some("".as_bytes().into())
///    ])
///);
///
/// // `value` method returns the last seen value
/// assert_eq!(parser.value(b"foo"), Some(Some("".as_bytes().into())));
/// ```
pub struct DuplicateQS<'a> {
    pairs: BTreeMap<Cow<'a, [u8]>, Vec<Pair<'a>>>,
}

impl<'a> DuplicateQS<'a> {
    /// Parse a slice of bytes into a `DuplicateQS`
    pub fn parse(slice: &'a [u8]) -> Self {
        let mut pairs: BTreeMap<Cow<'a, [u8]>, Vec<Pair<'a>>> = BTreeMap::new();
        let mut scratch = Vec::new();

        let mut index = 0;

        while index < slice.len() {
            let pair = Pair::parse(&slice[index..]);
            index += pair.skip_len();

            let decoded_key = pair.0.decode(&mut scratch);

            if let Some(values) = pairs.get_mut(decoded_key.as_ref()) {
                values.push(pair);
            } else {
                pairs.insert(decoded_key.into_cow(), vec![pair]);
            }
        }

        Self { pairs }
    }

    /// Returns a vector containing all the keys in querystring.
    pub fn keys(&self) -> Vec<&Cow<'a, [u8]>> {
        self.pairs.keys().collect()
    }

    /// Returns a vector containing all the values assigned to a key.
    ///
    /// It returns None if the **key doesn't exist** in the querystring,
    /// the resulting vector may contain None if the **key had assignments without a value**, ex `&key&`
    ///
    /// # Note
    /// Percent decoding the value is done on-the-fly **every time** this function is called.
    pub fn values(&self, key: &'a [u8]) -> Option<Vec<Option<Cow<'a, [u8]>>>> {
        let mut scratch = Vec::new();

        Some(
            self.pairs
                .get(key)?
                .iter()
                .map(|p| p.1.as_ref().map(|v| v.decode(&mut scratch).into_cow()))
                .collect(),
        )
    }

    /// Returns the last value assigned to a key.
    ///
    /// It returns `None` if the **key doesn't exist** in the querystring,
    /// and returns `Some(None)` if the last assignment to a **key doesn't have a value**, ex `"&key&"`
    ///
    /// # Note
    /// Percent decoding the value is done on-the-fly **every time** this function is called.
    pub fn value(&self, key: &'a [u8]) -> Option<Option<Cow<'a, [u8]>>> {
        let mut scratch = Vec::new();

        self.pairs
            .get(key)?
            .iter()
            .last()
            .map(|p| p.1.as_ref().map(|v| v.decode(&mut scratch).into_cow()))
    }
}

#[cfg(feature = "serde")]
mod de {
    use _serde::Deserialize;

    use crate::de::{
        Error, ErrorKind, QSDeserializer,
        __implementors::{DecodedSlice, IntoRawSlices, RawSlice},
    };

    use super::DuplicateQS;

    impl<'a> DuplicateQS<'a> {
        /// Deserialize the parsed slice into T
        pub fn deserialize<T: Deserialize<'a>>(self) -> Result<T, Error> {
            T::deserialize(QSDeserializer::new(self.into_iter()))
        }

        pub(crate) fn into_iter(
            self,
        ) -> impl Iterator<
            Item = (
                DecodedSlice<'a>,
                DuplicateValueIter<impl Iterator<Item = RawSlice<'a>>>,
            ),
        > {
            self.pairs.into_iter().map(|(key, pairs)| {
                (
                    DecodedSlice(key),
                    DuplicateValueIter(
                        pairs
                            .into_iter()
                            .map(|v| RawSlice(v.1.map(|v| v.slice()).unwrap_or_default())),
                    ),
                )
            })
        }
    }

    pub(crate) struct DuplicateValueIter<I>(I);

    impl<'a, I> IntoRawSlices<'a> for DuplicateValueIter<I>
    where
        I: Iterator<Item = RawSlice<'a>>,
    {
        type SizedIterator = I;
        type UnSizedIterator = I;

        #[inline]
        fn into_sized_iterator(self, size: usize) -> Result<I, Error> {
            if self.0.size_hint().0 == size {
                Ok(self.0)
            } else {
                Err(Error::new(ErrorKind::InvalidLength))
            }
        }

        #[inline]
        fn into_unsized_iterator(self) -> I {
            self.0
        }

        #[inline]
        fn into_single_slice(self) -> RawSlice<'a> {
            self.0
                .last()
                .expect("Iterator has at least one value in it")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::DuplicateQS;

    #[test]
    fn parse_pair() {
        let slice = b"key=value";

        let parser = DuplicateQS::parse(slice);

        assert_eq!(parser.keys(), vec![&Cow::Borrowed(b"key")]);
        assert_eq!(
            parser.values(b"key"),
            Some(vec![Some(Cow::Borrowed("value".as_bytes()))])
        );
        assert_eq!(
            parser.value(b"key"),
            Some(Some(Cow::Borrowed("value".as_bytes())))
        );
    }

    #[test]
    fn parse_multiple_pairs() {
        let slice = b"foo=bar&foobar=baz&qux=box";

        let parser = DuplicateQS::parse(slice);

        assert_eq!(
            parser.values(b"foo"),
            Some(vec![Some("bar".as_bytes().into())])
        );
        assert_eq!(
            parser.values(b"foobar"),
            Some(vec![Some("baz".as_bytes().into())])
        );
        assert_eq!(
            parser.values(b"qux"),
            Some(vec![Some("box".as_bytes().into())])
        );
    }

    #[test]
    fn parse_no_value() {
        let slice = b"foo&foobar=";

        let parser = DuplicateQS::parse(slice);

        assert_eq!(parser.value(b"key"), None);
        assert_eq!(parser.values(b"key"), None);
        assert_eq!(parser.value(b"foo"), Some(None));
        assert_eq!(parser.values(b"foo"), Some(vec![None]));
        assert_eq!(
            parser.values(b"foobar"),
            Some(vec![Some("".as_bytes().into())])
        );
        assert_eq!(parser.value(b"foobar"), Some(Some("".as_bytes().into())));
    }

    #[test]
    fn parse_multiple_values() {
        let slice = b"foo=bar&foo=baz&foo=foobar&foo&foo=";

        let parser = DuplicateQS::parse(slice);

        assert_eq!(
            parser.values(b"foo"),
            Some(vec![
                Some("bar".as_bytes().into()),
                Some("baz".as_bytes().into()),
                Some("foobar".as_bytes().into()),
                None,
                Some("".as_bytes().into())
            ])
        );

        assert_eq!(parser.value(b"foo"), Some(Some("".as_bytes().into())));
    }
}
