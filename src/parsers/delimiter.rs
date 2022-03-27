use std::{borrow::Cow, collections::BTreeMap};

use crate::decode::{parse_bytes, Reference};

pub struct Key<'a> {
    slice: &'a [u8],
}

impl<'a> Key<'a> {
    fn parse(slice: &'a [u8]) -> Self {
        let mut index = 0;
        while index < slice.len() {
            match slice[index] {
                b'&' | b'=' => break,
                _ => index += 1,
            }
        }

        Self {
            slice: &slice[..index],
        }
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn decode_to<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.slice, scratch)
    }
}

pub struct Value<'a> {
    slice: &'a [u8],
}

impl<'a> Value<'a> {
    fn decode_to<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.slice, scratch)
    }

    pub fn decode(&self) -> Cow<'a, [u8]> {
        let mut scratch = Vec::new();
        self.decode_to(&mut scratch).into_cow()
    }

    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }
}

#[derive(Default)]
pub struct Values<'a> {
    slice: &'a [u8],
}

impl<'a> Values<'a> {
    fn parse(slice: &'a [u8]) -> Option<Self> {
        if *slice.get(0)? == b'&' {
            return None;
        }

        let mut index = 1;
        while index < slice.len() {
            match slice[index] {
                b'&' => break,
                _ => index += 1,
            }
        }

        Some(Self {
            slice: &slice[1..index],
        })
    }

    fn len(&self) -> usize {
        self.slice.len()
    }

    fn values(&self, delimiter: u8) -> impl Iterator<Item = Value<'a>> {
        self.slice
            .split(move |c| *c == delimiter)
            .map(|slice| Value { slice })
    }

    fn decode_to<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.slice, scratch)
    }

    pub fn decode(&self) -> Cow<'a, [u8]> {
        let mut scratch = Vec::new();
        self.decode_to(&mut scratch).into_cow()
    }

    pub fn slice(&self) -> &'a [u8] {
        self.slice
    }
}

pub struct Pair<'a>(Key<'a>, Option<Values<'a>>);

impl<'a> Pair<'a> {
    fn parse(slice: &'a [u8]) -> Self {
        let key = Key::parse(slice);
        let value = Values::parse(&slice[key.len()..]);

        Self(key, value)
    }

    fn len(&self) -> usize {
        match &self.1 {
            Some(v) => self.0.len() + v.len() + 2,
            None => self.0.len() + 1,
        }
    }
}

pub struct DelimiterQS<'a> {
    pairs: BTreeMap<Cow<'a, [u8]>, Pair<'a>>,
    delimiter: u8,
}

impl<'a> DelimiterQS<'a> {
    pub fn parse(slice: &'a [u8], delimiter: u8) -> Self {
        let mut pairs: BTreeMap<Cow<'a, [u8]>, Pair<'a>> = BTreeMap::new();
        let mut scratch = Vec::new();

        let mut index = 0;

        while index < slice.len() {
            let pair = Pair::parse(&slice[index..]);
            index += pair.len();

            let decoded_key = pair.0.decode_to(&mut scratch);

            if let Some(old_pair) = pairs.get_mut(decoded_key.as_ref()) {
                *old_pair = pair;
            } else {
                pairs.insert(decoded_key.into_cow(), pair);
            }
        }

        Self { pairs, delimiter }
    }

    pub fn keys(&self) -> Vec<&Cow<'a, [u8]>> {
        self.pairs.keys().collect()
    }

    pub fn values(&self, key: &'a [u8]) -> Option<Option<Vec<Cow<'a, [u8]>>>> {
        let delimiter = self.delimiter;
        let mut scratch = Vec::new();

        Some(self.pairs.get(key)?.1.as_ref().map(|values| {
            values
                .values(delimiter)
                .map(|v| v.decode_to(&mut scratch).into_cow())
                .collect()
        }))
    }

    pub fn value(&self, key: &'a [u8]) -> Option<Option<Cow<'a, [u8]>>> {
        let mut scratch = Vec::new();

        Some(
            self.pairs
                .get(key)?
                .1
                .as_ref()
                .map(|values| values.decode_to(&mut scratch).into_cow()),
        )
    }

    pub fn raw_values(&self, key: &'a [u8]) -> Option<Option<Vec<Value<'a>>>> {
        Some(
            self.pairs
                .get(key)?
                .1
                .as_ref()
                .map(|values| values.values(self.delimiter).collect()),
        )
    }

    pub fn raw_value(&self, key: &'a [u8]) -> Option<Option<&Values<'a>>> {
        Some(self.pairs.get(key)?.1.as_ref())
    }
}

#[cfg(feature = "serde")]
mod de {
    use crate::de::__implementors::{IntoSizedIterator, ParsedSlice, RawSlice};

    use super::DelimiterQS;

    impl<'a> DelimiterQS<'a> {
        pub(crate) fn into_iter(
            self,
        ) -> impl Iterator<Item = (ParsedSlice<'a>, SeparatorValues<'a>)> {
            let delimiter = self.delimiter;
            self.pairs.into_iter().map(move |(key, pair)| {
                (
                    ParsedSlice(key),
                    SeparatorValues::from_slice(
                        pair.1.map(|v| v.slice).unwrap_or_default(),
                        delimiter,
                    ),
                )
            })
        }
    }

    pub(crate) struct SeparatorValues<'a> {
        slice: &'a [u8],
        delimiter: u8,
    }

    impl<'a> SeparatorValues<'a> {
        fn from_slice(slice: &'a [u8], delimiter: u8) -> Self {
            Self { slice, delimiter }
        }
    }

    impl<'a> IntoSizedIterator<'a> for SeparatorValues<'a> {
        type SizedIterator = SizedValuesIterator<'a>;

        type UnSizedIterator = SizedValuesIterator<'a>;

        fn into_sized_iterator(self, size: usize) -> Result<Self::SizedIterator, crate::de::Error> {
            Ok(SizedValuesIterator::new(
                self.slice,
                self.delimiter,
                Some(size),
            ))
        }

        fn into_unsized_iterator(self) -> Self::UnSizedIterator {
            SizedValuesIterator::new(self.slice, self.delimiter, None)
        }
    }

    pub(crate) struct SizedValuesIterator<'a> {
        slice: &'a [u8],
        delimiter: u8,
        remaining: Option<usize>,
        index: usize,
    }

    impl<'a> SizedValuesIterator<'a> {
        fn new(slice: &'a [u8], delimiter: u8, size: Option<usize>) -> Self {
            Self {
                slice,
                delimiter,
                remaining: size,
                index: 0,
            }
        }

        #[inline]
        fn decrease_remaining(&mut self) {
            if let Some(remaining) = self.remaining {
                self.remaining = Some(remaining - 1)
            }
        }
    }

    impl<'a> Iterator for SizedValuesIterator<'a> {
        type Item = RawSlice<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.slice.len() {
                return None;
            }

            if let Some(remaining) = self.remaining {
                match remaining {
                    0 => {
                        return None;
                    }
                    1 => {
                        self.remaining = Some(0);
                        return Some(RawSlice(&self.slice[self.index..]));
                    }
                    _ => {}
                }
            }

            let start = self.index;
            for c in &self.slice[self.index..] {
                if *c == self.delimiter {
                    let end = self.index;
                    self.index += 1;

                    self.decrease_remaining();
                    return Some(RawSlice(&self.slice[start..end]));
                }
                self.index += 1;
            }

            self.decrease_remaining();
            Some(RawSlice(&self.slice[start..]))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::DelimiterQS;

    #[test]
    fn parse_pair() {
        let slice = b"key=value";

        let parser = DelimiterQS::parse(slice, b'|');

        assert_eq!(parser.keys(), vec![&Cow::Borrowed(b"key")]);
        assert_eq!(
            parser.values(b"key"),
            Some(Some(vec![Cow::Borrowed("value".as_bytes())]))
        );
        assert_eq!(
            parser.value(b"key"),
            Some(Some(Cow::Borrowed("value".as_bytes())))
        );

        assert_eq!(parser.values(b"test"), None);
    }

    #[test]
    fn parse_multiple_pairs() {
        let slice = b"foo=bar&foobar=baz&qux=box";

        let parser = DelimiterQS::parse(slice, b'|');

        assert_eq!(
            parser.values(b"foo"),
            Some(Some(vec!["bar".as_bytes().into()]))
        );
        assert_eq!(
            parser.values(b"foobar"),
            Some(Some(vec!["baz".as_bytes().into()]))
        );
        assert_eq!(
            parser.values(b"qux"),
            Some(Some(vec!["box".as_bytes().into()]))
        );
    }

    #[test]
    fn parse_no_value() {
        let slice = b"foo&foobar=";

        let parser = DelimiterQS::parse(slice, b'|');

        // Expecting a vector of values
        assert_eq!(parser.values(b"foo"), Some(None));
        assert_eq!(
            parser.values(b"foobar"),
            Some(Some(vec!["".as_bytes().into()]))
        );

        // Expecting a single value
        assert_eq!(parser.value(b"foo"), Some(None));
        assert_eq!(parser.value(b"foobar"), Some(Some("".as_bytes().into())));
    }

    #[test]
    fn parse_multiple_values() {
        let slice = b"foo=bar|baz|foobar||";

        let parser = DelimiterQS::parse(slice, b'|');

        assert_eq!(
            parser.values(b"foo"),
            Some(Some(vec![
                "bar".as_bytes().into(),
                "baz".as_bytes().into(),
                "foobar".as_bytes().into(),
                "".as_bytes().into(),
                "".as_bytes().into()
            ]))
        );

        let slice = b"foo=bar,baz,foobar,,";

        let parser = DelimiterQS::parse(slice, b',');

        assert_eq!(
            parser.values(b"foo"),
            Some(Some(vec![
                "bar".as_bytes().into(),
                "baz".as_bytes().into(),
                "foobar".as_bytes().into(),
                "".as_bytes().into(),
                "".as_bytes().into()
            ]))
        );
    }
}