use std::ops;
use std::str;

use crate::error::{Error, Result};
use serde::de;

macro_rules! overflow {
    ($a:ident * 10 + $b:ident, $c:expr) => {
        $a >= $c / 10 && ($a > $c / 10 || $b > $c % 10)
    };
}

#[inline]
pub(crate) fn parse_char(bytes: &[u8]) -> Option<u8> {
    let high = char::from(bytes[0]).to_digit(16)?;
    let low = char::from(bytes[1]).to_digit(16)?;
    Some(high as u8 * 0x10 + low as u8)
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
                    if self.slice.len() > self.index + 2 {
                        match parse_char(&self.slice[(self.index + 1)..=(self.index + 2)]) {
                            Some(b) => {
                                scratch.extend_from_slice(&self.slice[start..self.index]);
                                scratch.push(b);
                                self.index += 3;
                                start = self.index;
                            }
                            None => {
                                // If it wasn't valid, just add the bytes as they were
                                self.index += 3;
                            }
                        }
                    } else {
                        self.index += 1;
                    }
                }
                _ => {
                    self.index += 1;
                }
            }
        }

        if scratch.is_empty() {
            let borrowed = &self.slice[start..self.index];
            self.discard();
            result(self, borrowed).map(Reference::Borrowed)
        } else {
            scratch.extend_from_slice(&self.slice[start..self.index]);
            self.discard();
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

    pub(crate) fn parse_bytes<'s>(
        &'s mut self,
        scratch: &'s mut Vec<u8>,
    ) -> Result<Reference<'de, 's, [u8]>> {
        self.parse_str_bytes(scratch, |_, bytes| Ok(bytes))
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
            match POW10.get(exponent.wrapping_abs() as usize) {
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

    pub(crate) fn parse_ignore(&mut self) -> Result<()> {
        while let Some(b) = self.next()? {
            match b {
                b'&' | b';' => {
                    self.discard();
                    break;
                }
                _ => {}
            }
        }
        Ok(())
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

        if let Some(b';') | Some(b'&') = self.peek()? {
            self.discard()
        }

        Ok(())
    }

    pub(crate) fn parse_bool(&mut self) -> Result<bool> {
        match self.next()? {
            Some(b't') => {
                self.parse_ident(b"rue")?;
                Ok(true)
            }
            Some(b'f') => {
                self.parse_ident(b"alse")?;
                Ok(false)
            }
            Some(b'o') => match self.peek()? {
                Some(b'n') => {
                    self.parse_ident(b"n")?;
                    Ok(true)
                }
                Some(b'f') => {
                    self.parse_ident(b"ff")?;
                    Ok(false)
                }
                _ => Err(Error::EofReached),
            },
            Some(b'0') => {
                if let Some(b';') | Some(b'&') = self.peek()? {
                    self.discard()
                }
                Ok(false)
            }
            Some(b'1') => {
                if let Some(b';') | Some(b'&') = self.peek()? {
                    self.discard()
                }
                Ok(true)
            }
            Some(_) => Err(Error::InvalidIdent),
            None => Err(Error::EofReached),
        }
    }

    fn parse_subpair(&mut self, key: &'de [u8], stash: &mut super::Stash<'de>) -> Result<()> {
        if key.is_empty() {
            // empty keys are not allowed at root level
            return Err(Error::InvalidMapKey);
        }

        let mut key_index = self.index;
        while key_index < self.slice.len() {
            match self.slice[key_index] {
                b'=' => {
                    break;
                }
                _ => {
                    key_index += 1;
                }
            }
        }

        if key_index == self.slice.len() {
            // We didn't see equal sign
            return Err(Error::InvalidMapKey);
        }

        let mut value_index = key_index + 1;
        while value_index < self.slice.len() {
            match self.slice[value_index] {
                b';' | b'&' => {
                    break;
                }
                _ => {
                    value_index += 1;
                }
            }
        }

        stash.add(
            &key,
            &self.slice[self.index..key_index],
            &self.slice[(key_index + 1)..value_index],
        );

        self.index = value_index + 1;
        Ok(())
    }

    pub(crate) fn parse_key(&mut self, stash: &mut super::Stash<'de>) -> Result<Option<&'de [u8]>> {
        // Parse key
        let mut key_found = false;
        let mut key_index = self.index;
        while key_index < self.slice.len() {
            match self.slice[key_index] {
                b'=' | b'&' | b';' => {
                    key_found = true;
                    break;
                }
                b'[' => {
                    // It's a subkey
                    let key = &self.slice[self.index..key_index];
                    self.index = key_index + 1;
                    self.parse_subpair(&key, stash)?;
                    key_index = self.index;
                }
                b'%' => {
                    if self.slice.len() > key_index + 2 {
                        match parse_char(&self.slice[(key_index + 1)..=(key_index + 2)]) {
                            Some(b'[') => {
                                // let key = &self.slice[self.index..(key_index - 1)];
                                let key = &self.slice[self.index..key_index];
                                self.index = key_index + 3;
                                self.parse_subpair(&key, stash)?;
                                key_index = self.index;
                            }
                            _ => {
                                // If it wasn't valid, just add the bytes as they were
                                key_index += 2;
                            }
                        }
                    } else {
                        key_index += 1;
                    }
                }
                _ => {
                    key_index += 1;
                }
            }
        }

        if !key_found {
            return Ok(None);
        }
        let key = &self.slice[self.index..key_index];

        if key.is_empty() {
            // Empty keys are not allowed at root level
            return Err(Error::InvalidMapKey);
        }

        self.index = key_index + 1;

        Ok(Some(key))
    }

    pub(crate) fn parse_one_seq_value(&mut self) -> Result<Option<&'de [u8]>> {
        if self.done() {
            return Ok(None);
        }

        let start_index = self.index;

        while self.index < self.slice.len() {
            match self.slice[self.index] {
                b'&' | b';' | b',' => break,
                _ => {
                    self.index += 1;
                }
            }
        }

        let slice = &self.slice[start_index..self.index];
        Ok(Some(slice))
    }

    pub(crate) fn done(&self) -> bool {
        self.index >= self.slice.len()
    }
}

pub static POW10: [f64; 309] = [
    1e000, 1e001, 1e002, 1e003, 1e004, 1e005, 1e006, 1e007, 1e008, 1e009, //
    1e010, 1e011, 1e012, 1e013, 1e014, 1e015, 1e016, 1e017, 1e018, 1e019, //
    1e020, 1e021, 1e022, 1e023, 1e024, 1e025, 1e026, 1e027, 1e028, 1e029, //
    1e030, 1e031, 1e032, 1e033, 1e034, 1e035, 1e036, 1e037, 1e038, 1e039, //
    1e040, 1e041, 1e042, 1e043, 1e044, 1e045, 1e046, 1e047, 1e048, 1e049, //
    1e050, 1e051, 1e052, 1e053, 1e054, 1e055, 1e056, 1e057, 1e058, 1e059, //
    1e060, 1e061, 1e062, 1e063, 1e064, 1e065, 1e066, 1e067, 1e068, 1e069, //
    1e070, 1e071, 1e072, 1e073, 1e074, 1e075, 1e076, 1e077, 1e078, 1e079, //
    1e080, 1e081, 1e082, 1e083, 1e084, 1e085, 1e086, 1e087, 1e088, 1e089, //
    1e090, 1e091, 1e092, 1e093, 1e094, 1e095, 1e096, 1e097, 1e098, 1e099, //
    1e100, 1e101, 1e102, 1e103, 1e104, 1e105, 1e106, 1e107, 1e108, 1e109, //
    1e110, 1e111, 1e112, 1e113, 1e114, 1e115, 1e116, 1e117, 1e118, 1e119, //
    1e120, 1e121, 1e122, 1e123, 1e124, 1e125, 1e126, 1e127, 1e128, 1e129, //
    1e130, 1e131, 1e132, 1e133, 1e134, 1e135, 1e136, 1e137, 1e138, 1e139, //
    1e140, 1e141, 1e142, 1e143, 1e144, 1e145, 1e146, 1e147, 1e148, 1e149, //
    1e150, 1e151, 1e152, 1e153, 1e154, 1e155, 1e156, 1e157, 1e158, 1e159, //
    1e160, 1e161, 1e162, 1e163, 1e164, 1e165, 1e166, 1e167, 1e168, 1e169, //
    1e170, 1e171, 1e172, 1e173, 1e174, 1e175, 1e176, 1e177, 1e178, 1e179, //
    1e180, 1e181, 1e182, 1e183, 1e184, 1e185, 1e186, 1e187, 1e188, 1e189, //
    1e190, 1e191, 1e192, 1e193, 1e194, 1e195, 1e196, 1e197, 1e198, 1e199, //
    1e200, 1e201, 1e202, 1e203, 1e204, 1e205, 1e206, 1e207, 1e208, 1e209, //
    1e210, 1e211, 1e212, 1e213, 1e214, 1e215, 1e216, 1e217, 1e218, 1e219, //
    1e220, 1e221, 1e222, 1e223, 1e224, 1e225, 1e226, 1e227, 1e228, 1e229, //
    1e230, 1e231, 1e232, 1e233, 1e234, 1e235, 1e236, 1e237, 1e238, 1e239, //
    1e240, 1e241, 1e242, 1e243, 1e244, 1e245, 1e246, 1e247, 1e248, 1e249, //
    1e250, 1e251, 1e252, 1e253, 1e254, 1e255, 1e256, 1e257, 1e258, 1e259, //
    1e260, 1e261, 1e262, 1e263, 1e264, 1e265, 1e266, 1e267, 1e268, 1e269, //
    1e270, 1e271, 1e272, 1e273, 1e274, 1e275, 1e276, 1e277, 1e278, 1e279, //
    1e280, 1e281, 1e282, 1e283, 1e284, 1e285, 1e286, 1e287, 1e288, 1e289, //
    1e290, 1e291, 1e292, 1e293, 1e294, 1e295, 1e296, 1e297, 1e298, 1e299, //
    1e300, 1e301, 1e302, 1e303, 1e304, 1e305, 1e306, 1e307, 1e308,
];

pub(crate) enum ReaderNumber {
    F64(f64),
    U64(u64),
    I64(i64),
}

impl ReaderNumber {
    pub(crate) fn visit<'de, V>(self, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        match self {
            ReaderNumber::F64(x) => visitor.visit_f64(x),
            ReaderNumber::U64(x) => visitor.visit_u64(x),
            ReaderNumber::I64(x) => visitor.visit_i64(x),
        }
    }
}

pub(crate) enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T> ops::Deref for Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match *self {
            Reference::Borrowed(b) => b,
            Reference::Copied(c) => c,
        }
    }
}
