//! These tests are common between different deserialization methods

use std::collections::HashMap;

use _serde::Deserialize;
use serde_querystring::de::{from_bytes, Error, ErrorKind, ParseMode};

fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T, Error> {
    from_bytes(input.as_bytes(), ParseMode::UrlEncoded)
}

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

#[derive(Debug, Deserialize, Hash, Eq, PartialEq)]
#[serde(crate = "_serde")]
enum Side {
    Left,
    Right,
    God,
}

#[test]
fn deserialize_integer_valid() {
    // u8
    assert_eq!(from_str("value=255"), Ok(p!(u8::MAX)));
    assert_eq!(from_str("value=0"), Ok(p!(u8::MIN)));

    // i8
    assert_eq!(from_str("value=127"), Ok(p!(i8::MAX)));
    assert_eq!(from_str("value=-128"), Ok(p!(i8::MIN)));

    // u16
    assert_eq!(from_str("value=65535"), Ok(p!(u16::MAX)));
    assert_eq!(from_str("value=0"), Ok(p!(u16::MIN)));

    // i16
    assert_eq!(from_str("value=32767"), Ok(p!(i16::MAX)));
    assert_eq!(from_str("value=-32768"), Ok(p!(i16::MIN)));

    // u32
    assert_eq!(from_str("value=4294967295"), Ok(p!(u32::MAX)));
    assert_eq!(from_str("value=0"), Ok(p!(u32::MIN)));

    // i32
    assert_eq!(from_str("value=2147483647"), Ok(p!(i32::MAX)));
    assert_eq!(from_str("value=-2147483648"), Ok(p!(i32::MIN)));

    // u64
    assert_eq!(from_str("value=18446744073709551615"), Ok(p!(u64::MAX)));
    assert_eq!(from_str("value=0"), Ok(p!(u64::MIN)));

    // i64
    assert_eq!(from_str("value=9223372036854775807"), Ok(p!(i64::MAX)));
    assert_eq!(from_str("value=-9223372036854775808"), Ok(p!(i64::MIN)));

    // In keys
    let mut map = HashMap::new();
    map.insert(-1337_i64, "value1");
    map.insert(-7331_i64, "value2");
    map.insert(1337_i64, "value3");
    map.insert(7331_i64, "value4");
    assert_eq!(
        from_bytes(
            b"-1337=value1&-7331=value2&1337=value3&7331=value4",
            ParseMode::UrlEncoded
        ),
        Ok(map)
    );
}

#[test]
fn deserialize_float_valid() {
    assert_eq!(from_str("value=1.2"), Ok(p!(1.2_f64)));
    assert_eq!(from_str("value=-1.2"), Ok(p!(-1.2_f64)));
    assert_eq!(from_str("value=1.2E5"), Ok(p!(1.2E5_f64)));
    assert_eq!(from_str("value=-1.2E5"), Ok(p!(-1.2E5_f64)));
    assert_eq!(from_str("value=1.2E-5"), Ok(p!(1.2E-5_f64)));
    assert_eq!(from_str("value=-1.2E-5"), Ok(p!(-1.2E-5_f64)));
    assert_eq!(
        from_str("value=18446744073709551616"),
        Ok(p!(18_446_744_073_709_551_616_f64))
    );
    assert_eq!(
        from_str("value=-18446744073709551616"),
        Ok(p!(-18_446_744_073_709_551_616_f64))
    );
}

/// Check if different boolean idents work
#[test]
fn deserialize_bool() {
    // true
    assert_eq!(from_str("value=1"), Ok(p!(true)));
    assert_eq!(from_str("value=on"), Ok(p!(true)));
    assert_eq!(from_str("value=true"), Ok(p!(true)));

    // false
    assert_eq!(from_str("value=0"), Ok(p!(false)));
    assert_eq!(from_str("value=off"), Ok(p!(false)));
    assert_eq!(from_str("value=false"), Ok(p!(false)));

    // In keys
    let mut map = HashMap::new();
    map.insert(true, "value1");
    map.insert(false, "value2");
    assert_eq!(
        from_bytes(b"true=value1&off=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

/// Check if we can directly deserialize non percent encoded values to str
#[test]
fn deserialize_str() {
    assert_eq!(from_str("value=test"), Ok(p!("test")));

    // We don't make assumptions about numbers
    assert_eq!(from_str("value=250"), Ok(p!("250")));
    assert_eq!(from_str("value=-25"), Ok(p!("-25")));

    // In keys
    let mut map = HashMap::new();
    map.insert("some", "value1");
    map.insert("bytes", "value2");
    assert_eq!(
        from_bytes(b"some=value1&bytes=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_strings() {
    assert_eq!(from_str("value=foo"), Ok(p!("foo".to_string())));

    // percent decoded
    assert_eq!(
        from_str("value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF"),
        Ok(p!("بابابزرگ".to_string()))
    );

    // Plus in strings should be replaced with space
    assert_eq!(from_str("value=rum+rum"), Ok(p!("rum rum".to_string())));

    // In keys
    let mut map = HashMap::new();
    map.insert(String::from("some"), "value1");
    map.insert(String::from("st ri ng"), "value2");
    assert_eq!(
        from_bytes(b"some=value1&st+ri+ng=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_bytes() {
    assert_eq!(
        from_str("value=test"),
        Ok(p!(serde_bytes::Bytes::new(b"test")))
    );

    // We don't make assumptions about numbers
    assert_eq!(
        from_str("value=250"),
        Ok(p!(serde_bytes::Bytes::new(b"250")))
    );
    assert_eq!(
        from_str("value=-25"),
        Ok(p!(serde_bytes::Bytes::new(b"-25")))
    );

    // In keys
    let mut map = HashMap::new();
    map.insert(serde_bytes::Bytes::new(b"some"), "value1");
    map.insert(serde_bytes::Bytes::new(b"bytes"), "value2");
    assert_eq!(
        from_bytes(b"some=value1&bytes=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_byte_vecs() {
    assert_eq!(
        from_str("value=foo"),
        Ok(p!(serde_bytes::ByteBuf::from("foo")))
    );

    // percent decoded
    assert_eq!(
        from_str("value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF"),
        Ok(p!(serde_bytes::ByteBuf::from("بابابزرگ")))
    );

    // Plus in strings should be replaced with space
    assert_eq!(
        from_str("value=rum+rum"),
        Ok(p!(serde_bytes::ByteBuf::from("rum rum")))
    );

    // In keys
    let mut map = HashMap::new();
    map.insert(serde_bytes::ByteBuf::from("some"), "value1");
    map.insert(serde_bytes::ByteBuf::from("by\0te s"), "value2");
    assert_eq!(
        from_bytes(b"some=value1&by%00te+s=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

/// Check if unit enums work as keys and values
#[test]
fn deserialize_unit_enum() {
    // unit enums as map values
    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(crate = "_serde")]
    struct A {
        looser: Side,
        winner: Side,
    }
    assert_eq!(
        from_bytes::<A>(b"looser=Left&winner=God", ParseMode::UrlEncoded),
        Ok(A {
            looser: Side::Left,
            winner: Side::God
        })
    );

    // In keys
    let mut map = HashMap::new();
    map.insert(Side::God, "winner");
    map.insert(Side::Right, "looser");
    assert_eq!(
        from_bytes(b"God=winner&Right=looser", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_option() {
    assert_eq!(from_str("value=1337"), Ok(p!(Some(1337), Option<u32>)));
    assert_eq!(from_str("value="), Ok(p!(None, Option<u32>)));
    assert_eq!(from_str("value=1337&value="), Ok(p!(None, Option<u32>)));
}

#[test]
fn deserialize_new_type() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    #[serde(crate = "_serde")]
    struct NewType(i32);

    assert_eq!(from_str("value=-2500000"), Ok(p!(NewType(-2_500_000))));
}

#[test]
fn deserialize_extra_ampersands() {
    assert_eq!(from_str("&&value=bar"), Ok(p!("bar")));
    assert_eq!(from_str("value=bar&&"), Ok(p!("bar")));
    assert_eq!(from_str("value=bar&&&value=baz"), Ok(p!("baz")));
}

#[test]
fn deserialize_no_value() {
    assert_eq!(from_str("value"), Ok(p!("")));
    assert_eq!(from_str("value"), Ok(p!(true)));

    assert_eq!(from_str("value="), Ok(p!("")));
    assert_eq!(from_str("value="), Ok(p!(true)));

    assert_eq!(from_str("value"), Ok(p!(None, Option<i32>)));
    assert_eq!(from_str("value="), Ok(p!(None, Option<i32>)));

    // We could see this as an empty string too, but to keep it the same as
    // other types, we go with None
    assert_eq!(from_str("value="), Ok(p!(None, Option<&str>)));
}

#[test]
fn deserialize_integer_overflow() {
    // u8
    assert!(from_str::<Primitive<u8>>("value=-10").is_err());
    assert!(from_str::<Primitive<u8>>("value=260").is_err());

    // i8
    assert!(from_str::<Primitive<i8>>("value=255").is_err());
    assert!(from_str::<Primitive<i8>>("value=-200").is_err());

    // // u16
    assert!(from_str::<Primitive<u16>>("value=65537").is_err());
    assert!(from_str::<Primitive<u16>>("value=-200").is_err());

    // // i16
    assert!(from_str::<Primitive<i16>>("value=32768").is_err());
    assert!(from_str::<Primitive<i16>>("value=-32769").is_err());

    // // u32
    assert!(from_str::<Primitive<u32>>("value=4294967296").is_err());
    assert!(from_str::<Primitive<u32>>("value=-200").is_err());

    // // i32
    assert!(from_str::<Primitive<i32>>("value=2147483648").is_err());
    assert!(from_str::<Primitive<i32>>("value=-2147483649").is_err());

    // // u64
    assert!(from_str::<Primitive<u64>>("value=18446744073709551616").is_err());
    assert!(from_str::<Primitive<u64>>("value=-200").is_err());

    // // i64
    assert!(from_str::<Primitive<i64>>("value=9223372036854775808").is_err());
    assert!(from_str::<Primitive<i64>>("value=-9223372036854775809").is_err());

    // // invalid for integer
    assert!(from_str::<Primitive<i64>>("value=1.5").is_err());
    assert!(from_str::<Primitive<i64>>("value=-1.5").is_err());
    assert!(from_str::<Primitive<i64>>("value=1.2E3").is_err());
    assert!(from_str::<Primitive<i64>>("value=1.2E-3").is_err());
}

#[test]
fn deserialize_invalid_number() {
    assert!(from_str::<Primitive<i64>>("value=number").is_err());
    assert!(from_str::<Primitive<i64>>("value=123n").is_err());

    assert!(from_str::<Primitive<f64>>("value=number").is_err());
    assert!(from_str::<Primitive<f64>>("value=-1.5num").is_err());
    assert!(from_str::<Primitive<f64>>("value=&").is_err());
    assert!(from_str::<Primitive<f64>>("value=1.0a1.0").is_err());
    assert!(from_str::<Primitive<f64>>("value=%2222").is_err());
}

/// Check if different boolean idents work
#[test]
fn deserialize_invalid_bool() {
    assert!(from_str::<Primitive<bool>>("value=bla").is_err());
    assert!(from_str::<Primitive<bool>>("value=0off").is_err());
    assert!(from_str::<Primitive<bool>>("value=of").is_err());
    assert!(from_str::<Primitive<bool>>("value=onoff").is_err());
    assert!(from_str::<Primitive<bool>>("value=on%25").is_err());
}

#[test]
fn deserialize_invalid_type() {
    // Percent encoded values should fail to deserialize for &str
    assert!(
        from_str::<Primitive<&str>>("value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF")
            .is_err()
    );
    assert!(from_str::<Primitive<&str>>("value=rum+rum").is_err());

    // Invalid type for option
    assert!(from_str::<Primitive<Option<u32>>>("value=foo").is_err());
    assert!(from_str::<Primitive<Option<bool>>>("value=foo").is_err());

    // Only struct/map accepted as start point
    assert!(from_str::<String>("value").is_err());
}

#[test]
fn deserialize_invalid_precent_decoding() {
    // If the there is a percent but there aren't 2 characters after it, we ignore them
    assert_eq!(from_str("value=Test%8"), Ok(p!("Test%8")));

    // If the there is a percent but the next 2 characters aren't hex numbers, we ignore them
    assert_eq!(from_str("value=Test%as"), Ok(p!("Test%as")));

    // If we ignored a percent, we may accept the next percent encoded characters within the first one's bound
    assert_eq!(from_str("value=Test%%25"), Ok(p!("Test%%".to_string())));

    // If the provided char is invalid, we throw error
    assert!(from_str::<Primitive<String>>("value=Test%88").is_err());
}

#[test]
fn deserialize_error_test() {
    assert_eq!(
        from_str::<Primitive<String>>("value=Test%88%88")
            .unwrap_err()
            .kind,
        ErrorKind::InvalidEncoding
    );

    assert_eq!(
        from_str::<Primitive<i32>>("value=12foo").unwrap_err().kind,
        ErrorKind::InvalidNumber
    );

    assert_eq!(
        from_str::<Primitive<bool>>("value=foo").unwrap_err().kind,
        ErrorKind::InvalidBoolean
    );
}
