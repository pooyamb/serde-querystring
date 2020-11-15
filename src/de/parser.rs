use std::str;

use super::basis::{ReaderNumber, Reference};
use super::constants;
use crate::error::{Error, Result};

macro_rules! overflow {
    ($a:ident * 10 + $b:ident, $c:expr) => {
        $a >= $c / 10 && ($a > $c / 10 || $b > $c % 10)
    };
}

#[inline]
pub(crate) fn parse_char(high: u8, low: u8) -> Result<u8> {
    let high = char::from(high)
        .to_digit(16)
        .ok_or(Error::InvalidCharacter)?;
    let low = char::from(low)
        .to_digit(16)
        .ok_or(Error::InvalidCharacter)?;
    Ok(high as u8 * 0x10 + low as u8)
}

pub struct Parser<'de> {
    pub(crate) slice: &'de [u8],
    pub(crate) index: usize,
}

impl<'de> Parser<'de> {
    pub(crate) fn new(slice: &'de [u8]) -> Self {
        Parser { slice, index: 0 }
    }

    #[inline]
    pub(crate) fn next(&mut self) -> Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            let ch = self.slice[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        })
    }

    #[inline]
    pub(crate) fn peek(&mut self) -> Result<Option<u8>> {
        Ok(if self.index < self.slice.len() {
            Some(self.slice[self.index])
        } else {
            None
        })
    }

    #[inline]
    pub(crate) fn discard(&mut self) {
        self.index += 1;
    }

    // Return what you see before the next delimiter
    pub(crate) fn parse_token(&mut self) -> Result<&[u8]> {
        if self.done() {
            return Err(Error::EofReached);
        }

        let start = self.index;

        while self.index < self.slice.len() {
            match self.slice[self.index] {
                b'&' | b';' => {
                    break;
                }
                _ => {
                    self.index += 1;
                }
            }
        }

        Ok(&self.slice[start..self.index])
    }

    pub(crate) fn parse_str_bytes<'s, T, F>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
        result: F,
    ) -> Result<Reference<'de, 's, T>>
    where
        T: ?Sized + 's,
        F: for<'f> FnOnce(&'s Self, &'f [u8]) -> Result<&'f T>,
    {
        // Index of the first byte not yet copied into the scratch space.
        let mut start = self.index;

        scratch.clear();

        while self.index < self.slice.len() {
            match self.slice[self.index] {
                b'&' | b';' => {
                    break;
                }
                b'+' => {
                    scratch.extend_from_slice(&self.slice[start..self.index]);
                    scratch.push(b' ');
                    self.index += 1;
                    start = self.index;
                }
                b'%' => {
                    // we saw percentage
                    scratch.extend_from_slice(&self.slice[start..self.index]);

                    if self.slice.len() < self.index + 2 {
                        return Err(Error::EofReached);
                    }

                    scratch.push(parse_char(
                        self.slice[(self.index + 1)],
                        self.slice[(self.index + 2)],
                    )?);

                    self.index += 3;
                    start = self.index;
                }
                _ => {
                    self.index += 1;
                }
            }
        }

        if scratch.is_empty() {
            let borrowed = &self.slice[start..self.index];
            result(self, borrowed).map(Reference::Borrowed)
        } else {
            scratch.extend_from_slice(&self.slice[start..self.index]);
            result(self, scratch).map(Reference::Copied)
        }
    }

    pub(crate) fn parse_str<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, str>> {
        self.parse_str_bytes(scratch, |_, bytes| {
            str::from_utf8(bytes).map_err(|_| Error::InvalidString)
        })
    }

    #[cold]
    #[inline(never)]
    fn parse_decimal_overflow(
        &mut self,
        positive: bool,
        significand: u64,
        exponent: i32,
    ) -> Result<f64> {
        // The next multiply/add would overflow, so just ignore all further
        // digits.
        while let Some(b'0'..=b'9') = self.peek()? {
            self.discard();
        }

        match self.peek()? {
            Some(b'e') | Some(b'E') => self.parse_exponent(positive, significand, exponent),
            _ => self.f64_from_parts(positive, significand, exponent),
        }
    }

    // This cold code should not be inlined into the middle of the hot
    // exponent-parsing loop above.
    #[cold]
    #[inline(never)]
    fn parse_exponent_overflow(
        &mut self,
        positive: bool,
        zero_significand: bool,
        positive_exp: bool,
    ) -> Result<f64> {
        // Error instead of +/- infinity.
        if !zero_significand && positive_exp {
            return Err(Error::InvalidNumber);
        }

        while let Some(b'0'..=b'9') = self.peek()? {
            self.discard();
        }
        Ok(if positive { 0.0 } else { -0.0 })
    }

    fn parse_decimal(
        &mut self,
        positive: bool,
        mut significand: u64,
        mut exponent: i32,
    ) -> Result<f64> {
        self.discard();

        while let Some(c @ b'0'..=b'9') = self.peek()? {
            let digit = (c - b'0') as u64;

            if overflow!(significand * 10 + digit, u64::max_value()) {
                return self.parse_decimal_overflow(positive, significand, exponent);
            }

            self.discard();
            significand = significand * 10 + digit;
            exponent -= 1;
        }

        // Error if there is not at least one digit after the decimal point.
        if exponent == 0 {
            match self.peek()? {
                Some(_) => return Err(Error::InvalidNumber),
                None => return Err(Error::InvalidNumber),
            }
        }

        match self.peek()? {
            Some(b'e') | Some(b'E') => self.parse_exponent(positive, significand, exponent),
            _ => self.f64_from_parts(positive, significand, exponent),
        }
    }

    fn parse_exponent(
        &mut self,
        positive: bool,
        significand: u64,
        starting_exp: i32,
    ) -> Result<f64> {
        self.discard();

        let positive_exp = match self.peek()? {
            Some(b'+') => {
                self.discard();
                true
            }
            Some(b'-') => {
                self.discard();
                false
            }
            _ => true,
        };

        let next = match self.next()? {
            Some(b) => b,
            None => {
                return Err(Error::EofReached);
            }
        };

        // Make sure a digit follows the exponent place.
        let mut exp = match next {
            c @ b'0'..=b'9' => (c - b'0') as i32,
            _ => {
                return Err(Error::InvalidNumber);
            }
        };

        while let Some(c @ b'0'..=b'9') = self.peek()? {
            self.discard();
            let digit = (c - b'0') as i32;

            if overflow!(exp * 10 + digit, i32::max_value()) {
                let zero_significand = significand == 0;
                return self.parse_exponent_overflow(positive, zero_significand, positive_exp);
            }

            exp = exp * 10 + digit;
        }

        let final_exp = if positive_exp {
            starting_exp.saturating_add(exp)
        } else {
            starting_exp.saturating_sub(exp)
        };

        self.f64_from_parts(positive, significand, final_exp)
    }

    fn f64_from_parts(
        &mut self,
        positive: bool,
        significand: u64,
        mut exponent: i32,
    ) -> Result<f64> {
        let mut f = significand as f64;
        loop {
            match constants::POW10.get(exponent.wrapping_abs() as usize) {
                Some(&pow) => {
                    if exponent >= 0 {
                        f *= pow;
                        if f.is_infinite() {
                            return Err(Error::InvalidNumber);
                        }
                    } else {
                        f /= pow;
                    }
                    break;
                }
                None => {
                    if f == 0.0 {
                        break;
                    }
                    if exponent >= 0 {
                        return Err(Error::InvalidNumber);
                    }
                    f /= 1e308;
                    exponent += 308;
                }
            }
        }
        Ok(if positive { f } else { -f })
    }

    #[cold]
    #[inline(never)]
    fn parse_long_integer(&mut self, positive: bool, significand: u64) -> Result<f64> {
        let mut exponent = 0;
        loop {
            match self.peek()? {
                Some(b'0'..=b'9') => {
                    self.discard();
                    // This could overflow... if your integer is gigabytes long.
                    // Ignore that possibility.
                    exponent += 1;
                }
                Some(b'.') => {
                    return self.parse_decimal(positive, significand, exponent);
                }
                Some(b'e') | Some(b'E') => {
                    return self.parse_exponent(positive, significand, exponent);
                }
                _ => {
                    return self.f64_from_parts(positive, significand, exponent);
                }
            }
        }
    }

    pub(crate) fn parse_integer(&mut self, positive: bool) -> Result<ReaderNumber> {
        let next = match self.next()? {
            Some(b) => b,
            None => {
                return Err(Error::EofReached);
            }
        };

        match next {
            c @ b'0'..=b'9' => {
                let mut significand = (c - b'0') as u64;

                loop {
                    match self.peek()? {
                        Some(c @ b'0'..=b'9') => {
                            let digit = (c - b'0') as u64;

                            // We need to be careful with overflow. If we can,
                            // try to keep the number as a `u64` until we grow
                            // too large. At that point, switch to parsing the
                            // value as a `f64`.
                            if overflow!(significand * 10 + digit, u64::max_value()) {
                                return Ok(ReaderNumber::F64(
                                    self.parse_long_integer(positive, significand)?,
                                ));
                            }

                            self.discard();
                            significand = significand * 10 + digit;
                        }
                        Some(b'.') => {
                            return self.parse_number(positive, significand);
                        }
                        _ => {
                            let res = if positive {
                                ReaderNumber::U64(significand)
                            } else {
                                let neg = (significand as i64).wrapping_neg();

                                // Convert into a float if we underflow.
                                if neg > 0 {
                                    ReaderNumber::F64(-(significand as f64))
                                } else {
                                    ReaderNumber::I64(neg)
                                }
                            };
                            return Ok(res);
                        }
                    }
                }
            }
            _ => Err(Error::InvalidNumber),
        }
    }

    fn parse_number(&mut self, positive: bool, significand: u64) -> Result<ReaderNumber> {
        Ok(match self.peek()? {
            Some(b'.') => ReaderNumber::F64(self.parse_decimal(positive, significand, 0)?),
            Some(b'e') | Some(b'E') => {
                ReaderNumber::F64(self.parse_exponent(positive, significand, 0)?)
            }
            _ => {
                if positive {
                    ReaderNumber::U64(significand)
                } else {
                    let neg = (significand as i64).wrapping_neg();

                    // Convert into a float if we underflow.
                    if neg > 0 {
                        ReaderNumber::F64(-(significand as f64))
                    } else {
                        ReaderNumber::I64(neg)
                    }
                }
            }
        })
    }

    pub(crate) fn parse_ident(&mut self, ident: &[u8]) -> Result<()> {
        for expected in ident {
            match self.next()? {
                None => {
                    return Err(Error::EofReached);
                }
                Some(next) => {
                    if next != *expected {
                        return Err(Error::InvalidIdent);
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn parse_bool(&mut self) -> Result<bool> {
        match self.peek()? {
            Some(b't') => {
                self.discard();
                self.parse_ident(b"rue")?;
                Ok(true)
            }
            Some(b'f') => {
                self.discard();
                self.parse_ident(b"alse")?;
                Ok(false)
            }
            Some(b'o') => {
                self.discard();
                match self.next()? {
                    Some(b'n') => Ok(true),
                    Some(b'f') => {
                        self.parse_ident(b"f")?;
                        Ok(false)
                    }
                    _ => Err(Error::InvalidIdent),
                }
            }
            Some(b'0') => {
                self.discard();
                Ok(false)
            }
            Some(b'1') => {
                self.discard();
                Ok(true)
            }
            Some(_) => Err(Error::InvalidIdent),
            None => Err(Error::EofReached),
        }
    }

    pub(crate) fn done(&self) -> bool {
        self.index >= self.slice.len()
    }
}
