//! These tests are meant for the `BracketsQS` method

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
struct Brackets<'a> {
    #[serde(borrow)]
    foo: &'a str,
    foobar: u32,
    bar: Option<u32>,
    vec: Vec<u32>,
}

#[test]
fn deserialize_brackets() {
    assert_eq!(
        from_bytes(
            b"foo=bar&foobar=1337&foo=baz&bar=13&vec[1]=1337&vec=11",
            ParseMode::Brackets
        ),
        Ok(Brackets {
            foo: "baz",
            foobar: 1337,
            bar: Some(13),
            vec: vec![11, 1337]
        })
    );
}

#[test]
fn deserialize_sequence() {
    // vector
    assert_eq!(
        from_bytes(b"value[3]=1337&value[2]=3&value[1]=1", ParseMode::Brackets),
        Ok(p!(vec![1, 3, 1337]))
    );

    // array
    assert_eq!(
        from_bytes(b"value[3]=1337&value[2]=3&value[1]=1", ParseMode::Brackets),
        Ok(p!([1, 3, 1337]))
    );

    // tuple
    assert_eq!(
        from_bytes(b"value[0]=1&value[1]=3&value[2]=1337", ParseMode::Brackets),
        Ok(p!((1, 3, 1337)))
    );
    assert_eq!(
        from_bytes(b"value[0]=1&value[1]=3&value[2]=1337", ParseMode::Brackets),
        Ok(p!((true, "3", 1337)))
    );
}

#[test]
fn deserialize_struct_value() {
    // vector
    assert_eq!(
        from_bytes(
            b"value[value][3]=1337&value[value][2]=3&value[value][1]=1",
            ParseMode::Brackets
        ),
        Ok(p!(p!(vec![1, 3, 1337])))
    );

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(crate = "_serde")]
    struct Sample2<'a> {
        #[serde(borrow)]
        foo: Primitive<&'a str>,
        #[serde(borrow)]
        qux: Primitive<&'a str>,
    }

    assert_eq!(
        from_bytes(b"foo[value]=bar&qux[value]=foobar", ParseMode::Brackets),
        Ok(Sample2 {
            foo: p!("bar"),
            qux: p!("foobar")
        })
    )
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
        from_bytes(b"God=winner&Right=looser", ParseMode::Brackets),
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
        from_bytes::<A>(b"looser=Left&winner=God", ParseMode::Brackets),
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
        from_bytes(b"value=God&value=Left&value=Right", ParseMode::Brackets),
        Ok(VecEnum {
            value: vec![Side::God, Side::Left, Side::Right]
        })
    );
}
/// Check if unit enums work as keys and values
#[test]
fn deserialize_enums() {
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(crate = "_serde")]
    enum Enum {
        Unit,
        NewType(i32),
        Tuple(i32, i32),
        Struct { bee: i32, loose: i32 },
    }

    assert_eq!(
        from_bytes(b"value=Unit", ParseMode::Brackets),
        Ok(p!(Enum::Unit))
    );
    assert_eq!(
        from_bytes(b"value[NewType]=2022", ParseMode::Brackets),
        Ok(p!(Enum::NewType(2022)))
    );
    assert_eq!(
        from_bytes(
            b"value[Tuple][0]=100&value[Tuple][1]=200",
            ParseMode::Brackets
        ),
        Ok(p!(Enum::Tuple(100, 200)))
    );
    assert_eq!(
        from_bytes(
            b"value[Struct][bee]=833&value[Struct][loose]=10053",
            ParseMode::Brackets
        ),
        Ok(p!(Enum::Struct {
            bee: 833,
            loose: 10053
        }))
    );

    // Assigning a key again should override its value
    assert_eq!(
        from_bytes(
            b"value[Struct][bee]=833&value[Struct][loose]=10053&value[NewType]=100",
            ParseMode::Brackets
        ),
        Ok(p!(Enum::NewType(100)))
    );
    assert_eq!(
        from_bytes(
            b"value[Struct][bee]=833&value[NewType]=100&value[Struct][loose]=10053",
            ParseMode::Brackets
        ),
        Ok(p!(Enum::Struct {
            bee: 833,
            loose: 10053
        }))
    );
    assert_eq!(
        from_bytes(
            b"value[Struct][bee]=833&value[NewType]=100&value[Struct][loose]=10053&value=Unit",
            ParseMode::Brackets
        ),
        Ok(p!(Enum::Unit))
    );
}

#[test]
fn deserialize_invalid_sequence() {
    // array length
    assert!(from_bytes::<Primitive<[usize; 3]>>(
        b"value=1&value=3&value=1337&value=999",
        ParseMode::Brackets
    )
    .is_err());

    // tuple length
    assert!(from_bytes::<Primitive<(usize, usize, usize)>>(
        b"value=1&value=3&value=1337&value=999",
        ParseMode::Brackets
    )
    .is_err());

    // tuple value types
    assert!(from_bytes::<Primitive<(&str, usize, &str)>>(
        b"value=foo&value=bar&value=baz",
        ParseMode::Brackets
    )
    .is_err());
}

#[test]
fn deserialize_decoded_keys() {
    // having different encoded kinds of the string `value` for key
    // `v%61lu%65` `valu%65` `value`
    assert_eq!(
        from_bytes(b"v%61lu%65=1&valu%65=2&value=3", ParseMode::Brackets),
        Ok(p!(vec!["1", "2", "3"]))
    );
}
