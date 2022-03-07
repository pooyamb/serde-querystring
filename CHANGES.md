## 0.1 (Unreleased)

- Removed the support for brackets
- Moved to support sequences with duplicate assignments `foo=bar&foo=baz`
- The behaviour should be much more consistent for repeated keys
- Moved to using lexical instead of the half-baked copy of serde-json for number parsing
- Improved error reporting
