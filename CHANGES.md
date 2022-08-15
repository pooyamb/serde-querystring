## 0.1-beta-1
### Fixed
- Providing size_hint for map types in brackets mode
- Stop parsing delimiters in delimiter mode when parsing a single value, ex "hello|world" as string will be used as is

## 0.1-beta.0 

- Rebuilt from the ground up to support multiple parsing methods
- Moved to using lexical instead of the half-baked copy of serde-json for number parsing
