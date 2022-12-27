// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use super::{attributes::*, format_to_tokens};
use crate::*;

pub(crate) struct AppTopLevelInfo {
    pub(crate) env_prefix: Option<String>,
    pub(crate) clap_app_info: NormalClapAppInfo,
    pub(crate) configs: ConfigFilesInfo,
    pub(crate) debug_cmd_input: Option<TokenStream>,
    pub(crate) table_name: Option<String>,
}

impl AppTopLevelInfo {
    pub(crate) fn extract(class_attrs: &[Attribute]) -> Self {
        Self {
            env_prefix: extract_env_prefix(class_attrs),
            clap_app_info: extract_clap_app(class_attrs),
            configs: extract_configs_info(class_attrs),
            debug_cmd_input: extract_debug_cmd_input(class_attrs),
            table_name: extract_table_name(class_attrs),
        }
    }
}

#[derive(Default, Clone)]
pub(crate) struct NormalClapAppInfo {
    pub(crate) name: String,
    pub(crate) version: Option<String>,
    pub(crate) author: Option<String>,
    pub(crate) about: Option<String>,
    pub(crate) long_about: Option<String>,
}

impl ToTokens for NormalClapAppInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend({
            let name = &self.name;
            let name = format_to_tokens!("clap::Command::new({name})");
            let version = match &self.version {
                None => TokenStream::new(),
                Some(version) => format_to_tokens!(".version({version})"),
            };
            let author = match &self.author {
                None => TokenStream::new(),
                Some(author) => format_to_tokens!(".author({author})"),
            };
            let about = match &self.about {
                None => TokenStream::new(),
                Some(about) => format_to_tokens!(".about({about})"),
            };
            let long_about = match &self.long_about {
                None => TokenStream::new(),
                Some(long_about) => format_to_tokens!(".long_about({long_about})"),
            };
            quote! {
                #name
                #version
                #author
                #about
                #long_about
            }
        })
    }
}

pub(crate) fn extract_clap_app(attrs: &[Attribute]) -> NormalClapAppInfo {
    attrs
        .iter()
        .find(|a| compare_attribute_name(a, CLAP_KEY))
        .map(|atr| match atr.parse_meta() {
            Err(err) => panic!("Can't parse attribute as meta: {err}"),
            Ok(meta) => match meta {
                Meta::List(clap_meta_list) => parse_clap_app_attribute(&clap_meta_list),
                _ => panic!("{CLAP_KEY} attribute must match #[{CLAP_KEY}(...)"),
            },
        })
        .unwrap_or_default()
        .normalize()
}

pub(crate) fn extract_env_prefix(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|a| compare_attribute_name(a, ENV_PREFIX_KEY))
        .map(|atr| match atr.parse_meta() {
            Err(err) => panic!("Can't parse attribute as meta: {err}"),
            Ok(meta) => match meta {
                Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(input_name),
                    ..
                }) => input_name.value(),
                _ => panic!("{ENV_PREFIX_KEY} must match #[{ENV_PREFIX_KEY} = \"...\"]"),
            },
        })
}

pub(crate) fn extract_debug_cmd_input(attrs: &[Attribute]) -> Option<TokenStream> {
    attrs
        .iter()
        .find(|a| compare_attribute_name(a, DEBUG_INPUT_KEY))
        .map(|atr| match atr.parse_meta() {
            Err(err) => panic!("Can't parse attribute as meta: {err}"),
            Ok(meta) => match meta {
                Meta::List(clap_meta_list) => clap_meta_list.nested.to_token_stream(),
                _ => panic!("{DEBUG_INPUT_KEY} attribute must match #[{DEBUG_INPUT_KEY}(...)"),
            },
        })
}

pub(crate) fn extract_table_name(attrs: &[Attribute]) -> Option<String> {
    attrs
        .iter()
        .find(|a| compare_attribute_name(a, TABLE_NAME_KEY))
        .map(|atr| match atr.parse_meta() {
            Err(err) => panic!("Can't parse attribute as meta: {err}"),
            Ok(meta) => match meta {
                Meta::NameValue(MetaNameValue {
                    lit: Lit::Str(input_name),
                    ..
                }) => input_name.value(),
                _ => panic!("{TABLE_NAME_KEY} must match #[{TABLE_NAME_KEY} = \"...\"]"),
            },
        })
}
