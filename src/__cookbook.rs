// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

//! # Documentation: Cookbook
//! 1. [Examples](#examples)
//! 2. [Intro](#intro)
//! 3. [Options](#options)
//! 4. [Structure level attributes](#structure-attributes)
//!     1. [env_prefix](#env_prefix)
//!     2. [file](#file)
//!     3. [clap](#clap)
//!     4. [table](#table)
//!     5. [default source order](#default_order)
//! 5. [Field level attributes](#field-attributes)
//!     1. [source](#source)
//!         - [default](#default)
//!         - [env](#env)
//!         - [config](#config)
//!         - [clap](#clap-1)
//!         - [deserialize_with](#deserialize_with)
//!     2. [flatten](#flatten)
//!         - [attributes](#flatten-attributes)
//!     3. [subcommand](#subcommand)
//! 6. [`get_command` method](#get_command)
//! # [Appendix](#appendix)
//! 1. [Allowed clap attributes](#clap-attributes)
//!     1. [App attributes](#clap-command)
//!     2. [Field attributes](#clap-arg)
//! ## Examples
//! There are [tests](https://github.com/3xMike/config-manager/tree/main/tests)
//! and [examples](https://github.com/3xMike/config-manager/tree/main/examples) in
//! the crate repository to get you started
//!
//! ## Intro
//! To label a structure as a config, it is required to annotate it with `#[config]`:
//! ```
//! use config_manager::config;
//!
//! #[config]
//! struct Application {}
//! ```
//! **OBTAINING RESULTS**\
//! [ConfigInit](../trait.ConfigInit.html) trait will be derived
//! for the struct and one can invoke the initialization and obtain the result
//! with [`<Application as ConfigInit>::parse()`](../trait.ConfigInit.html#method.parse)
//! or [`<Application as ConfigInit>::parse_options(options)`](../trait.ConfigInit.html#tymethod.parse_options) method.
//!  
//! **All the sources of the value of a field must be specified explicitly.** Fields that does not
//! have at least one source specified are not allowed.
//! ```
//! use config_manager::config;
//!
//! #[config]
//! struct Application {
//!     #[source(clap(long = "cli_my_field"), env = "ENV_MY_FIELD")]
//!     my_field: i32,
//! }
//! ```
//!
//! In this example, it will be checked that `cli_my_field` is specified via the command line
//! interface (i.e. <nobr>`./your_binary --cli_my_field=42`;</nobr>
//! see [the clap documentation](https://docs.rs/clap/3.1.18/clap/) for more details).
//! If `cli_my_field` is indeed specified, it will be parsed with `serde`
//! and, if the parsing is successful, the value for `my_field` will be assigned from the result.
//! In case of a parsing error, the error will be returned instead.
//!
//! If `cli_my_field` is not specified, it will be checked that the `ENV_MY_FIELD`
//! environment variable is present. If the `ENV_MY_FIELD` environment variable is present, its
//! value will be parsed with `serde` and, if the parsing is successful, the value for `my_field`
//! will be assigned from the result. In case of a parsing error, the error will be returned instead.
//!
//! If the `ENV_MY_FIELD` environment variable is not found, an error will be returned, because
//! this is the last source we can take the value from, and there was a failure.
//!
//! ### Note
//! The order of the sources is important! The following example does **NOT** do the same thing as
//! the previous:
//! ```
//! use config_manager::config;
//!
//! #[config]
//! struct Application {
//!     #[source(env = "ENV_MY_FIELD", clap(long = "cli_my_field"))]
//!     my_field: i32,
//! }
//! ```
//!
//! In this example, the `env` source will be checked first.
//!
//! **NOTES**
//! - The possible sources are: `clap`, `env`, `config`, `default` (see below)
//! - Default value will be assigned the last (after the others were not found).
//! - If the value is not found in any of the sources, an error will be returned
//! - Field type must implement `serde::de::Deserialize`
//! - All attributes except `default` must match either `attribute = literal`, or
//!     `attribute(init_from = ...valid Rust code...)`, or `attribute`. In the last case, the "key"
//!     value (the CLI argument name, the environment variable name, or the config file key name —
//!     depending on the source) will match the field name. For example, annotating `my_field` with
//!     `#[clap]` means that the value could be assigned to `my_field` by specifying
//!     `--my_field=...` via the CLI
//! - Attribute `default` must match `default = ...valid Rust code...` or `default`
//! - If the `deserialize_with` attribute is not set, values from command line,
//!     environment will be deserialized according to [hjson syntax](https://hjson.github.io/)
//!
//! ## Options
//! Parsing process may be run with a set of options by using the [ConfigInit::parse_options(options)](../trait.ConfigInit.html#tymethod.parse_options).
//! The key point here is the fact that the options take precedence over the corresponding structure attributes, that can be useful in testing and other cases.\
//! More information can be found in the [ConfigOption](../enum.ConfigOption.html) documentation.
//!
//! ## Structure attributes
//! ### `env_prefix`
//! Prefix of the environment variables. If not specified, the prefix will not be added.
//! Thus, the `iter` field in the example below will be searched in the environment by the `demo_iter` key.
//! ```
//! # use config_manager::config;
//! #
//! #[config(
//!     env_prefix = "demo"
//! )]
//! struct AppConfig {
//!     #[source(env)]
//!     iter: i32,
//! }
//! ```
//! **Notes**
//! - The delimiter ('_') is placed automatically
//! - `env_prefix = ""` will not add any prefix
//! - `env`, `env_prefix` and similar attributes are case-insensitive. If both the `demo_iter` and
//!     `DEMO_ITER` environment variables are present, which of these two will be parsed *is not defined*
//! - One can use `env_prefix` (without a value) to set the binary file name as a prefix
//!
//! **Example**
//! ```
//! # use config_manager::config;
//! #
//! #[config(env_prefix)]
//! struct Config {
//!     #[source(env)]
//!     capacity: i32,
//! }
//! ```
//! In the example above, the `capacity` field will be searched in the environment
//! by the "*bin*_capacity" key, where `bin` is the name of the executable file.
//!
//! ### `file`
//! Description of the configuration file. Has the following nested attributes:
//! - `format`: `toml`/`json`/`yaml`/`ron`/`json5`
//! - `env`: environment key containing path to the configuration file (case-insensitive)
//! - `clap`: clap attributes of the argument, responsible for the path to the configuration file
//!
//! **Note:** in this case, clap attribute must have the nested `long` attribute (`clap(long = "...")`)
//! - `default`: default configuration file path
//! - `optional`: If the attribute is set, macro won't return Error if file is not found at set (by `clap`/`env`/`default`) path. Does not take values.\
//!
//! **Note:** It is allowed to specify multiple files: all of them will be merged.
//!     If there is a collision (the values of a particular key have been specified in two or more files),
//!     the value will be assigned from the file that has been described later (in the attribute list).
//!
//! **Example**
//! ```
//! # use config_manager::config;
//! #
//! #[config(
//!     env_prefix = "",
//!     file(format = "toml", env = "demo_config")
//! )]
//! struct AppConfig {
//! #[source(clap(long), env, default = 5)]
//!     iter: i32,
//! }
//! ```
//! In this case, the initialization order for the `iter` field is:
//! - command line argument `--iter`
//! - environment variable `iter`
//! - variable `iter` from configuration file with path set by the `demo_config` environment variable
//! - default value (`5`)
//!
//! ### `clap`
//! Clap app attributes, like `name`, `version`, etc.
//! Full list of supported clap attributes can be checked in the [Appendix](#clap-command)
//!
//! **Note**: Following attributes can be used without value (for example: `clap(name)`):
//! - `name`: will be taken as package from Cargo.toml,
//! - `version`: will be taken as crate version from Cargo.toml,
//! - `author`: will be taken as crate authors from Cargo.toml,
//! - `about`: will be taken as crate discription from Cargo.toml,
//! - `long_about`: will be taken from doc comments of the struct (aka `///` or `/** */`).
//!
//! ### `table`
//! Table of the configuration files to find fields of the structure.
//!
//! **Example**
//! ```
//! # use config_manager::config;
//! #
//! #[config(file(format = "toml", default = "./config.toml"), table = "input.data")]
//! struct Config {
//!     #[source(config)]
//!     frames: i32,
//! }
//! ```
//! Field `frames` will be searched in the "input.data" table of the configuration file "config.toml".
//!
//! ### `default_order`
//! The default order of any field that wasn't annotated with any of `source`,`flatten` or `subcommand`.\
//! `clap`, `env`, `config` and `default` are all possible parameters.
//! Each attribute will be applied to each unannotated field in a "short" form
//! (i.e., form without value; for example, `#[source(default)]` means that
//! `Default::default()` will be used as a default value. See the [source](#source) section for more information).
//! **Example**
//! ```
//! # use config_manager::config;
//! #
//! #[config(default_order(env, clap, default))]
//! struct Config {
//!     rotation: f32,
//! }
//! ```
//! It will be checked that the `ROTATION` environment variable is set; if not, the `--rotation` command line argument will be checked,
//! and, lastly, the `Default::default()` will be assigned.
//! **Note:** If this attribute isn't set, the default order is:
//! 1. command line
//! 2. environment variables
//! 3. configuration files
//!
//! ## Field attributes
//! Only fields can be annotated with the following attributes and only one of them can be assigned to a field.
//!
//! **Note:** if a field is not annotated with any of the following attributes,
//! it will be parsed using the default source order (see the section above).
//!
//! ### Source
//! If a field is annotated with the `source` attribute, at least one of the following nested attributes must be present.
//!
//! #### `default`
//! Valid Rust code that will be assigned to the field if other sources are not found. \
//! If the attribute is set without a value (`#[source(default)]`),
//! the default value is [Default::default()].
//!
//! **Example**
//! ```
//! # use config_manager::config;
//! #
//! #[config]
//! struct AppConfig {
//!     #[source(default = Vec::new())]
//!     buf: Vec<String>,
//!     #[source(default)]
//!     opt: Option<String>
//!     // Option::<String>::default() will be assigned (None)
//! }
//! ```
//! **Note:** For [String] fields [Into::into()] will be invoked under the hood. So it is possible to use `&str` to initialize [String] field:
//!  ```
//! # use config_manager::config;
//! #
//! #[config]
//! struct AppConfig {
//!     #[source(default = "default value")]
//!     string: String,
//! }
//! ```
//!  
//! #### `env`
//! The name of the environment variable from which the value is to be set.
//! `env_prefix` (see above) is ignored if present with a value (`#[source(env = "...")]`).  The case is ignored. \
//! If the attribute is set without value, the name of the environment variable to be set is `env_prefix + field_name`.
//!
//! #### `config`
//! Name of the configuration file field to set the value from. It can contain dots: in this case
//! the name will be parsed as the path of the field.\
//! If the attribute is set without a value (`#[source(config)]`), the field name is the name of the configuration file field to be set.
//! **Example**
//! ```
//! # use config_manager::config;
//! #
//! #[config(file(format = "toml", default = "./config.toml"), table = "input.data")]
//! struct Config {
//!     #[source(config = "images.frame_rate")]
//!     rate: i32,
//! }
//! ```
//! Field `rate` will be searched in the "input.data.images" table of the "config.toml"
//! configuration file by the `frame_rate` key.
//!
//! #### `clap`
//! Clap-crate field attributes, like `long`, `short`. Full list of supported clap attributes can be checked in the [Appendix](#clap-arg).
//!
//! **Note:** There is a new attribute: `flag`. If used on `field: bool`, this field can be set via CLI like a flag: `--field`. \
//! **Note:** Following attributes can be used without value (for example: `clap(short)`):
//! - `long`: the field name,
//! - `short`: the first letter of the field name,
//! - `help`: will be taken from doc comments of the field (aka `///` or `/** */`),
//! - `long_help`: will be taken from doc comments of the field (aka `///` or `/** */`),
//!
//! **Note:** `#[source(clap)]` is equivalent to `#[source(clap(long))]` \
//! **Note:** boolean fields can be marked as `#[source(clap(flag))]` that allow to set it as `true` with no value provided. \
//! **Example:** the following field can be set to `true` using the CLI: `./my_app -f` or `./my_asp --flag true`.
//! ```
//! # use config_manager::config;
//!
//! #[config]
//! struct Cfg {
//!     #[source(clap(long, short, flag))]
//!     flag: bool
//! }
//! ```
//!
//! #### `deserialize_with`
//! Custom deserialization of the field. The deserialization function should have the following signature:
//! ```ignore
//! fn fn_name<S: AsRef<str>>(s: S) -> Result<FieldType, impl std::fmt::Display>
//! ```
//! **Note:** actually, `&String` will be passed to the function,
//! so function can take any argument that is derivable from `&String`.
//! It may be `&str`, `&String`, `T: AsRef<str>`, `T: AsRef<String>`, and so on.
//! It is recommended to choose error type explicitly.
//!
//! **Example**
//! ```
//! # use config_manager::config;
//! use std::time::Duration;
//!
//! #[config]
//! struct MethodConfig {
//!     #[source(clap(long), deserialize_with = "deser_duration")]
//!     a: Duration,
//! }
//!
//! fn deser_duration(dur: &str) -> Result<Duration, String> {
//!     match dur.parse::<u64>() {
//!         Ok(dur) => Ok(Duration::from_millis(dur)),
//!         Err(err) => Err(err.to_string()),
//!     }
//! }
//! ```
//!
//! ### Flatten
//! If a field is annotated with the `flatten` attribute, it will be parsed as a nested structure and its fields will be initiated
//! like fields of the primary config. In this case, the field's type must implement `config_manager::Flatten`
//! (it is highly discouraged to implement this trait manually, use derive macro: `#[derive(Flatten)]`) and `serde::Deserialize`
//!
//! **Example**
//! ```
//! use config_manager::{config, Flatten};
//! # use serde::Deserialize;
//! #
//! #[config]
//! struct PrimalConfig {
//!     #[flatten]
//!     child: NestedConfig,
//! }
//!
//! #[derive(Deserialize, Flatten)]
//! struct NestedConfig {
//!     #[source(env = "recharge")]
//!     recharge_time: f32,
//!     #[source(default = 0.0)]
//!     capacity: f32,
//! }
//! ```
//! **Notes:**
//! - Nested configs can also contain `flatten` fields
//! - `env_prefix` will be inherited from the initial struct
//!
//! #### Flatten attributes
//! Flatten struct may have the following helper attributes: `table`, `flatten`, `source` (they work the same way as the described above ones).
//! ### Subcommand
//! If a field is annotated with the `subcommand` attribute, it will be taken as a `clap` subcommand
//! (see [clap documentation](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#subcommands) for more info).
//! The field's type must implement `clap::Subcommand` and `serde::Deserialize`.
//!
//! **Example**
//! ```
//! # use config_manager::config;
//! # use serde::Deserialize;
//! # use clap;
//! #
//! #[config]
//! struct Cargo {
//!     #[subcommand]
//!     sub: CargoCommands,
//! }
//!
//! #[derive(Deserialize, clap::Subcommand)]
//! enum CargoCommands {
//!     #[clap(about = "Compile the current package")]
//!     Build {
//!     // ...
//!     },
//!     #[clap(about = "Analyze the current package and report errors, but don't build object files")]
//!     Check {
//!     // ...
//!     },
//!     #[clap(about = "Build this package's and its dependencies' documentation")]
//!     Doc,
//!     #[clap(about = "Create a new cargo package")]
//!     New,
//!     // ...
//! }
//! ```
//! **Notes:**
//! - Value for the `subcommand` enumeration will be searched only in command line, so the `source` and the `flatten` attributes are forbidden
//!     (flatten `subcommand` attribute is allowed due to clap documentation).
//! - Multiple `subcommand` fields are forbidden.
//! - `subcommand` field in nested(`flatten`) structures are forbidden.
//! - `subcommand` field can be optional (`Option<T>`, `T: clap::Subcommand + serde::Deserialize`),
//!     so if no subcommand is found in the command line, the `None` will be assigned.
//!
//! ## get_command
//! [ConfigInit](../trait.ConfigInit.html) trait has the [get_command](../trait.ConfigInit.html#tymethod.get_command)
//! method that builds [Command](https://docs.rs/clap/latest/clap/struct.Command.html) that can initialize the structure. \
//! By using this method along with the [ClapSource::Matches](config_manager/enum.ClapSource.html#variant.Matches) option,
//! one can initialize the structure as a subcommand, so settings of the application and the configuration can be divided, like:
//! ```console
//! binary [APPLICATION'S SETTINGS] configuration [CONFIGURATION'S SETTINGS]
//! ```
//!
//! **Example**
//! ```
//! # use std::collections::HashSet;
//! use config_manager::*;
//! use clap::*;
//!
//! #[config(clap(name = "configuration", version, author))]
//! struct Config {
//! #[source(clap(long, short))]
//!     a: i32,
//! }
//!
//! fn init_from_app() -> Option<Config> {
//!     let app = Command::new("binary")
//!         .arg(Arg::new("some_field").long("field"))
//!         .subcommand(Config::get_command())
//!         .get_matches();
//!
//!     if let Some(subcommand) = app.subcommand_matches(Config::get_command().get_name()) {
//!         let opts = ConfigOption::ExplicitSource(Source::Clap(ClapSource::Matches(subcommand.clone())));
//!         Some(Config::parse_options(HashSet::from([opts])).unwrap())
//!     } else {
//!         None
//!     }
//! }
//! ```
//!
//! # Appendix
//! ## Clap attributes
//! All the attributes should be set in the form:
//! ```ignore
//! #[clap(attribute = value)]
//! ```
//! Where `value` is the code that should be passed to the corresponding clap method (or no value, if the attribute is a flag). For example:
//! ```ignore
//! #[clap(value_hint = ValueHint::Username, short_aliases = ['c', 'e'], long = "my_field", exclusive)]
//! field: usize,
//! ```
//! Full usage can be checked [in the crate tests](https://github.com/3xMike/config-manager/blob/main/tests/parse_method/get_command.rs)
//! ### Clap command
//! Documentation on original methods: [clap::Command]
//!
//! Next attributes are allowed: \
//! name, version, author, about, long_about, color, styles, term_width, max_term_width, disable_version_flag, next_line_help, disable_help_flag, disable_colored_help, help_expected, hide_possible_values, bin_name, display_name, after_help, after_long_help, before_help, before_long_help, long_version, override_usage, override_help, help_template, next_help_heading, next_display_order, allow_missing_positional, arg_required_else_help,
//! ### Clap arg
//! Documentation on original methods: [clap::Arg]
//!
//! Next attributes are allowed: \
//! help, long_help, short, long, flag, help_heading, alias, short_alias, aliases, short_aliases, visible_alias, visible_short_alias, visible_aliases, visible_short_aliases, index, last, requires, exclusive, value_name, value_hint, ignore_case, allow_hyphen_values, allow_negative_numbers, require_equals, display_order, next_line_help, hide, hide_possible_values, hide_default_value, hide_short_help, hide_long_help, conflicts_with, conflicts_with_all, overrides_with, overrides_with_all,
