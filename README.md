<h1 align="center">Serde-querystring</h1>
<br />

<div align="center">
  <a href="https://crates.io/crates/serde-querystring">
    <img src="https://img.shields.io/crates/v/serde-querystring.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <a href="https://docs.rs/serde-querystring">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
  <img src="https://img.shields.io/github/actions/workflow/status/pooyamb/serde-querystring/test.yml?style=flat-square" alt="actions status" />
  <img alt="Codecov" src="https://img.shields.io/codecov/c/github/pooyamb/serde-querystring?style=flat-square">
  <img alt="Crates.io license" src="https://img.shields.io/crates/l/serde-querystring?style=flat-square">
</div>

<br>
A query string parser for rust with support for different parsing methods.

## Install

```toml
# Cargo.toml
[dependencies]
serde-querystring = "0.3.0"
```

## Usage

You can use the parsers provided in this crate directly, examples are available in each parser's tests.

```rust
use serde_querystring::DuplicateQS;

let parsed = DuplicateQS::parse(b"foo=bar&foo=baz");
let values = parsed.values(b"foo"); // Will give you a vector of b"bar" and b"baz"
```

Or you can use serde(with `serde` feature, enabled by default)

```rust,ignore
use serde::Deserialize;
use serde_querystring::{from_str, ParseMode, DuplicateQS};

#[derive(Deserialize)]
struct MyStruct{
  foo: Vec<String> // Or (String, u32) tuple
}

let parsed: MyStruct = from_str("foo=bar&foo=2022", ParseMode::Duplicate).unwrap();
// or
let parsed: MyStruct = DuplicateQS::parse(b"foo=bar&foo=baz").deserialize().unwrap();
```

There are also crates for `actix_web`(`serde-querystring-actix`) and `axum`(`serde-querystring-axum`) which provide extractors for their frameworks and can be used without directly relying on the core crate.

## Parsers

### Simple Mode

Simply parses key=value pairs, accepting only one value per key. In case a key is repeated, we only collect the last value.

```rust,ignore
use serde_querystring::{UrlEncodedQS, ParseMode, from_str};

UrlEncodedQS::parse(b"key=value");
// or
let res: MyStruct = from_str("foo=bar&key=value", ParseMode::UrlEncoded).unwrap();
```

### Repeated key mode

Supports vectors or values by repeating a key.

```rust,ignore
use serde_querystring::{DuplicateQS, ParseMode, from_str};

DuplicateQS::parse(b"foo=bar&foo=bar2&foo=bar3");
// or
let res: MyStruct = from_str("foo=bar&foo=bar2&foo=bar3", ParseMode::Duplicate).unwrap();
```

### Delimiter mode

Supports vectors or values by using a delimiter byte(ex. b'|').

```rust,ignore
use serde_querystring::{DelimiterQS, ParseMode, from_str};

DelimiterQS::parse(b"foo=bar|bar2|bar3", b'|');
// or
let res: MyStruct = from_str("foo=bar|bar2|bar3", ParseMode::Delimiter(b'|')).unwrap();
```

### Brackets mode

Supports vectors or values by using a brackets and subkeys.

```rust,ignore
use serde_querystring::{BracketsQS, ParseMode, from_str};

BracketsQS::parse(b"foo[1]=bar&foo[2]=bar&foo[3]=bar");
// or
let res: MyStruct = from_str("foo[1]=bar&foo[2]=bar&foo[3]=bar", ParseMode::Brackets).unwrap();
```

## Credit

We use some lines of code from `form_urlencoded` to parse percent encoded chars.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
