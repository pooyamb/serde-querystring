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
  <img src="https://img.shields.io/github/workflow/status/pooyamb/serde-querystring/Test?style=flat-square" alt="actions status" />
  <img alt="Crates.io license" src="https://img.shields.io/crates/l/serde-querystring?style=flat-square">
</div>

<br>
An alternative query string parser based on serde for rust.

## Install

```toml
# Cargo.toml
[dependencies]
serde-querystring = "0.0.7"
```

## Usage

```rust
// In your main function
let x: MyStruct = serde_querystring::from_str("YOUR QUERY STRING");
```

To see what is supported and what is not, please [read the docs](https://docs.rs/serde-querystring).

## Why

Existing alternatives don't cover some cases, for example enums (having enums in sequences, usefull for filters in rest apis) is not a first class value in similar crates. This crate tries to cover more real world use cases and it uses a different strategy to parse the query string which gives it some freedom to decide how to parse depending on the resulting data structure. Note that it may not be fully compatible with some existing standards in some cases, but it tries to support the cases they defined.

## Warning

This project is still in its early stage of development and things may change without notice, so use it at your own risk.

## Alternatives

If you're looking for a more production ready alternative, consider looking at these crates:

`serde_urlencoded`: a performant query parser which doesn't support subkeys (aka dicts)

`serde_qs` a better alternative to this one which is more mature and is featureful enough for most cases

## Known bugs

- Doesn't have correct error reporting yet
- Doesn't support unit types
- Doesn't check if it visited all the input when visiting subkeys
- Doesn't support deserializing into a sequence of key-values instead of map(More of a feature)

Tests only cover valid querystrings in some cases, so there can be some bugs here and there. If you face a bug, please open an issue and let me know.

## Credit

It uses a good amount of code ported or copied from `serde_json` crate and some lines from `form_urlencoded` crate for example to parse numbers or strings.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
