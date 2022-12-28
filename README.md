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
serde-querystring = "0.1.0-beta.3"
```

## Usage

You can use the parsers provided in this crate directly, examples are available in each parser's tests.

```rust
let parsed = DuplicateQS::parse(b"foo=bar&foo=baz");
let values = parser.values(b"foo"); // Will give you a vector of b"bar" and b"baz"
```

Or you can use serde(with `serde` feature, enabled by default)

```rust
use serde_querystring::de;

let parsed: MyStruct = de::from_str("foo=bar&foo=baz", de::ParseMode::Duplicate).unwrap();
```

There is also `serde-querystring-actix` crate to support `actix-web`. It provides `QueryString` extractor which works just like the actix-web's own web::Query but uses `serde-querystring` to deserialize.

## Parsers

### `UrlEncodedQS` or `ParseMode::UrlEncoded`

Simply parses key=value pairs, accepting only one value per key. In case a key is repeated, we only collect the last value.

### `DuplicateQS` or `ParseMode::Duplicate`

Just like UrlEncoded mode, except that if a key is repeated, we collect all the values for that key.

### `DelimiterQS` or `ParseMode::Delimiter`

Uses a delimiter byte to parse multiple values from a slice of value, ex: `"key=value1|value2|value3"`

### `BracketsQS` or `ParseMode::Brackets`

Works like the PHP querystring parser, using brackets and subkeys to assign values, ex: `"key[a]=value&key[b]=value"`

## Credit

We use some lines of code from `form_urlencoded` to parse percent encoded chars.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
