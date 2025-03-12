// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use super::{super::attributes::*, *};
use crate::*;

#[derive(Default, Clone)]
pub(crate) enum ClapOption<T> {
    #[default]
    None,
    Empty,
    Some(T),
}

type MaybeString = ClapOption<String>;

impl<T> ClapOption<T> {
    fn on_empty<F: FnOnce() -> T>(self, alternative: F) -> Option<T> {
        match self {
            ClapOption::None => None,
            ClapOption::Some(v) => Some(v),
            ClapOption::Empty => Some(alternative()),
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct ClapFieldParseResult {
    pub(crate) docs: Option<String>,
    pub(crate) long: MaybeString,
    pub(crate) short: MaybeString,
    pub(crate) help: MaybeString,
    pub(crate) long_help: MaybeString,
    pub(crate) help_heading: Option<String>,
    pub(crate) flag: bool,
}

impl ClapFieldParseResult {
    pub(crate) fn normal_long(&self, field_name: &str) -> String {
        match &self.long {
            ClapOption::None | ClapOption::Empty => format!("\"{field_name}\""),
            ClapOption::Some(long) => long.clone(),
        }
    }

    pub(crate) fn normalize(self, field_name: &str) -> NormalClapFieldInfo {
        NormalClapFieldInfo {
            long: self.normal_long(field_name),
            short: self.short.on_empty(||field_name
                .chars()
                .next()
                .expect("empty clap(short) is forbidden for config files")
                .to_token_stream()
                .to_string()),
            help: self.help.on_empty(|| format!("\"{}\"", self.docs.clone().expect("if clap(help) is used without value, struct docs must be provided. But there are no docs"))),
            long_help: self.long_help.on_empty(|| format!("\"{}\"", self.docs.expect("if clap(long_help) is used without value, struct docs must be provided. But there are no docs"))),
            help_heading: self.help_heading,
            flag: self.flag,
        }
    }
}

#[derive(Default)]
pub(crate) struct ClapAppParseResult {
    pub(crate) name: Option<String>,
    pub(crate) version: MaybeString,
    pub(crate) author: MaybeString,
    pub(crate) about: MaybeString,
    pub(crate) long_about: MaybeString,
}

impl ClapAppParseResult {
    pub(crate) fn normalize(self, docs: Option<String>) -> NormalClapAppInfo {
        NormalClapAppInfo {
            name: match self.name {
                None => "::config_manager::__private::clap::crate_name!()".to_string(),
                Some(name) => name,
            },
            version: self.version.on_empty(|| "::config_manager::__private::clap::crate_version!()".to_string()),
            author: self.author.on_empty(|| "::config_manager::__private::clap::crate_authors!(\"\\n\")".to_string()),
            about: self.about.on_empty(|| "::config_manager::__private::clap::crate_description!()".to_string()),
            long_about: self.long_about.on_empty(|| format!("\"{}\"",docs.expect("if clap(long_about) is used without value, struct docs must be provided. But there are no docs"))),
        }
    }
}

fn meta_to_maybe(meta: &Meta) -> MaybeString {
    match_literal_or_init_from(meta, AcceptedLiterals::String)
        .map(|value| ClapOption::Some(value.as_string()))
        .unwrap_or(ClapOption::Empty)
}

fn meta_to_option(meta: &Meta) -> Option<String> {
    Some(
        match_literal_or_init_from(meta, AcceptedLiterals::String)
            .as_ref()
            .map(InitFrom::as_string)
            .unwrap_or_else(|| panic!("{} attribute can't be empty", path_to_string(meta.path()))),
    )
}

pub(crate) fn parse_clap_app_attribute(attributes: &MetaList) -> ClapAppParseResult {
    let mut res = ClapAppParseResult::default();

    let attrs = &attributes
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .unwrap();

    for attr in attrs {
        match path_to_string(attr.path()).as_str() {
            "name" => {
                res.name = match attr {
                    Meta::Path(_) => panic!("long_about attribute can't be path"),
                    other => meta_to_option(other),
                }
            }
            "version" => res.version = meta_to_maybe(attr),
            "author" => res.author = meta_to_maybe(attr),
            "about" => res.about = meta_to_maybe(attr),
            "long_about" => res.long_about = meta_to_maybe(attr),
            other => panic!(
                "clap attibute \"{other}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_APP_ATTRS
            ),
        };
    }

    res
}

pub(crate) fn parse_clap_field_attribute(
    attributes: &MetaList,
    is_bool: bool,
) -> ClapFieldParseResult {
    let mut res = ClapFieldParseResult::default();

    let attrs = &attributes
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .unwrap();

    for attr in attrs {
        match path_to_string(attr.path()).as_str() {
            "long" => res.long = meta_to_maybe(attr),
            "short" => {
                res.short = match_literal_or_init_from(attr, AcceptedLiterals::Char)
                    .map(|value| ClapOption::Some(value.as_string()))
                    .unwrap_or(ClapOption::Empty)
            }
            "help" => res.help = meta_to_maybe(attr),
            "long_help" => res.long_help = meta_to_maybe(attr),
            "help_heading" => {
                res.help_heading = match attr {
                    Meta::Path(_) => panic!("help_heading attribute can't be path"),
                    other => meta_to_option(other),
                }
            }
            "flag" => {
                if !is_bool {
                    panic!("Only boolean arguments can be flags")
                }
                if !matches!(attr, Meta::Path(_)) {
                    panic!("flag attribute can't take any value(s)")
                }
                res.flag = true
            }
            other => panic!(
                "clap attibute \"{other}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_FIELD_ATTRS
            ),
        };
    }

    res
}
