use std::borrow::Cow;
use std::fmt;
use std::ops::{AddAssign, MulAssign, SubAssign};
use std::str;

use atoi::FromRadix10SignedChecked;
use atoi::MaxNumDigits;
use num_traits::{CheckedAdd, CheckedMul, CheckedSub, One, Zero};

use crate::decode::parse_bytes;
use crate::decode::Reference;

use super::{Error, ErrorKind};

pub trait Value<'de> {
    fn parse_int<T>(&self, scratch: &mut Vec<u8>) -> Result<T, Error>
    where
        T: Zero
            + One
            + AddAssign
            + MulAssign
            + SubAssign
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + MaxNumDigits;

    fn parse_float<T>(&self, scratch: &mut Vec<u8>) -> Result<T, Error>
    where
        T: str::FromStr;

    fn parse_bool(&self, scratch: &mut Vec<u8>) -> Result<bool, Error>;

    fn parse_bytes<'s>(self, scratch: &'s mut Vec<u8>) -> Reference<'de, 's, [u8]>;
    fn parse_str<'s>(self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>, Error>;

    fn is_none(&self) -> bool;
}

#[inline]
fn invalid_boolean_error(slice: &[u8]) -> Error {
    Error::new(ErrorKind::InvalidBoolean).value(slice).message(
        "invalid boolean {}, supported values are 1, on and true for true \
        and 0, off and false for false"
            .to_string(),
    )
}

/// Holds a slice of bytes that is already percent decoded
#[derive(Debug)]
pub struct DecodedSlice<'de>(pub Cow<'de, [u8]>);

impl<'de> fmt::Display for DecodedSlice<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.0))
    }
}

impl<'de> Value<'de> for DecodedSlice<'de> {
    fn parse_int<T>(&self, _: &mut Vec<u8>) -> Result<T, Error>
    where
        T: Zero
            + One
            + AddAssign
            + MulAssign
            + SubAssign
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + MaxNumDigits,
    {
        if self.0.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidNumber)
                .value(&self.0)
                .message(format!("invalid index: the key has no value")));
        }

        let (value, len) = T::from_radix_10_signed_checked(&self.0);
        value
            .and_then(|v| if len == self.0.len() { Some(v) } else { None })
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidNumber)
                    .value(&self.0)
                    .message(format!("invalid index: the key has non-numeric characters"))
            })
    }

    fn parse_float<T>(&self, _: &mut Vec<u8>) -> Result<T, Error>
    where
        T: str::FromStr,
    {
        // TODO: Maybe just check is_ascii and use the unsafe version
        str::from_utf8(&self.0)
            .map_err(|_err| {
                Error::new(ErrorKind::InvalidNumber)
                    .value(&self.0)
                    .message("invalid index: the key has invalid characters".to_owned())
            })
            .and_then(|v| {
                v.parse().map_err(|_err| {
                    Error::new(ErrorKind::InvalidNumber)
                        .value(&self.0)
                        .message("invalid index: the key has non-numeric characters".to_owned())
                })
            })
    }

    fn parse_bool(&self, _: &mut Vec<u8>) -> Result<bool, Error> {
        match self.0.len() {
            0 => Ok(true),
            1 => match self.0[0] {
                b'1' => Ok(true),
                b'0' => Ok(false),
                _ => Err(invalid_boolean_error(&self.0)),
            },
            2 if self.0.as_ref() == b"on" => Ok(true),
            3 if self.0.as_ref() == b"off" => Ok(false),
            4 if self.0.as_ref() == b"true" => Ok(true),
            5 if self.0.as_ref() == b"false" => Ok(false),
            _ => Err(invalid_boolean_error(&self.0)),
        }
    }

    fn parse_bytes<'s>(self, _: &'s mut Vec<u8>) -> Reference<'de, 's, [u8]> {
        match self.0 {
            Cow::Borrowed(b) => Reference::Borrowed(b),
            Cow::Owned(o) => Reference::Owned(o),
        }
    }

    fn parse_str<'s>(self, _: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>, Error> {
        let res = match self.0 {
            Cow::Borrowed(b) => str::from_utf8(b)
                .map(Reference::Borrowed)
                .map_err(|e| (e, Reference::Borrowed(b))),
            Cow::Owned(o) => String::from_utf8(o)
                .map(Reference::Owned)
                .map_err(|e| (e.utf8_error(), Reference::Owned(e.into_bytes()))),
        };

        res.map_err(|(error, slice)| {
            Error::new(ErrorKind::InvalidEncoding)
                .message("invalid utf-8 sequence found in the percent decoded value".to_string())
                .value(&slice)
                .index(error.valid_up_to())
        })
    }

    fn is_none(&self) -> bool {
        self.0.is_empty()
    }
}

/// Holds a slice of bytes that is not percent decoded yet
#[derive(Default, Clone, Copy)]
pub struct RawSlice<'de>(pub &'de [u8]);

impl<'de> fmt::Display for RawSlice<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.0))
    }
}

impl<'de> Value<'de> for RawSlice<'de> {
    fn parse_int<T>(&self, _: &mut Vec<u8>) -> Result<T, Error>
    where
        T: Zero
            + One
            + AddAssign
            + MulAssign
            + SubAssign
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + MaxNumDigits,
    {
        if self.0.len() == 0 {
            return Err(Error::new(ErrorKind::InvalidNumber)
                .value(&self.0)
                .message(format!("invalid index: the key has no value")));
        }

        let (value, len) = T::from_radix_10_signed_checked(&self.0);
        value
            .and_then(|v| if len == self.0.len() { Some(v) } else { None })
            .ok_or_else(|| {
                Error::new(ErrorKind::InvalidNumber)
                    .value(self.0)
                    .message(format!("invalid index: the key has non-numeric characters"))
            })
    }

    fn parse_float<T>(&self, _: &mut Vec<u8>) -> Result<T, Error>
    where
        T: str::FromStr,
    {
        // TODO: Maybe just check is_ascii and use the unsafe version
        str::from_utf8(&self.0)
            .map_err(|_err| {
                Error::new(ErrorKind::InvalidNumber)
                    .value(&self.0)
                    .message("invalid index: the key has invalid characters".to_owned())
            })
            .and_then(|v| {
                v.parse().map_err(|_err| {
                    Error::new(ErrorKind::InvalidNumber)
                        .value(&self.0)
                        .message("invalid index: the key has non-numeric characters".to_owned())
                })
            })
    }

    fn parse_bool(&self, _: &mut Vec<u8>) -> Result<bool, Error> {
        match self.0.len() {
            0 => Ok(true),
            1 => match self.0[0] {
                b'1' => Ok(true),
                b'0' => Ok(false),
                _ => Err(invalid_boolean_error(self.0)),
            },
            2 if self.0 == b"on" => Ok(true),
            3 if self.0 == b"off" => Ok(false),
            4 if self.0 == b"true" => Ok(true),
            5 if self.0 == b"false" => Ok(false),
            _ => Err(invalid_boolean_error(self.0)),
        }
    }

    fn parse_bytes<'s>(self, scratch: &'s mut Vec<u8>) -> Reference<'de, 's, [u8]> {
        parse_bytes(self.0, scratch)
    }

    fn parse_str<'s>(self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>, Error> {
        let slice = self.0;

        parse_bytes(slice, scratch)
            .try_map(str::from_utf8)
            .map_err(|error| {
                Error::new(ErrorKind::InvalidEncoding)
                    .message(
                        "invalid utf-8 sequence found in the percent decoded value".to_string(),
                    )
                    .value(slice)
                    .index(error.valid_up_to())
            })
    }

    fn is_none(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'de> Value<'de> for Option<RawSlice<'de>> {
    fn parse_int<T>(&self, scratch: &mut Vec<u8>) -> Result<T, Error>
    where
        T: Zero
            + One
            + AddAssign
            + MulAssign
            + SubAssign
            + CheckedAdd
            + CheckedSub
            + CheckedMul
            + MaxNumDigits,
    {
        self.unwrap_or_default().parse_int(scratch)
    }

    fn parse_float<T>(&self, scratch: &mut Vec<u8>) -> Result<T, Error>
    where
        T: str::FromStr,
    {
        self.unwrap_or_default().parse_float(scratch)
    }

    fn parse_bool(&self, scratch: &mut Vec<u8>) -> Result<bool, Error> {
        self.unwrap_or_default().parse_bool(scratch)
    }

    fn parse_bytes<'s>(self, scratch: &'s mut Vec<u8>) -> Reference<'de, 's, [u8]> {
        self.unwrap_or_default().parse_bytes(scratch)
    }

    fn parse_str<'s>(self, scratch: &'s mut Vec<u8>) -> Result<Reference<'de, 's, str>, Error> {
        self.unwrap_or_default().parse_str(scratch)
    }

    fn is_none(&self) -> bool {
        self.is_none()
    }
}
