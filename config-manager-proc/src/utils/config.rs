// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use std::collections::HashSet;

use strum::IntoEnumIterator;

use super::attributes::*;
use crate::*;

fn str_to_config_format_repr(s: &str) -> String {
    match s {
        "json" | "json5" | "toml" | "yaml" | "ron" => {
            let capitalize_first = |s: &str| -> String {
                let mut chars = s.chars();
                let first_char = chars.next().unwrap();
                first_char.to_uppercase().to_string() + &chars.collect::<String>()
            };

            let accepted_format = capitalize_first(s);
            let pref = "::config_manager::__private::config::FileFormat::".to_string();
            pref + &accepted_format
        }
        _ => panic!("{s} format is not supported"),
    }
}

struct ParsedConfigFileAttributes {
    file_format: String,
    clap_info: Option<NormalClapFieldInfo>,
    env_key: Option<String>,
    optional: bool,
    default: Option<String>,
}

fn handle_file_attributes(
    class_attributes: &[Attribute],
) -> impl Iterator<Item = ParsedConfigFileAttributes> + '_ {
    class_attributes
        .iter()
        .filter(|a| a.path().is_ident(CONFIG_FILE_KEY))
        .map(handle_file_attribute)
}

fn handle_file_attribute(attr: &Attribute) -> ParsedConfigFileAttributes {
    let nested = attr
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .expect(
            "config arguments must match file(format = \"...\", clap_key = \
                             \"...\", env = \"...\", default = \"...\", optional = \
                             true/[false])",
        );

    let mut clap_info = None;
    let mut config_file_attributes: Vec<_> = ConfigFileAttr::iter()
        .map(|ty| OptionalAttribute {
            value: None,
            accepted_literals: match ty {
                ConfigFileAttr::Optional => AcceptedLiterals::Bool,
                _ => AcceptedLiterals::String,
            },
            ty,
        })
        .collect();

    'next_arg: for arg in nested {
        if path_to_string(arg.path()) == CLAP_KEY {
            let clap_list = arg
                .require_list()
                .expect("clap attribute must match \"clap(...)\"");

            clap_info = Some(parse_clap_field_attribute(clap_list, false));
            continue 'next_arg;
        } else {
            for attr in &mut config_file_attributes {
                match try_set_optional_attribute::<ConfigFileAttr>(&arg, attr, false) {
                    SetOptionalAttrResult::NameMismatch => (),
                    SetOptionalAttrResult::Set => continue 'next_arg,
                    SetOptionalAttrResult::ErrorAlreadySet => {
                        panic!("attempted to set {} twice", attr.ty)
                    }
                }
            }
            panic!("unknown attribute: {:#?}", arg)
        }
    }

    let mut file_format = None;
    let mut env_key = None;
    let mut optional = false;
    let mut default = None;

    for attr in config_file_attributes {
        match attr.ty {
            ConfigFileAttr::EnvKey => env_key = attr.value,
            ConfigFileAttr::Optional => {
                if let Some(attr) = attr.value {
                    optional = attr.parse().unwrap()
                }
            }
            ConfigFileAttr::Default => {
                if let Some(attr) = attr.value {
                    default = Some(attr)
                }
            }
            ConfigFileAttr::Format => {
                if let Some(format_attr) = attr.value {
                    assert_eq!(format_attr.chars().next().unwrap(), '"');
                    assert_eq!(format_attr.chars().last().unwrap(), '"');

                    let drop_fst_and_lst: String = format_attr
                        .chars()
                        .skip(1)
                        .take(format_attr.len() - 2)
                        .collect();
                    file_format = Some(str_to_config_format_repr(&drop_fst_and_lst))
                }
            }
        }
    }

    if clap_info.is_none() && env_key.is_none() && default.is_none() {
        panic!("you must specify at least one of (clap, env, default)");
    }
    if let Some(clap_info) = &clap_info {
        if let ClapOption::None | ClapOption::Empty = clap_info.long {
            panic!(
                "if #[clap] attribute is specified for configuration file, nested \
                             `long = ...` must be provided"
            );
        }
    }

    let clap_info = clap_info.map(|info| info.normalize(Default::default()));

    ParsedConfigFileAttributes {
        default,
        optional,
        clap_info,
        env_key,
        file_format: file_format
            .unwrap_or_else(|| panic!("`format` attribute of config file must be set")),
    }
}

pub(crate) struct ConfigFilesInfo {
    pub(crate) configs_attributes: Vec<ConfigFileInfo>,
    pub(crate) configs_as_clap_args: Punctuated<ClapInitialization, Token![.]>,
}

pub(crate) struct ConfigFileInfo {
    pub(crate) file_format: TokenStream,
    pub(crate) clap_long: TokenStream,
    pub(crate) env_key: TokenStream,
    pub(crate) is_optional: bool,
    pub(crate) default_path: TokenStream,
}

pub(crate) fn extract_configs_info(class_attributes: &[Attribute]) -> ConfigFilesInfo {
    let mut configs_attributes = Vec::<ConfigFileInfo>::new();
    let mut configs_as_clap_args = Punctuated::new();
    let mut config_clap_keys = HashSet::<String>::new();
    let mut config_env_keys = HashSet::<String>::new();

    for ParsedConfigFileAttributes {
        file_format,
        clap_info,
        env_key,
        optional,
        default,
    } in handle_file_attributes(class_attributes)
    {
        if optional && default.is_some() {
            panic!(
                concat!(
                    "setting optional to true while specifying the default path is pointless, ",
                    "since the config file will always be tried to parse [default = {}]"
                ),
                default.unwrap()
            );
        }

        let clap_long = if let Some(clap_info) = &clap_info {
            let clap_long = clap_info.long.clone();
            let is_new = config_clap_keys.insert(clap_long.clone());
            if !is_new {
                panic!("config file with clap key {clap_long} already specified");
            }
            TokenStream::from_str(&format!("::std::option::Option::Some({clap_long})"))
        } else {
            TokenStream::from_str("::std::option::Option::<&::std::primitive::str>::None")
        }
        .unwrap();

        let env_key = if let Some(env_key) = env_key {
            let is_new = config_env_keys.insert(env_key.clone());
            if !is_new {
                panic!("config file with env key {env_key} already specified");
            }
            TokenStream::from_str(&format!("::std::option::Option::Some({env_key})"))
        } else {
            TokenStream::from_str("::std::option::Option::<&::std::primitive::str>::None")
        }
        .unwrap();

        let default_path = if let Some(default) = default {
            TokenStream::from_str(&format!("::std::option::Option::Some({default})"))
        } else {
            TokenStream::from_str("::std::option::Option::None")
        }
        .unwrap();

        let file_format = TokenStream::from_str(&file_format).unwrap();
        configs_attributes.push(ConfigFileInfo {
            file_format,
            clap_long,
            env_key,
            is_optional: optional,
            default_path,
        });

        if let Some(clap_info) = clap_info {
            configs_as_clap_args.push(ClapInitialization::Normal(clap_info));
        }
    }

    ConfigFilesInfo {
        configs_attributes,
        configs_as_clap_args,
    }
}
