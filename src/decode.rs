use std::borrow::Cow;

#[inline]
pub fn parse_char(h: u8, l: u8) -> Option<u8> {
    Some(char::from(h).to_digit(16)? as u8 * 0x10 + char::from(l).to_digit(16)? as u8)
}

/// Decode a slice to the scratch or directly return the slice if not encoded
pub fn parse_bytes<'de, 's>(
    slice: &'de [u8],
    scratch: &'s mut Vec<u8>,
) -> Reference<'de, 's, [u8]> {
    scratch.clear();

    // Index of the last byte we copied to scratch
    let mut index = 0;

    // Index of the first byte not yet copied into the scratch space.
    let mut cursor = 0;

    while let Some(v) = slice.get(cursor) {
        match v {
            b'+' => {
                scratch.extend_from_slice(&slice[index..cursor]);
                scratch.push(b' ');

                cursor += 1;
                index = cursor;
            }
            b'%' => {
                // we saw percentage
                if slice.len() > cursor + 2 {
                    match parse_char(slice[cursor + 1], slice[cursor + 2]) {
                        Some(b) => {
                            scratch.extend_from_slice(&slice[index..cursor]);
                            scratch.push(b);

                            cursor += 3;
                            index = cursor;
                        }
                        None => {
                            // If it wasn't valid, go to the next byte
                            cursor += 1;
                        }
                    }
                } else {
                    cursor += 1;
                }
            }
            _ => {
                cursor += 1;
            }
        }
    }

    if scratch.is_empty() {
        Reference::Borrowed(&slice[index..cursor])
    } else {
        scratch.extend_from_slice(&slice[index..cursor]);
        Reference::Copied(scratch)
    }
}

pub enum Reference<'b, 'c, T>
where
    T: ?Sized + 'static,
{
    Borrowed(&'b T),
    Copied(&'c T),
}

impl<'b, 'c, T> Reference<'b, 'c, T>
where
    T: ?Sized + ToOwned + 'static,
{
    pub fn into_cow(self) -> Cow<'b, T> {
        match self {
            Reference::Borrowed(b) => Cow::Borrowed(b),
            Reference::Copied(c) => Cow::Owned(c.to_owned()),
        }
    }

    pub fn try_map<F, B, E>(self, f: F) -> Result<Reference<'b, 'c, B>, E>
    where
        F: FnOnce(&T) -> Result<&B, E>,
        B: ?Sized + 'static,
    {
        match self {
            Reference::Borrowed(b) => f(b).map(Reference::Borrowed),
            Reference::Copied(c) => f(c).map(Reference::Copied),
        }
    }
}

impl<'b, 'c, T> std::ops::Deref for Reference<'b, 'c, T>
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
