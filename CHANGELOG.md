# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] - ReleaseDate
## [0.3.0](https://github.com/3xMike/config-manager/releases/tag/0.3.0) - 2025-03-12
### Changed
- default and init_from now takes code without quotation marks. I.e. following old code is invalid:
```rust
#[config]
struct Config {
    #[default = "HashMap::new()"]
    map: HashMap<usize, String>
}
```
Valid code now is:
```rust
#[config]
struct Config {
    #[default = HashMap::new()]
    map: HashMap<usize, String>
}
```
- Using the default for String fields will invoke Into::into() before init.
Therefore it's possible to use `&str` for String default initialization:
```rust
#[config]
struct Config {
    #[default = "default string value"]
    s: String
}
```

## [0.2.1](https://github.com/3xMike/config-manager/releases/tag/0.2.1) - 2025-03-11
### Added
- Clap field `flag` attribute.
- Clap field `help_heading` attribute.

## [0.2.0](https://github.com/3xMike/config-manager/releases/tag/0.2.0) - 2023-04-03
### Changed
- `config` macro isn't implementing `serde::Deserialize` implicitly for the annotated struct now.
- If a field is annotated with `deserialize_with`, the field type is not required to implement `serde::Deserialize`.
- The signature of a function argument of the `deserialize_with` is now: 
```rust
fn fn_name(s: &str) -> Result<FieldType, String>
```

## [0.1.1](https://github.com/3xMike/config-manager/releases/tag/0.1.1) - 2023-03-06
### Added
- If a field is not annotated, default source order will be assigned.
### Changed
- The default behavior of `env_prefix` is no prefix instead of a binary name now.
## [0.1.0](https://github.com/3xMike/config-manager/releases/tag/0.1.0) - 2022-12-27
Initial version.