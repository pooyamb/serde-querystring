//! These tests are meant for the `UrlEncodedQS` method

use _serde::Deserialize;
use serde_querystring::de::{from_bytes, ErrorKind, ParseMode};

/// It is a helper struct we use to test primitive types
/// as we don't support anything beside maps/structs at the root level
#[derive(Debug, PartialEq, Deserialize)]
#[serde(crate = "_serde")]
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
#[serde(crate = "_serde")]
struct UrlEncoded<'a> {
    #[serde(borrow)]
    foo: &'a str,
    foobar: u32,
    bar: Option<u32>,
}

#[test]
fn deserialize_urlencoded() {
    assert_eq!(
        from_bytes(b"foo=bar&foobar=1337&foo=baz&bar=13", ParseMode::UrlEncoded),
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
        from_bytes(b"value=1&value=3&value=1337", ParseMode::UrlEncoded),
        Ok(p!(1337))
    );
}

#[test]
fn deserialize_decoded_keys() {
    // having different encoded kinds of the string `value` for key
    // `v%61lu%65` `valu%65` `value`
    assert_eq!(
        from_bytes(b"v%61lu%65=1&valu%65=2&value=3", ParseMode::UrlEncoded),
        Ok(p!(3))
    );
}

#[test]
fn deserialize_error_type() {
    // we don't support sequences in this mode
    assert_eq!(
        from_bytes::<Primitive<[usize; 3]>>(
            b"value=1&value=3&value=1337&value=999",
            ParseMode::UrlEncoded,
        )
        .unwrap_err()
        .kind,
        ErrorKind::InvalidType
    );

    assert_eq!(
        from_bytes::<Primitive<(usize, usize, usize)>>(
            b"value=1&value=3&value=1337&value=999",
            ParseMode::UrlEncoded,
        )
        .unwrap_err()
        .kind,
        ErrorKind::InvalidType
    );

    // We don't support non-unit enums
    #[derive(Debug, Deserialize)]
    #[serde(crate = "_serde")]
    #[allow(dead_code)]
    enum ValueEnum {
        A(i32, i32),
        B(i32),
        C {},
    }

    assert_eq!(
        from_bytes::<Primitive<ValueEnum>>(b"value=A&value=B&key=value", ParseMode::UrlEncoded)
            .unwrap_err()
            .kind,
        ErrorKind::InvalidType
    );
    assert_eq!(
        from_bytes::<Primitive<ValueEnum>>(b"value=B", ParseMode::UrlEncoded)
            .unwrap_err()
            .kind,
        ErrorKind::InvalidType
    );
    assert_eq!(
        from_bytes::<Primitive<ValueEnum>>(b"value=C", ParseMode::UrlEncoded)
            .unwrap_err()
            .kind,
        ErrorKind::InvalidType
    );
}
