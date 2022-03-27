use std::borrow::Cow;
use std::fmt;

#[derive(Debug)]
pub struct ParsedSlice<'de>(pub Cow<'de, [u8]>);

impl<'de> fmt::Display for ParsedSlice<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(&self.0))
    }
}

#[derive(Debug)]
pub struct RawSlice<'de>(pub &'de [u8]);

impl<'de> fmt::Display for RawSlice<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&String::from_utf8_lossy(self.0))
    }
}

#[derive(Debug)]
pub struct OptionalRawSlice<'de>(pub Option<&'de [u8]>);

impl<'de> fmt::Display for OptionalRawSlice<'de> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(v) => f.write_str(&String::from_utf8_lossy(v)),
            None => f.write_str("none"),
        }
    }
}
