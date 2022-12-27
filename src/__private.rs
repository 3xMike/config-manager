// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

//! # Private module
//!
//! There is some public API, because
//! code generated in the user's environment has to
//! pull some dependencies. \
//! It is highly discouraged to use this API manually.

pub use clap;
pub use config;
pub use ctor;
pub use deser_hjson;
pub use serde;
pub use serde_json;

pub use config_manager_proc::__Config__;

use super::utils;
use std::collections::HashMap;

pub struct EnvData {
    inner: HashMap<String, String>,
}

impl EnvData {
    pub fn get<'a, Q: Into<&'a str>>(&self, k: Q) -> Option<&String> {
        self.inner.get(&k.into().to_lowercase())
    }

    pub fn from(inner: HashMap<String, String>) -> Self {
        Self { inner }
    }
}

pub trait Flatten {
    fn get_args() -> Vec<clap::Arg>;
    fn parse(
        env_data: &EnvData,
        config_file_data: &HashMap<String, config::Value>,
        clap_data: &clap::ArgMatches,
        env_prefix: Option<String>,
    ) -> Result<Self, super::Error>
    where
        Self: Sized;
}

pub fn parse_subcommand<T>(
    args: impl Iterator<Item = String>,
    am: &clap::ArgMatches,
) -> Result<Option<T>, crate::Error>
where
    T: clap::Subcommand,
{
    utils::parse_subcommand(args, am)
}

pub fn find_field_in_table(
    config: &HashMap<String, config::Value>,
    table: Option<String>,
    field_name: String,
) -> Result<Option<String>, crate::Error> {
    utils::find_field_in_table(config, table, field_name)
}
