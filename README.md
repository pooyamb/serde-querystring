# Serde query string parser

An alternative query string parser based on serde for rust.

## Install

```toml
# Cargo.toml
[dependencies]
serde-querystring = "0.0.4"
```

## Usage

```rust
// In your main function
let x: MyStruct = serde_querystring::from_str("YOUR QUERY STRING");
```

Look at the tests folder to find the samples of what is supported and what is not.

## Why

Existing alternatives don't cover some cases, for example enums (having enums in sequences, usefull for filters in rest apis) is not a first class value in similar crates. This crate tries to cover more real world use cases and it uses a different strategy to parse the query string which gives it some freedom to decide how to parse depending on the resulting data structure. Note that it may not be fully compatible with some existing standards in some cases, but it tries to support the cases they defined.

## Warning

This project is in Alpha stage as of now, so use it at your own risk.

Benchmarks provided may not be accurate and I'm not in any way qualified to write benchmarks. Feel free to open pull requests if you see a problem.

## Alternatives

If you're looking for a more production ready alternative, consider looking at these crates:

`serde_urlencoded`: a performant query parser which doesn't support subkeys (aka dicts)

`serde_qs` a better alternative to this one which is more mature and is featureful enough for most cases

## Known bugs

- Doesn't have correct error reporting yet
- Not known(Please open issues)

Tests only cover valid cases, so there can be some bugs here and there. If you face a bug [lease open an issue and let me know.

## Credit

It uses a good amount of code ported or copied from `serde_json` crate and some lines from `form_urlencoded` crate for example to parse numbers or strings.

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
