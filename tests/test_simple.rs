//! These tests are meant for the Simple method

use std::collections::HashMap;

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

#[test]
fn deserialize_sequence() {
    // vector
    assert_eq!(
        from_bytes(b"value=1&value=3&value=1337", Config::Simple),
        Ok(p!(1337))
    );
}

/// Check if unit enums work as keys and values
#[test]
fn deserialize_unit_variants() {
    #[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
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
        from_bytes(b"God=winner&Right=looser", Config::Simple),
        Ok(map)
    );

    // unit enums as map values
    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        looser: Side,
        winner: Side,
    }
    assert_eq!(
        from_bytes::<A>(b"looser=Left&winner=God", Config::Simple),
        Ok(A {
            looser: Side::Left,
            winner: Side::God
        })
    );
}

#[test]
fn deserialize_invalid_sequence() {
    // array length
    assert!(from_bytes::<Primitive<[usize; 3]>>(
        b"value=1&value=3&value=1337&value=999",
        Config::Simple
    )
    .is_err());

    // tuple length
    assert!(from_bytes::<Primitive<(usize, usize, usize)>>(
        b"value=1&value=3&value=1337&value=999",
        Config::Simple
    )
    .is_err());

    // tuple value types
    assert!(from_bytes::<Primitive<(&str, usize, &str)>>(
        b"value=foo&value=bar&value=baz",
        Config::Simple
    )
    .is_err());
}

#[test]
fn deserialize_decoded_keys() {
    // having different encoded kinds of the string `value` for key
    // `v%61lu%65` `valu%65` `value`
    assert_eq!(
        from_bytes(b"v%61lu%65=1&valu%65=2&value=3", Config::Simple),
        Ok(p!(3))
    );
}
