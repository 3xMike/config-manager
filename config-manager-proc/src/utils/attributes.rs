// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use strum_macros::EnumIter;

pub(crate) const CLAP_KEY: &str = "clap";
pub(crate) const ENV_KEY: &str = "env";
pub(crate) const CONFIG_KEY: &str = "config";
pub(crate) const DESERIALIZER: &str = "deserialize_with";
pub(crate) const DEFAULT: &str = "default";
pub(crate) const ENV_PREFIX_KEY: &str = "env_prefix";
pub(crate) const SOURCE_KEY: &str = "source";
pub(crate) const CONFIG_FILE_KEY: &str = "file";
pub(crate) const DEBUG_INPUT_KEY: &str = "__debug_cmd_input__";
pub(crate) const TABLE_NAME_KEY: &str = "table";
pub(crate) const SOURCE_ORDER_KEY: &str = "default_order";
pub(crate) const FLATTEN: &str = "flatten";
pub(crate) const SUBCOMMAND: &str = "subcommand";

pub(super) const ALLOWED_CLAP_APP_ATTRS: &[&str] =
    &["name", "version", "author", "about", "long_about"];
pub(super) const ALLOWED_CLAP_FIELD_ATTRS: &[&str] = &["help", "long_help", "short", "long"];

#[derive(EnumIter, strum_macros::Display)]
pub(super) enum TopLevelAttr {
    #[strum(serialize = "default")]
    Default,
    #[strum(serialize = "env")]
    Env,
    #[strum(serialize = "config")]
    Config,
    #[strum(serialize = "deserialize_with")]
    Deserializer,
}

#[derive(EnumIter, strum_macros::Display, Copy, Clone)]
pub(super) enum ClapNestedAttr {
    #[strum(serialize = "short")]
    Short,
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "help")]
    Help,
}

#[derive(EnumIter, strum_macros::Display)]
pub(super) enum ClapAppAttr {
    #[strum(serialize = "author")]
    Author,
    #[strum(serialize = "about")]
    About,
    #[strum(serialize = "version")]
    Version,
    #[strum(serialize = "override_help")]
    Help,
}

#[derive(EnumIter, strum_macros::Display)]
pub(super) enum ConfigFileAttr {
    #[strum(serialize = "format")]
    Format,
    #[strum(serialize = "env")]
    EnvKey,
    #[strum(serialize = "optional")]
    Optional,
    #[strum(serialize = "default")]
    Default,
}
