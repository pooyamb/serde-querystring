//! These tests are meant for the Simple method

use std::collections::HashMap;

use serde::Deserialize;
use serde_querystring::from_str;

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
        from_str("value=1&value=3&value=1337"),
        Ok(p!(vec![1, 3, 1337]))
    );

    // array
    assert_eq!(from_str("value=1&value=3&value=1337"), Ok(p!([1, 3, 1337])));

    // tuple
    assert_eq!(from_str("value=1&value=3&value=1337"), Ok(p!((1, 3, 1337))));
    assert_eq!(
        from_str("value=1&value=3&value=1337"),
        Ok(p!((true, "3", 1337)))
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
    assert_eq!(from_str("God=winner&Right=looser"), Ok(map));

    // unit enums as map values
    #[derive(Debug, Deserialize, PartialEq)]
    struct A {
        looser: Side,
        winner: Side,
    }
    assert_eq!(
        from_str::<A>("looser=Left&winner=God"),
        Ok(A {
            looser: Side::Left,
            winner: Side::God
        })
    );

    // unit enums as map values
    #[derive(Debug, Deserialize, PartialEq)]
    struct VecEnum {
        value: Vec<Side>,
    }

    // unit enums in sequence
    assert_eq!(
        from_str("value=God&value=Left&value=Right"),
        Ok(VecEnum {
            value: vec![Side::God, Side::Left, Side::Right]
        })
    );
}

#[test]
fn deserialize_invalid_sequence() {
    // array length
    assert!(from_str::<Primitive<[usize; 3]>>("value=1&value=3&value=1337&value=999").is_err());

    // tuple length
    assert!(
        from_str::<Primitive<(usize, usize, usize)>>("value=1&value=3&value=1337&value=999")
            .is_err()
    );

    // tuple value types
    assert!(from_str::<Primitive<(&str, usize, &str)>>("value=foo&value=bar&value=baz").is_err());
}
