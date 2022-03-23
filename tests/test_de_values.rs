//! These tests are common between different deserialization methods

use serde::Deserialize;
use serde_querystring::{from_bytes, Error};

fn from_str<'de, T: Deserialize<'de>>(input: &'de str) -> Result<T, Error> {
    from_bytes(input.as_bytes(), serde_querystring::Config::Simple)
}

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
fn deserialize_integer_valid() {
    // u8
    assert_eq!(from_str("value=255"), Ok(p!(255_u8)));
    assert_eq!(from_str("value=0"), Ok(p!(0_u8)));

    // i8
    assert_eq!(from_str("value=127"), Ok(p!(127_i8)));
    assert_eq!(from_str("value=-128"), Ok(p!(-128_i8)));

    // u16
    assert_eq!(from_str("value=65535"), Ok(p!(65535_u16)));
    assert_eq!(from_str("value=0"), Ok(p!(0_u16)));

    // i16
    assert_eq!(from_str("value=32767"), Ok(p!(32767_i16)));
    assert_eq!(from_str("value=-32768"), Ok(p!(-32768_i16)));

    // u32
    assert_eq!(from_str("value=4294967295"), Ok(p!(4294967295_u32)));
    assert_eq!(from_str("value=0"), Ok(p!(0_u32)));

    // i32
    assert_eq!(from_str("value=2147483647"), Ok(p!(2147483647_i32)));
    assert_eq!(from_str("value=-2147483648"), Ok(p!(-2147483648_i32)));

    // u64
    assert_eq!(
        from_str("value=18446744073709551615"),
        Ok(p!(18446744073709551615_u64))
    );
    assert_eq!(from_str("value=0"), Ok(p!(0_u64)));

    // i64
    assert_eq!(
        from_str("value=9223372036854775807"),
        Ok(p!(9223372036854775807_i64))
    );
    assert_eq!(
        from_str("value=-9223372036854775808"),
        Ok(p!(-9223372036854775808_i64))
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
        Ok(p!(18446744073709551616_f64))
    );
    assert_eq!(
        from_str("value=-18446744073709551616"),
        Ok(p!(-18446744073709551616_f64))
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
}

/// Check if we can directly deserialize non percent encoded values to str
#[test]
fn deserialize_str() {
    assert_eq!(from_str("value=test"), Ok(p!("test")));

    // We don't make assumptions about numbers
    assert_eq!(from_str("value=250"), Ok(p!("250")));
    assert_eq!(from_str("value=-25"), Ok(p!("-25")));
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
}

/// Check if different boolean idents work
#[test]
fn deserialize_invalid_bool() {
    assert!(from_str::<Primitive<bool>>("value=bla").is_err());
    assert!(from_str::<Primitive<bool>>("value=0off").is_err());
    assert!(from_str::<Primitive<bool>>("value=of").is_err());
    assert!(from_str::<Primitive<bool>>("value=onoff").is_err());
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
    assert!(from_str::<Primitive<String>>("value=Test%88").is_err());
}

// #[test]
// fn deserialize_error_test() {
//     assert_eq!(
//         from_str::<Primitive<(i32, i32)>>("value=12&value=13&value=14")
//             .unwrap_err()
//             .kind,
//         ErrorKind::InvalidLength
//     );

//     #[derive(Debug, Deserialize)]
//     struct Tuple(i32, i32);
//     assert_eq!(
//         from_str::<Primitive<Tuple>>("value=12&value=13&value=14")
//             .unwrap_err()
//             .kind,
//         ErrorKind::InvalidLength
//     );

//     assert_eq!(
//         from_str::<Primitive<String>>("value=Test%88%88")
//             .unwrap_err()
//             .kind,
//         ErrorKind::InvalidEncoding
//     );

//     assert_eq!(
//         from_str::<Primitive<i32>>("value=12foo").unwrap_err().kind,
//         ErrorKind::InvalidNumber
//     );

//     assert_eq!(
//         from_str::<Primitive<bool>>("value=foo").unwrap_err().kind,
//         ErrorKind::InvalidBoolean
//     );

//     #[derive(Debug, Deserialize)]
//     enum ValueEnum {
//         A(i32, i32),
//         B(i32),
//         C {},
//     }

//     assert_eq!(
//         from_str::<Primitive<ValueEnum>>("value=A")
//             .unwrap_err()
//             .kind,
//         ErrorKind::UnexpectedType
//     );
//     assert_eq!(
//         from_str::<Primitive<ValueEnum>>("value=B")
//             .unwrap_err()
//             .kind,
//         ErrorKind::UnexpectedType
//     );
//     assert_eq!(
//         from_str::<Primitive<ValueEnum>>("value=C")
//             .unwrap_err()
//             .kind,
//         ErrorKind::UnexpectedType
//     );
// }
