// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use std::collections::HashSet;

use super::attributes::*;
use crate::*;

fn str_to_config_format_repr<S: AsRef<str>>(s: S, span: Span) -> Result<TokenStream> {
    let s = s.as_ref().trim_matches('"');
    match s {
        "json" | "json5" | "toml" | "yaml" | "ron" => {
            let mut chars = s.chars();
            let first_char = chars.next().unwrap();
            let accepted_format =
                first_char.to_uppercase().to_string() + &chars.collect::<String>();
            let accepted_format = Ident::new(&accepted_format, span);

            Ok(
                quote_spanned!(span=> ::config_manager::__private::config::FileFormat::#accepted_format),
            )
        }
        _ => panic_span!(span, "{s} format is not supported"),
    }
}

struct ParsedConfigFileAttributes {
    span: Span,
    file_format: TokenStream,
    clap_info: Option<NormalClapFieldInfo>,
    env_key: Option<TokenStream>,
    optional: bool,
    default: Option<TokenStream>,
}

fn handle_file_attributes(class_attributes: &[Meta]) -> Result<Vec<ParsedConfigFileAttributes>> {
    class_attributes
        .iter()
        .filter(|m| m.path().is_ident(CONFIG_FILE_KEY))
        .map(handle_file_attribute)
        .collect()
}

fn handle_file_attribute(attr: &Meta) -> Result<ParsedConfigFileAttributes> {
    let nested = attr
        .require_list()
        .map_err(|_| Error::new(attr.span(), "file attribute must match \"file(...)\""))?
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    let mut clap_info = None;
    let mut file_format = None;
    let mut env_key = None;
    let mut optional = false;
    let mut default = None;

    for arg in nested {
        match path_to_string(arg.path()).as_str() {
            "clap" => {
                if clap_info.is_some() {
                    panic_span!(arg.span(), "attempted to set clap twice")
                }
                let clap_list = arg.require_list().map_err(|_err| {
                    Error::new(arg.span(), "clap attribute must match \"clap(...)\"")
                })?;

                clap_info = Some(parse_clap_field_attribute(clap_list, false)?);
            }
            "format" => {
                let file_attr = set_config_attr(file_format.is_some(), &arg, "format")?.unwrap();
                file_format = Some(str_to_config_format_repr(
                    file_attr.to_string(),
                    arg.span(),
                )?);
            }
            "env" => env_key = set_config_attr(env_key.is_some(), &arg, "env")?,
            "optional" => {
                if optional {
                    panic_span!(arg.span(), "attempted to set optional twice")
                }
                if !matches!(arg, Meta::Path(_)) {
                    panic_span!(
                        arg.span(),
                        "optional cannot take values. Usage: file(optional)"
                    )
                }
                optional = true;
            }
            "default" => default = set_config_attr(default.is_some(), &arg, "default")?,
            other => panic_span!(arg.span(), "unknown attribute {other}"),
        }
    }

    if clap_info.is_none() && env_key.is_none() && default.is_none() {
        panic_span!(
            attr.span(),
            "you must specify at least one of (clap, env, default)"
        );
    }
    if let Some(clap_info) = &clap_info {
        if !clap_info.has_explicit_long() {
            panic_span!(
                clap_info.span,
                "if #[clap] attribute is specified for configuration file, nested `long = ...` \
                    must be provided. Otherwise it's impossible to determine arg name"
            );
        }
    }

    let clap_info = clap_info
        .map(|info| info.normalize(Default::default()))
        .transpose()?;

    Ok(ParsedConfigFileAttributes {
        span: attr.span(),
        default,
        optional,
        clap_info,
        env_key,
        file_format: file_format
            .err_on_none(attr.span(), "`format` attribute of config file must be set")?,
    })
}

fn set_config_attr<N: AsRef<str>>(
    already_set: bool,
    arg: &Meta,
    attr_name: N,
) -> Result<Option<TokenStream>> {
    let attr_name = attr_name.as_ref();
    if already_set {
        panic_span!(arg.span(), "attempted to set {attr_name} twice")
    }
    Ok(Some(meta_to_option(arg)?.err_on_none(
        arg.span(),
        format!("file({attr_name}) can't be empty"),
    )?))
}

pub(crate) struct ConfigFilesInfo {
    pub(crate) configs_attributes: Vec<ConfigFileInfo>,
    pub(crate) configs_as_clap_args: Punctuated<ClapInitialization, Token![.]>,
}

pub(crate) struct ConfigFileInfo {
    pub(crate) span: Span,
    pub(crate) file_format: TokenStream,
    pub(crate) clap_long: TokenStream,
    pub(crate) env_key: TokenStream,
    pub(crate) is_optional: bool,
    pub(crate) default_path: TokenStream,
}

pub(crate) fn extract_configs_info(class_attributes: &[Meta]) -> Result<ConfigFilesInfo> {
    let mut configs_attributes = Vec::<ConfigFileInfo>::new();
    let mut configs_as_clap_args = Punctuated::new();
    let mut config_clap_keys = HashSet::<String>::new();
    let mut config_env_keys = HashSet::<String>::new();

    for ParsedConfigFileAttributes {
        span,
        file_format,
        clap_info,
        env_key,
        optional,
        default,
    } in handle_file_attributes(class_attributes)?
    {
        if optional && default.is_some() {
            panic_span!(
                span,
                "setting optional to true while specifying the default path is pointless, \
                    since the config file will always be tried to parse"
            );
        }

        let clap_long = if let Some(clap_info) = &clap_info {
            let clap_long = clap_info.long.clone();
            let is_new = config_clap_keys.insert(clap_long.to_string());
            if !is_new {
                panic_span!(
                    span,
                    "config file with clap key {clap_long} already specified"
                );
            }

            quote_spanned!(span=> ::std::option::Option::Some(#clap_long))
        } else {
            quote_spanned!(span=> ::std::option::Option::<&::std::primitive::str>::None)
        };

        let env_key = if let Some(env_key) = env_key {
            let is_new = config_env_keys.insert(env_key.to_string());
            if !is_new {
                panic_span!(span, "config file with env key {env_key} already specified");
            }

            quote_spanned!(span=> ::std::option::Option::Some(#env_key))
        } else {
            quote_spanned!(span=> ::std::option::Option::<&::std::primitive::str>::None)
        };

        let default_path = if let Some(default) = default {
            quote_spanned!(span=> ::std::option::Option::Some(#default))
        } else {
            quote_spanned!(span=> ::std::option::Option::None)
        };

        configs_attributes.push(ConfigFileInfo {
            span,
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

    Ok(ConfigFilesInfo {
        configs_attributes,
        configs_as_clap_args,
    })
}
