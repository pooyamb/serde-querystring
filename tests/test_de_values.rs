//! These tests are common between different deserialization methods

use _serde::Deserialize;
use serde_querystring::de::{from_bytes, from_str, ErrorKind, ParseMode};

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

macro_rules! map {
    () => {{
        std::collections::HashMap::new()
    }};
    ($($k:expr => $v:expr),+ $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(map.insert($k, $v);)+
        map
    }};
}

fn check_result<F, R>(f: F, r: R)
where
    F: Fn(ParseMode) -> R,
    R: PartialEq + std::fmt::Debug,
{
    assert_eq!(f(ParseMode::UrlEncoded), r);
    assert_eq!(f(ParseMode::Duplicate), r);
    assert_eq!(f(ParseMode::Delimiter(b'|')), r);
    assert_eq!(f(ParseMode::Brackets), r);
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
    check_result(|mode| from_str("value=255", mode), Ok(p!(u8::MAX)));
    // check_result(|mode| from_str("value=255", mode), Ok(p!(u8::MIN)));
    check_result(|mode| from_str("value=0", mode), Ok(p!(u8::MIN)));

    // i8
    check_result(|mode| from_str("value=127", mode), Ok(p!(i8::MAX)));
    check_result(|mode| from_str("value=-128", mode), Ok(p!(i8::MIN)));

    // u16
    check_result(|mode| from_str("value=65535", mode), Ok(p!(u16::MAX)));
    check_result(|mode| from_str("value=0", mode), Ok(p!(u16::MIN)));

    // i16
    check_result(|mode| from_str("value=32767", mode), Ok(p!(i16::MAX)));
    check_result(|mode| from_str("value=-32768", mode), Ok(p!(i16::MIN)));

    // u32
    check_result(|mode| from_str("value=4294967295", mode), Ok(p!(u32::MAX)));
    check_result(|mode| from_str("value=0", mode), Ok(p!(u32::MIN)));

    // i32
    check_result(|mode| from_str("value=2147483647", mode), Ok(p!(i32::MAX)));
    check_result(|mode| from_str("value=-2147483648", mode), Ok(p!(i32::MIN)));

    // u64
    check_result(
        |mode| from_str("value=18446744073709551615", mode),
        Ok(p!(u64::MAX)),
    );
    check_result(|mode| from_str("value=0", mode), Ok(p!(u64::MIN)));

    // i64
    check_result(
        |mode| from_str("value=9223372036854775807", mode),
        Ok(p!(i64::MAX)),
    );
    check_result(
        |mode| from_str("value=-9223372036854775808", mode),
        Ok(p!(i64::MIN)),
    );

    // In keys
    let map = map! {
        -1337_i64 => "value1",
        -7331_i64 => "value2",
        1337_i64 => "value3",
        7331_i64 => "value4"
    };
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
    check_result(|mode| from_str("value=1.2", mode), Ok(p!(1.2_f64)));
    check_result(|mode| from_str("value=-1.2", mode), Ok(p!(-1.2_f64)));
    check_result(|mode| from_str("value=1.2E5", mode), Ok(p!(1.2E5_f64)));
    check_result(|mode| from_str("value=-1.2E5", mode), Ok(p!(-1.2E5_f64)));
    check_result(|mode| from_str("value=1.2E-5", mode), Ok(p!(1.2E-5_f64)));
    check_result(|mode| from_str("value=-1.2E-5", mode), Ok(p!(-1.2E-5_f64)));
    check_result(
        |mode| from_str("value=18446744073709551616", mode),
        Ok(p!(18_446_744_073_709_551_616_f64)),
    );
    check_result(
        |mode| from_str("value=-18446744073709551616", mode),
        Ok(p!(-18_446_744_073_709_551_616_f64)),
    );
}

/// Check if different boolean idents work
#[test]
fn deserialize_bool() {
    // true
    check_result(|mode| from_str("value=1", mode), Ok(p!(true)));
    check_result(|mode| from_str("value=on", mode), Ok(p!(true)));
    check_result(|mode| from_str("value=true", mode), Ok(p!(true)));

    // false
    check_result(|mode| from_str("value=0", mode), Ok(p!(false)));
    check_result(|mode| from_str("value=off", mode), Ok(p!(false)));
    check_result(|mode| from_str("value=false", mode), Ok(p!(false)));

    // In keys
    let map = map! {
        true => "value1",
        false => "value2"
    };
    assert_eq!(
        from_bytes(b"true=value1&off=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

/// Check if we can directly deserialize non percent encoded values to str
#[test]
fn deserialize_str() {
    check_result(|mode| from_str("value=test", mode), Ok(p!("test")));

    // We don't make assumptions about numbers
    check_result(|mode| from_str("value=250", mode), Ok(p!("250")));
    check_result(|mode| from_str("value=-25", mode), Ok(p!("-25")));

    // In keys
    let map = map! {
        "some" => "value1",
        "bytes" => "value2"
    };
    assert_eq!(
        from_bytes(b"some=value1&bytes=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_strings() {
    check_result(
        |mode| from_str("value=foo", mode),
        Ok(p!("foo".to_string())),
    );

    // percent decoded
    check_result(
        |mode| {
            from_str(
                "value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF",
                mode,
            )
        },
        Ok(p!("بابابزرگ".to_string())),
    );

    // Plus in strings should be replaced with space
    check_result(
        |mode| from_str("value=rum+rum", mode),
        Ok(p!("rum rum".to_string())),
    );

    // In keys
    let map = map! {
        String::from("some") => "value1",
        String::from("st ri ng") => "value2"
    };
    assert_eq!(
        from_bytes(b"some=value1&st+ri+ng=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_bytes() {
    use serde_bytes::Bytes;

    check_result(
        |mode| from_str("value=test", mode),
        Ok(p!(Bytes::new(b"test"))),
    );

    // We don't make assumptions about numbers
    check_result(
        |mode| from_str("value=250", mode),
        Ok(p!(Bytes::new(b"250"))),
    );
    check_result(
        |mode| from_str("value=-25", mode),
        Ok(p!(Bytes::new(b"-25"))),
    );

    // In keys
    let map = map! {
        Bytes::new(b"some") => "value1",
        Bytes::new(b"bytes") => "value2"
    };
    assert_eq!(
        from_bytes(b"some=value1&bytes=value2", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_byte_vecs() {
    use serde_bytes::ByteBuf;

    check_result(
        |mode| from_str("value=foo", mode),
        Ok(p!(ByteBuf::from("foo"))),
    );

    // percent decoded
    check_result(
        |mode| {
            from_str(
                "value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF",
                mode,
            )
        },
        Ok(p!(ByteBuf::from("بابابزرگ"))),
    );

    // Plus in strings should be replaced with space
    check_result(
        |mode| from_str("value=rum+rum", mode),
        Ok(p!(ByteBuf::from("rum rum"))),
    );

    // In keys
    let map = map! {
        ByteBuf::from("some") => "value1",
        ByteBuf::from("by\0te s") => "value2"
    };
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
    let map = map! {
        Side::God => "winner",
        Side::Right => "looser"
    };
    assert_eq!(
        from_bytes(b"God=winner&Right=looser", ParseMode::UrlEncoded),
        Ok(map)
    );
}

#[test]
fn deserialize_option() {
    check_result(
        |mode| from_str("value=1337", mode),
        Ok(p!(Some(1337), Option<u32>)),
    );

    check_result(
        |mode| from_str::<Primitive<Option<u32>>>("value=", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<Option<u32>>>("value=1337&value=", mode).is_err(),
        true,
    );
}

#[test]
fn deserialize_new_type() {
    #[derive(Debug, Deserialize, Eq, PartialEq)]
    #[serde(crate = "_serde")]
    struct NewType(i32);

    check_result(
        |mode| from_str("value=-2500000", mode),
        Ok(p!(NewType(-2_500_000))),
    );
}

#[test]
fn deserialize_extra_ampersands() {
    check_result(|mode| from_str("&&value=bar", mode), Ok(p!("bar")));
    check_result(|mode| from_str("value=bar&&", mode), Ok(p!("bar")));
    check_result(
        |mode| from_str("value=bar&&&value=baz", mode),
        Ok(p!("baz")),
    );
}

#[test]
fn deserialize_no_value() {
    check_result(|mode| from_str("value", mode), Ok(p!("")));
    check_result(|mode| from_str("value", mode), Ok(p!(true)));

    check_result(|mode| from_str("value=", mode), Ok(p!("")));
    check_result(|mode| from_str("value=", mode), Ok(p!(true)));

    // We could see this as an empty string too, but to keep it the same as
    // other types, we go with None
    check_result(
        |mode| from_str("value=", mode),
        Ok(p!(Some(""), Option<&str>)),
    );
}

#[test]
fn deserialize_integer_overflow() {
    // u8
    check_result(
        |mode| from_str::<Primitive<u8>>("value=-10", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<u8>>("value=260", mode).is_err(),
        true,
    );

    // i8
    check_result(
        |mode| from_str::<Primitive<i8>>("value=255", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i8>>("value=-200", mode).is_err(),
        true,
    );

    // u16
    check_result(
        |mode| from_str::<Primitive<u16>>("value=65537", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<u16>>("value=-200", mode).is_err(),
        true,
    );

    // i16
    check_result(
        |mode| from_str::<Primitive<i16>>("value=32768", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i16>>("value=-32769", mode).is_err(),
        true,
    );

    // u32
    check_result(
        |mode| from_str::<Primitive<u32>>("value=4294967296", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<u32>>("value=-200", mode).is_err(),
        true,
    );

    // i32
    check_result(
        |mode| from_str::<Primitive<i32>>("value=2147483648", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i32>>("value=-2147483649", mode).is_err(),
        true,
    );

    // u64
    check_result(
        |mode| from_str::<Primitive<u64>>("value=18446744073709551616", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<u64>>("value=-200", mode).is_err(),
        true,
    );

    // i64
    check_result(
        |mode| from_str::<Primitive<i64>>("value=9223372036854775808", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i64>>("value=-9223372036854775809", mode).is_err(),
        true,
    );

    // invalid for integer
    check_result(
        |mode| from_str::<Primitive<i64>>("value=1.5", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i64>>("value=-1.5", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i64>>("value=1.2E3", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i64>>("value=1.2E-3", mode).is_err(),
        true,
    );
}

#[test]
fn deserialize_invalid_number() {
    check_result(
        |mode| from_str::<Primitive<i64>>("value=number", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<i64>>("value=123n", mode).is_err(),
        true,
    );

    check_result(
        |mode| from_str::<Primitive<f64>>("value=number", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<f64>>("value=-1.5num", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<f64>>("value=&", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<f64>>("value=1.0a1.0", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<f64>>("value=%2222", mode).is_err(),
        true,
    );
}

/// Check if different boolean idents work
#[test]
fn deserialize_invalid_bool() {
    check_result(
        |mode| from_str::<Primitive<bool>>("value=bla", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<bool>>("value=0off", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<bool>>("value=of", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<bool>>("value=onoff", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<bool>>("value=on%25", mode).is_err(),
        true,
    );
}

#[test]
fn deserialize_invalid_type() {
    // Percent encoded values should fail to deserialize for &str
    check_result(
        |mode| {
            from_str::<Primitive<&str>>(
                "value=%D8%A8%D8%A7%D8%A8%D8%A7%D8%A8%D8%B2%D8%B1%DA%AF",
                mode,
            )
            .is_err()
        },
        true,
    );
    check_result(
        |mode| from_str::<Primitive<&str>>("value=rum+rum", mode).is_err(),
        true,
    );

    // Invalid type for option
    check_result(
        |mode| from_str::<Primitive<Option<u32>>>("value=foo", mode).is_err(),
        true,
    );
    check_result(
        |mode| from_str::<Primitive<Option<bool>>>("value=foo", mode).is_err(),
        true,
    );

    // Only struct/map accepted as start point
    check_result(|mode| from_str::<String>("value", mode).is_err(), true);
}

#[test]
fn deserialize_invalid_precent_decoding() {
    // If the there is a percent but there aren't 2 characters after it, we ignore them
    check_result(|mode| from_str("value=Test%8", mode), Ok(p!("Test%8")));

    // If the there is a percent but the next 2 characters aren't hex numbers, we ignore them
    check_result(|mode| from_str("value=Test%as", mode), Ok(p!("Test%as")));

    // If we ignored a percent, we may accept the next percent encoded characters within the first one's bound
    check_result(
        |mode| from_str("value=Test%%25", mode),
        Ok(p!("Test%%".to_string())),
    );

    // If the provided char is invalid, we throw error
    check_result(
        |mode| from_str::<Primitive<String>>("value=Test%88", mode).is_err(),
        true,
    );
}

#[test]
fn deserialize_error_test() {
    check_result(
        |mode| {
            from_str::<Primitive<String>>("value=Test%88%88", mode)
                .unwrap_err()
                .kind
        },
        ErrorKind::InvalidEncoding,
    );

    check_result(
        |mode| {
            from_str::<Primitive<i32>>("value=12foo", mode)
                .unwrap_err()
                .kind
        },
        ErrorKind::InvalidNumber,
    );

    check_result(
        |mode| {
            from_str::<Primitive<bool>>("value=foo", mode)
                .unwrap_err()
                .kind
        },
        ErrorKind::InvalidBoolean,
    );
}
