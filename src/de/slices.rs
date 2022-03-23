use std::borrow::Cow;

pub struct ParsedSlice<'s>(pub Cow<'s, [u8]>);

pub struct RawSlice<'s>(pub &'s [u8]);

pub struct OptionalRawSlice<'s>(pub Option<&'s [u8]>);
