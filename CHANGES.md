# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [0.3.0]

### Changed

- Switched back from `atoi` and `std` to `lexcial` because it has fixed its soundness issues.

## [0.3.0-beta.0] - 2024-08-08

### Changed

- Removed the unmaintained `lexical` from dependencies
- Switched to `atoi` for parsing integers and `std` for parsing floats
- General code improvements and pipeline fix
- Updated `derive_more` to v1 for `serde-querystring-actix` (reverted due to MSRV)

## [0.2.1] - 2023-03-06

### Fixed

- Fixed the wrong version number in the README and docs

## [0.2.0] - 2023-02-01

### Added

- Provide an extractor for axum

### Fixed

- When having Option<Vec> or any sequence type as a struct field, deserialization failed.

### Changed

- `key=` and `key` now cause deserialization error for Option<T> instead of giving `None`.
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
