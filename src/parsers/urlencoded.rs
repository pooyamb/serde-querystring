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

    fn decode_to<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.0, scratch)
    }
}

struct Value<'a>(&'a [u8]);

impl<'a> Value<'a> {
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

        Some(Self(&slice[1..index]))
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn decode_to<'s>(&self, scratch: &'s mut Vec<u8>) -> Reference<'a, 's, [u8]> {
        parse_bytes(self.0, scratch)
    }
}

struct Pair<'a>(Key<'a>, Option<Value<'a>>);

impl<'a> Pair<'a> {
    fn parse(slice: &'a [u8]) -> Self {
        let key = Key::parse(slice);
        let value = Value::parse(&slice[key.len()..]);

        Self(key, value)
    }

    fn len(&self) -> usize {
        match &self.1 {
            Some(v) => self.0.len() + v.len() + 2,
            None => self.0.len() + 1,
        }
    }
}

pub struct UrlEncodedQS<'a> {
    pairs: BTreeMap<Cow<'a, [u8]>, Pair<'a>>,
}

impl<'a> UrlEncodedQS<'a> {
    pub fn parse(slice: &'a [u8]) -> Self {
        let mut pairs = BTreeMap::new();
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

        Self { pairs }
    }

    /// Returns a vector containing all the keys in querystring
    pub fn keys(&self) -> Vec<&Cow<'a, [u8]>> {
        self.pairs.keys().collect()
    }

    /// Returns the last value assigned to a key
    /// It will return None if the key didn't exist in the querystring
    /// It will return Some(None) if the last assignment to a key didn't have a value, ex `&key&`
    pub fn value(&self, key: &'a [u8]) -> Option<Option<Cow<'a, [u8]>>> {
        let mut scratch = Vec::new();
        self.pairs
            .get(key)
            .map(|p| p.1.as_ref().map(|v| v.decode_to(&mut scratch).into_cow()))
    }
}

#[cfg(feature = "serde")]
mod de {
    use crate::de::__implementors::{OptionalRawSlice, ParsedSlice};

    use super::UrlEncodedQS;

    impl<'a> UrlEncodedQS<'a> {
        pub(crate) fn into_iter(
            self,
        ) -> impl Iterator<Item = (ParsedSlice<'a>, OptionalRawSlice<'a>)> {
            self.pairs
                .into_iter()
                .map(|(key, pair)| (ParsedSlice(key), OptionalRawSlice(pair.1.map(|v| v.0))))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::UrlEncodedQS;

    #[test]
    fn parse_pair() {
        let slice = b"key=value";

        let parser = UrlEncodedQS::parse(slice);

        assert_eq!(parser.keys(), vec![&Cow::Borrowed(b"key")]);
        assert_eq!(
            parser.value(b"key"),
            Some(Some(Cow::Borrowed("value".as_bytes())))
        );
    }

    #[test]
    fn parse_multiple_pairs() {
        let slice = b"foo=bar&foobar=baz&qux=box";

        let parser = UrlEncodedQS::parse(slice);

        assert_eq!(parser.value(b"foo"), Some(Some("bar".as_bytes().into())));
        assert_eq!(parser.value(b"foobar"), Some(Some("baz".as_bytes().into())));
        assert_eq!(parser.value(b"qux"), Some(Some("box".as_bytes().into())));
    }

    #[test]
    fn parse_no_value() {
        let slice = b"foo&foobar=&foo2";

        let parser = UrlEncodedQS::parse(slice);

        assert_eq!(parser.value(b"foo3"), None);
        assert_eq!(parser.value(b"foo2"), Some(None));
        assert_eq!(parser.value(b"foo"), Some(None));
        assert_eq!(parser.value(b"foobar"), Some(Some("".as_bytes().into())));
    }

    #[test]
    fn parse_multiple_values() {
        let slice = b"foo=bar&foo=baz&foo=foobar&foo&foo=";

        let parser = UrlEncodedQS::parse(slice);

        assert_eq!(parser.value(b"foo"), Some(Some("".as_bytes().into())));
    }
}
