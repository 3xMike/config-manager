// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

//! > **Crate to build config from environment, command line and files**
//! # Motivation
//! Non-runtime data generally comes to a project from
//! command line, environment and configuration files.\
//! Sometimes it comes from each of the sources simultaneously,
//! so all of them must be handled.\
//! None of the popular crates (including [clap](https://docs.rs/clap/latest/clap/) and [config](https://docs.rs/config/latest/config/))
//! can't handle all 3 together, so this crate has been created to solve this problem.
//!
//! # Basis
//! The Core of the crate is an attribute-macro [config](attr.config.html). \
//! Annotate structure with this macro and a field of it with the `source` attribute,
//! so the field will be searched in one of the provided sources. The sources can be provided by using the following nested `source` attributes: \
//! 1. `clap`: command line argument
//! 2. `env`: environment variable
//! 3. `config`: configuration file key
//! 4. `default`: default value
//!
//! **Example**
//! ```
//! use config_manager::config;
//!
//! #[config]
//! struct ApplicationConfig {
//!     #[source(clap(long, short = 'p'), env = "APP_MODEL_PATH", config)]
//!     model_path: String,
//!     #[source(env, config, default = 0)]
//!     prediction_delay: u64,
//! }
//! ```
//! In the example above, to set the value of the `model_path` field, a user may provide:
//! - command line argument `--model_path`
//! - environment variable named `model_path`
//! - configuration file containing field `model_path`
//!
//! If the value is found in multiple provided sources, the value will be assigned according to the provided order
//! (the order for the `model_path` field is `clap -> env -> config` and `env -> config -> default` for the `prediction_delay`). \
//! If none of them (including the default value) is found, the program returns error `MissingArgument`.
//!
//! **Note:** the default value is always assigned last.
//!
//! # Attributes documentation
//! For further understanding of project syntax and features, it is recommended to visit [Cookbook](__cookbook).
//!
//! # Complex example
//! ```no_run
#![doc = include_str!("../examples/src/demo.rs")]
//! ```
//! Run in [the repository](https://gitlab.kryptodev.ru/dev/research/rust/config-manager)
//! ```console
//! cargo run --package examples --bin demo -- --config="examples/config.toml" --a=5
//! ```
//! Result must be:
//! ```console
//! [examples/src/demo.rs:34] &*CFG = MethodConfig {
//!     a: 5,
//!     b: "qwerty",
//!     c: 165,
//!     d: {},
//! }
//! ```

use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;

pub use config_manager_proc::config;
pub use config_manager_proc::Flatten;
pub mod __cookbook;
#[doc(hidden)]
pub mod __private;
#[doc(hidden)]
mod utils;

/// Runtime initializing error.
#[derive(Debug)]
pub enum Error {
    MissingArgument(String),
    FailedParse(String),
    ExternalError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::MissingArgument(msg) | Error::FailedParse(msg) | Error::ExternalError(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

impl std::error::Error for Error {}
/// Config trait that constructs an instance of itself from
/// environment, command line and configuration files. \
///
/// Don't implement the trait manually,
/// invoking `#[config]` is the only correct way to derive this trait.
pub trait ConfigInit {
    /// Takes all the environment and tries to build an instance according to the structure attributes.
    fn parse() -> Result<Self, Error>
    where
        Self: Sized,
    {
        Self::parse_options(HashSet::new())
    }

    /// Takes all the environment and tries to build an instance according to the options and the structure attributes.
    fn parse_options(options: ConfigOptions) -> Result<Self, Error>
    where
        Self: Sized;

    /// Build `clap::Command` that can initialize the annotated struct.
    fn get_command() -> clap::Command;
}

/// Set of rules to build an instance of the annotated by `#[config]` structure.
pub type ConfigOptions = HashSet<ConfigOption>;
/// Allowed formats for the configuration files.
///
/// **Note:** `Ini` format is not supported.
pub type FileFormat = config::FileFormat;

/// Settings to build an instance of a struct, implementing `ConfigInit`.
///
/// **Note** Each option takes precedence over the corresponding structure attribute (see [cookbook](__cookbook/index.html) for more information).
#[derive(Debug, Clone)]
pub enum ConfigOption {
    /// Prefix of the environment variables.
    EnvPrefix(String),
    /// Replacement of the usual source.
    ExplicitSource(Source),
}

/// Replacement of the usual source to find values for the fields.
#[derive(Debug, Clone)]
pub enum Source {
    /// Configuration files.
    ///
    /// **Note:** It is allowed to specify multiple files: all of them will be merged.
    /// If there is a collision (the values of a particular key have been specified in two or more files),
    /// the value will be assigned from the file that has been described later.
    ConfigFiles(Vec<FileOptions>),
    /// Command line source.
    Clap(ClapSource),
    /// Map that replaces the enviromnent (fields, annotated with #[source(env)] will be searched in this map).
    ///
    /// Can be useful in testing.
    Env(HashMap<String, String>),
}

/// Replacement of the command line source.
#[derive(Debug, Clone)]
pub enum ClapSource {
    /// Same as ClapSource::Args(Vec::new()).
    None,
    /// Values of the command line source will be got from the passed arguments (like they were the command line arguments).
    ///
    /// Can be useful in testing.
    Args(Vec<String>),
    /// Values of the command line source will be got from the passed ArgMatches.
    ///
    /// Can be useful if the configuration is a subcommand of the main programm.
    Matches(::clap::ArgMatches),
}

/// Description of the configuration file.
#[derive(Debug, Clone)]
pub struct FileOptions {
    /// File format.
    pub format: FileFormat,
    /// Path to the file.
    pub path: String,
}
