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

#[derive(Default, Clone)]
pub(crate) struct ClapFieldParseResult {
    pub(crate) long: MaybeString,
    pub(crate) short: MaybeString,
    pub(crate) help: Option<String>,
    pub(crate) long_help: Option<String>,
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
            long: match self.long {
                ClapOption::None | ClapOption::Empty => format!("\"{field_name}\""),
                ClapOption::Some(long) => long,
            },
            short: match self.short {
                ClapOption::None => None,
                ClapOption::Empty => Some(
                    field_name
                        .chars()
                        .next()
                        .expect("empty clap(short) is forbidden for config files")
                        .to_token_stream()
                        .to_string(),
                ),
                ClapOption::Some(short) => Some(short),
            },
            help: self.help,
            long_help: self.long_help,
        }
    }
}

#[derive(Default)]
pub(crate) struct ClapAppParseResult {
    pub(crate) name: Option<String>,
    pub(crate) version: MaybeString,
    pub(crate) author: MaybeString,
    pub(crate) about: MaybeString,
    pub(crate) long_about: Option<String>,
}

impl ClapAppParseResult {
    pub(crate) fn normalize(self) -> NormalClapAppInfo {
        NormalClapAppInfo {
            name: match self.name {
                None => "::config_manager::__private::clap::crate_name!()".to_string(),
                Some(name) => name,
            },
            version: match self.version {
                ClapOption::None => None,
                ClapOption::Empty => {
                    Some("::config_manager::__private::clap::crate_version!()".to_string())
                }
                ClapOption::Some(version) => Some(version),
            },
            author: match self.author {
                ClapOption::None => None,
                ClapOption::Empty => {
                    Some("::config_manager::__private::clap::crate_authors!(\"\\n\")".to_string())
                }
                ClapOption::Some(author) => Some(author),
            },
            about: match self.about {
                ClapOption::None => None,
                ClapOption::Empty => {
                    Some("::config_manager::__private::clap::crate_description!()".to_string())
                }
                ClapOption::Some(about) => Some(about),
            },
            long_about: self.long_about,
        }
    }
}

fn meta_to_maybe(meta: &Meta) -> MaybeString {
    match_literal_or_init_from(meta, AcceptedLiterals::StringOnly)
        .map(|value| ClapOption::Some(value.as_string()))
        .unwrap_or(ClapOption::Empty)
}

fn meta_to_option(meta: &Meta) -> Option<String> {
    Some(
        match_literal_or_init_from(meta, AcceptedLiterals::StringOnly)
            .as_ref()
            .map(InitFrom::as_string)
            .unwrap_or_else(|| panic!("{} attribute can't be empty", path_to_string(meta.path()))),
    )
}

pub(crate) fn parse_clap_app_attribute(attributes: &MetaList) -> ClapAppParseResult {
    let attrs = &attributes.nested;
    let mut res = ClapAppParseResult::default();

    attrs.iter().for_each(|atr| match atr {
        NestedMeta::Lit(_) => panic!("clap attribute can't be a literal"),
        NestedMeta::Meta(atr) => match path_to_string(atr.path()).as_str() {
            "name" => {
                res.name = match atr {
                    Meta::Path(_) => panic!("long_about attribute can't be path"),
                    other => meta_to_option(other),
                }
            }
            "version" => res.version = meta_to_maybe(atr),
            "author" => res.author = meta_to_maybe(atr),
            "about" => res.about = meta_to_maybe(atr),
            "long_about" => {
                res.long_about = match atr {
                    Meta::Path(_) => panic!("long_about attribute can't be path"),
                    other => meta_to_option(other),
                }
            }
            other => panic!(
                "clap attibute \"{other}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_APP_ATTRS
            ),
        },
    });

    res
}

pub(crate) fn parse_clap_field_attribute(attributes: &MetaList) -> ClapFieldParseResult {
    let attrs = &attributes.nested;
    let mut res = ClapFieldParseResult::default();

    attrs.iter().for_each(|atr| match atr {
        NestedMeta::Lit(_) => panic!("clap attribute can't be a literal"),
        NestedMeta::Meta(atr) => match path_to_string(atr.path()).as_str() {
            "long" => res.long = meta_to_maybe(atr),
            "short" => {
                res.short = match_literal_or_init_from(atr, AcceptedLiterals::CharOnly)
                    .map(|value| ClapOption::Some(value.as_string()))
                    .unwrap_or(ClapOption::Empty)
            }
            "help" => {
                res.help = match atr {
                    Meta::Path(_) => panic!("help attribute can't be path"),
                    other => meta_to_option(other),
                }
            }
            "long_help" => {
                res.long_help = match atr {
                    Meta::Path(_) => panic!("long_help attribute can't be path"),
                    other => meta_to_option(other),
                }
            }
            other => panic!(
                "clap attibute \"{other}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_FIELD_ATTRS
            ),
        },
    });

    res
}
