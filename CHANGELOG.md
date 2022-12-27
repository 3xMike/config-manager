# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate

## [0.1.0](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.1.0) - 2022-12-26
### Added
- License

## [0.0.11](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.11) - 2022-12-13
### Added
- `ConfigInit::get_command` method
### Fixed
- Bug that a field with no clap attribute actually has a clap entity

## [0.0.10](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.10) - 2022-11-28
### Added
- `ConfigInit::parse_options` method
### Removed
- `global_name` attribute

## [0.0.9](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.9) - 2022-11-28
### Added
- Setting fields by layers 

## [0.0.8](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.8) - 2022-11-16
### Added
- `subcommand` attribute
- `source(deserialize_with)` attribute
- Empty `source(default)` for types implements `Default`
- `table` struct attribute

## [0.0.7](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.7) - 2022-11-08
### Added
- `clap` nested attributes support `init_from` again
- `flatten` attribute
### Removed
- `source(clap(flatten))`
- `source(clap(subcommand))`

## [0.0.6](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.6) - 2022-10-24
### Removed
- `Warning` enum
- `ParseResult` struct
### Changed
- `ConfigInit::parse` returns `Result<Self>` now (instead of `Result<ParseResult<Self>>`)
- Generated global variable is `Self` now (instead of `ParseResult<Self>`) 
- `file` attribute now has the following nested attributes: `format`, `clap`, `env`, `default` and `optional`. 
If the last one set as `true` and none of `clap`, `env`, `default` is not found, programm will not panic

## [0.0.5](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.5) - 2022-09-29
### Changed
- `clap` attribute. Now it works according to [documentation](https://docs.rs/clap/latest/clap/_derive/index.html), so no `init_from` for `clap` now

## [0.0.4](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.4) - 2022-09-09
### Added
- `ConfigInit` trait, thus
    - `ParseResult` struct
    - `Warning` struct
    - `Error` enum
    - `global_name` attribute
- `clap(author, version, about)` from manifest
### Changed
- Global variable type is `ParseResult<Self>` instead of `Lazy<Self>`

## [0.0.3](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.3) - 2022-06-23
### Added
- `__private` mod

## [0.0.2](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.2) - 2022-06-15
### Added
- Rust-like `default` attribute
- Reexport internal-use crates

## [0.0.1](https://gitlab.kryptodev.ru/dev/research/rust/config-manager/-/tree/0.0.1) - 2022-06-07
Initial tag