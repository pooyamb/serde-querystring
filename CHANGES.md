# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.1.1]
### Fixed
- When having Option<Vec> or any sequence type as a struct field, deserialization failed.
### Changed
- in UrlEncoded mode, `key=` and `key` now cause deserialization error for Option<T> instead of giving `None`.
- Reexport the `ParseMode`, `Error` and `ErrorKind` enum from crate's root when `serde` feature is active.

## [0.1.0] - 2022-12-28
### Added
- Added more docs and improved the error handling.
### Fixed
- When having Option<SubType> as a struct field, deserialization always failed when SubType was itself a struct.

## [0.1-beta.2] - 2022-08-15
### Added
- Support encoded brackets in brackets mode

## [0.1-beta-1] - 2022-08-15
### Fixed
- Providing size_hint for map types in brackets mode
- Stop parsing delimiters in delimiter mode when parsing a single value, ex "hello|world" as string will be used as is

## [0.1-beta.0] - 2022-08-13

- Rebuilt from the ground up to support multiple parsing methods
- Moved to using lexical instead of the half-baked copy of serde-json for number parsing
