mod pair;
mod stash;
mod value;

use stash::Stash;

use self::pair::Pair;
use crate::error::Result;

struct KVParser<'de> {
    slice: &'de [u8],
}

impl<'de> KVParser<'de> {
    pub fn new(slice: &'de [u8]) -> Self {
        Self { slice }
    }

    fn parse(self) -> Stash<'de> {
        let mut stash = Stash::new();
        for subslice in self.slice.split(|c| *c == b'&') {
            if subslice.len() == 0 {
                continue;
            }

            let mut iter = subslice.splitn(2, |c| *c == b'=');
            let key = iter.next().unwrap();
            let value = iter.next();

            stash.push(Pair::new(key, value));
        }

        stash
    }
}

pub fn from_str<'de, T>(slice: &'de str) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    from_bytes(slice.as_bytes())
}

pub fn from_bytes<'de, T>(input: &'de [u8]) -> Result<T>
where
    T: serde::de::Deserialize<'de>,
{
    let de = KVParser::new(input).parse();
    let res = serde::de::Deserialize::deserialize(de)?;
    Ok(res)
}
