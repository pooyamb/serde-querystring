//! These tests are meant for the `UrlEncodedQS` method

use serde::Deserialize;
use serde_querystring::de::{from_bytes, Config};

/// It is a helper struct we use to test primitive types
/// as we don't support anything beside maps/structs at the root level
#[derive(Debug, PartialEq, Deserialize)]
struct Primitive<T> {
    value: T,
}

impl<T> Primitive<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

macro_rules! p {
    ($value:expr, $type: ty) => {
        Primitive::<$type>::new($value)
    };
    ($value:expr) => {
        Primitive::new($value)
    };
}

#[derive(Debug, Deserialize, PartialEq)]
struct UrlEncoded<'a> {
    #[serde(borrow)]
    foo: &'a str,
    foobar: u32,
    bar: Option<u32>,
}

#[test]
fn deserialize_urlencoded() {
    assert_eq!(
        from_bytes(b"foo=bar&foobar=1337&foo=baz&bar=13", Config::UrlEncoded),
        Ok(UrlEncoded {
            foo: "baz",
            foobar: 1337,
            bar: Some(13)
        })
    )
}

#[test]
fn deserialize_repeated_keys() {
    // vector
    assert_eq!(
        from_bytes(b"value=1&value=3&value=1337", Config::UrlEncoded),
        Ok(p!(1337))
    );
}

#[test]
fn deserialize_decoded_keys() {
    // having different encoded kinds of the string `value` for key
    // `v%61lu%65` `valu%65` `value`
    assert_eq!(
        from_bytes(b"v%61lu%65=1&valu%65=2&value=3", Config::UrlEncoded),
        Ok(p!(3))
    );
}

#[test]
fn deserialize_invalid_sequence() {
    // array length
    assert!(from_bytes::<Primitive<[usize; 3]>>(
        b"value=1&value=3&value=1337&value=999",
        Config::UrlEncoded
    )
    .is_err());

    // tuple length
    assert!(from_bytes::<Primitive<(usize, usize, usize)>>(
        b"value=1&value=3&value=1337&value=999",
        Config::UrlEncoded
    )
    .is_err());

    // tuple value types
    assert!(from_bytes::<Primitive<(&str, usize, &str)>>(
        b"value=foo&value=bar&value=baz",
        Config::UrlEncoded
    )
    .is_err());
}
