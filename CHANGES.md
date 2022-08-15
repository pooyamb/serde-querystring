# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]
### Added
- Support encoded brackets in brackets mode

## [0.1-beta-1] - 2022-08-15
### Fixed
- Providing size_hint for map types in brackets mode
- Stop parsing delimiters in delimiter mode when parsing a single value, ex "hello|world" as string will be used as is

## [0.1-beta.0] - 2022-08-13

- Rebuilt from the ground up to support multiple parsing methods
- Moved to using lexical instead of the half-baked copy of serde-json for number parsing
