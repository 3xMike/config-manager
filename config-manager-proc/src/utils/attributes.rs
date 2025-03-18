// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use strum_macros::EnumIter;
use syn::{Attribute, Meta};

use super::meta_value_lit;

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

pub(crate) const ALLOWED_CRATE_ATTRS: &[&str] = &[
    ENV_PREFIX_KEY,
    CONFIG_FILE_KEY,
    DEBUG_INPUT_KEY,
    TABLE_NAME_KEY,
    SOURCE_ORDER_KEY,
    CLAP_KEY,
];
pub(crate) const ALLOWED_FLATTEN_ATTRS: &[&str] = &[TABLE_NAME_KEY, SOURCE_ORDER_KEY];
pub(crate) const ALLOWED_CLAP_APP_ATTRS: &[&str] = &[
    "name",
    "version",
    "author",
    "about",
    "long_about",
    "color",
    "styles",
    "term_width",
    "max_term_width",
    "disable_version_flag",
    "next_line_help",
    "disable_help_flag",
    "disable_colored_help",
    "help_expected",
    "hide_possible_values",
    "bin_name",
    "display_name",
    "after_help",
    "after_long_help",
    "before_help",
    "before_long_help",
    "long_version",
    "override_usage",
    "override_help",
    "help_template",
    "next_help_heading",
    "next_display_order",
    "allow_missing_positional",
    "arg_required_else_help",
];

pub(crate) const ALLOWED_CLAP_FIELD_ATTRS: &[&str] = &[
    "help",
    "long_help",
    "short",
    "long",
    "flag",
    "help_heading",
    "alias",
    "short_alias",
    "aliases",
    "short_aliases",
    "visible_alias",
    "visible_short_alias",
    "visible_aliases",
    "visible_short_aliases",
    "index",
    "last",
    "requires",
    "exclusive",
    "value_name",
    "value_hint",
    "ignore_case",
    "allow_hyphen_values",
    "allow_negative_numbers",
    "require_equals",
    "display_order",
    "next_line_help",
    "hide",
    "hide_possible_values",
    "hide_default_value",
    "hide_short_help",
    "hide_long_help",
    "conflicts_with",
    "conflicts_with_all",
    "overrides_with",
    "overrides_with_all",
];

pub(crate) const CLAP_FLAG_ATTRIBUTES: &[&str] = &[
    // Command
    "disable_version_flag",
    "next_line_help",
    "disable_help_flag",
    "disable_colored_help",
    "help_expected",
    "hide_possible_values",
    "allow_missing_positional",
    "arg_required_else_help",
    // Arg
    "flag",
    "last",
    "exclusive",
    "ignore_case",
    "allow_hyphen_values",
    "allow_negative_numbers",
    "require_equals",
    "next_line_help",
    "hide",
    "hide_possible_values",
    "hide_default_value",
    "hide_short_help",
    "hide_long_help",
];

pub(crate) const CLAP_MULTIVALUES_FIELD_ATTRIBUTES: &[&str] = &[
    "aliases",
    "short_aliases",
    "visible_aliases",
    "visible_short_aliases",
    "conflicts_with_all",
    "overrides_with_all",
];
pub(crate) const CLAP_ATTRIBUTE_TAKES_CHAR: &[&str] =
    &["short", "short_aliases", "visible_short_aliases"];
pub(crate) const CLAP_ATTRIBUTE_TAKES_INT: &[&str] = &["index", "term_width", "max_term_width"];
pub(crate) const CLAP_ATTRIBUTE_TAKES_CODE: &[&str] = &["color", "styles"];
// todo: INDEX, aliases, short_aliases

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

pub(crate) fn extract_docs(attrs: &[Attribute]) -> Option<String> {
    let mut res = String::new();
    for attr in attrs {
        if !attr.meta.path().is_ident("doc") {
            continue;
        }
        if let Meta::NameValue(meta_value_lit!(str_lit)) = &attr.meta {
            res.push_str(&str_lit.value());
        }
    }
    if res.is_empty() {
        None
    } else {
        Some(res)
    }
}
