# Serde query string parser

It is an alternative query string parser based on serde for rust.

## Why

If you know, you know

## Note

It is not production ready in any way and is hardly even in alpha state, so be aware of that, I just uploaded it here so you can play with it and possibly report some bugs and give me some ideas.

Benchmarks provided are far from accurate and I only use them to check how different changes I make affect the performance so take them with a big grain of salt. I'm not in any way qualified to write benchmarks and I don't even know if those benchmarks work correctly.

## Alternatives

If you're looking for a more production ready alternative, consider looking at these crates:

`serde_urlencoded`: a performant query parser which doesn't support subkeys (aka dicts)

`serde_qs` a better alternative to this one which is more mature and is featureful enough for most cases

## Credit

It uses a good amount of code ported or copied from `serde_json` for example to parse numbers or strings.

## Todo

- Check if having a seprator deserializer for keys/values makes sense(it should)
- Fix `TODO` comments
- Writing tests for invalid cases
- Serializer
