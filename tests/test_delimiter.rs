//! These tests are meant for the `DelimiterQS` method

use std::collections::HashMap;

use _serde::Deserialize;
use serde_querystring::de::{from_bytes, ParseMode};

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
struct Delimiter<'a> {
    #[serde(borrow)]
    foo: &'a str,
    foobar: u32,
    bar: Option<u32>,
    vec: Vec<u32>,
}

#[test]
fn deserialize_delimiter() {
    assert_eq!(
        from_bytes(
            b"foo=bar&foobar=1337&foo=baz&bar=13&vec=1337|11",
            ParseMode::Delimiter(b'|')
        ),
        Ok(Delimiter {
            foo: "baz",
            foobar: 1337,
            bar: Some(13),
            vec: vec![1337, 11]
        })
    )
}

#[test]
fn deserialize_string_with_delimiter() {
    assert_eq!(
        from_bytes(b"value=1337|11", ParseMode::Delimiter(b'|')),
        Ok(p!("1337|11"))
    )
}

#[test]
fn deserialize_repeated_keys() {
    // vector
    assert_eq!(
        from_bytes(b"value=1|2|3&value=4|5|6", ParseMode::Delimiter(b'|')),
        Ok(p!(vec![4, 5, 6]))
    );

    // vector
    assert_eq!(
        from_bytes(b"value=1337&value=7331", ParseMode::Delimiter(b'|')),
        Ok(p!(7331))
    );
}

#[test]
fn deserialize_sequence() {
    // vector
    assert_eq!(
        from_bytes(b"value=1|3|1337", ParseMode::Delimiter(b'|')),
        Ok(p!(vec![1, 3, 1337]))
    );
    assert_eq!(
        from_bytes(b"value=1,3,1337", ParseMode::Delimiter(b',')),
        Ok(p!(vec![1, 3, 1337]))
    );

    // array
    assert_eq!(
        from_bytes(b"value=1|3|1337", ParseMode::Delimiter(b'|')),
        Ok(p!([1, 3, 1337]))
    );

    // tuple
    assert_eq!(
        from_bytes(b"value=1|3|1337", ParseMode::Delimiter(b'|')),
        Ok(p!((1, 3, 1337)))
    );
    assert_eq!(
        from_bytes(b"value=1|3|1337", ParseMode::Delimiter(b'|')),
        Ok(p!((true, "3", 1337)))
    );

    // More values than expected, we will try to recover if possible
    assert_eq!(
        from_bytes(
            b"value=more|values|than|expected",
            ParseMode::Delimiter(b'|')
        ),
        Ok(p!(("more", "values", "than|expected")))
    );
}

#[test]
fn deserialize_optional_seq() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(crate = "_serde")]
    struct OptionalSeq {
        seq: Option<Vec<u32>>,
    }

    assert_eq!(
        from_bytes(b"key=value", ParseMode::Delimiter(b'|')),
        Ok(OptionalSeq { seq: None })
    );
    assert_eq!(
        from_bytes(b"seq=20|30|40", ParseMode::Delimiter(b'|')),
        Ok(OptionalSeq {
            seq: Some(vec![20, 30, 40])
        })
    );
}

/// Check if unit enums work as keys and values
#[test]
fn deserialize_unit_enums() {
    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
    #[serde(crate = "_serde")]
    enum Side {
        Left,
        Right,
        God,
    }

    // unit enums as map keys
    let mut map = HashMap::new();
    map.insert(Side::God, "winner");
    map.insert(Side::Right, "looser");
    assert_eq!(
        from_bytes(b"God=winner&Right=looser", ParseMode::Delimiter(b'|')),
        Ok(map)
    );

    // unit enums as map values
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(crate = "_serde")]
    struct A {
        looser: Side,
        winner: Side,
    }
    assert_eq!(
        from_bytes::<A>(b"looser=Left&winner=God", ParseMode::Delimiter(b'|')),
        Ok(A {
            looser: Side::Left,
            winner: Side::God
        })
    );

    // unit enums as map values
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(crate = "_serde")]
    struct VecEnum {
        value: Vec<Side>,
    }

    // unit enums in sequence
    assert_eq!(
        from_bytes(b"value=God|Left|Right", ParseMode::Delimiter(b'|')),
        Ok(VecEnum {
            value: vec![Side::God, Side::Left, Side::Right]
        })
    );
}

#[test]
fn deserialize_invalid_sequence() {
    // array length
    assert!(
        from_bytes::<Primitive<[usize; 3]>>(b"value=1|3|1337|999", ParseMode::Delimiter(b'|'))
            .is_err()
    );

    // tuple length
    assert!(from_bytes::<Primitive<(usize, usize, usize)>>(
        b"1|3|1337|999",
        ParseMode::Delimiter(b'|')
    )
    .is_err());

    // tuple value types
    assert!(from_bytes::<Primitive<(&str, usize, &str)>>(
        b"value=foo|bar|baz",
        ParseMode::Delimiter(b'|')
    )
    .is_err());
}
