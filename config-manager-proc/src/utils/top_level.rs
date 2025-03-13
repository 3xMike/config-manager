// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use super::{attributes::*, format_to_tokens, meta_value_lit};
use crate::utils::field::utils::{ExtractedAttributes, FieldAttribute};
use crate::*;

pub(crate) struct AppTopLevelInfo {
    pub(crate) env_prefix: Option<String>,
    pub(crate) clap_app_info: NormalClapAppInfo,
    pub(crate) configs: ConfigFilesInfo,
    pub(crate) debug_cmd_input: Option<TokenStream>,
    pub(crate) table_name: Option<String>,
    pub(crate) default_order: Option<ExtractedAttributes>,
}

impl AppTopLevelInfo {
    pub(crate) fn extract(crate_attrs: &[Meta], docs: Option<String>) -> Result<Self> {
        Ok(Self {
            env_prefix: extract_env_prefix(crate_attrs),
            clap_app_info: extract_clap_app(crate_attrs, docs)?,
            configs: extract_configs_info(crate_attrs)?,
            debug_cmd_input: extract_debug_cmd_input(crate_attrs),
            table_name: extract_table_name(crate_attrs),
            default_order: extract_source_order(crate_attrs),
        })
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

pub(crate) fn extract_clap_app(attrs: &[Meta], docs: Option<String>) -> Result<NormalClapAppInfo> {
    attrs
        .iter()
        .find(|a| a.path().is_ident(CLAP_KEY))
        .map(|meta| {
            let list = meta
                .require_list()
                .expect("clap attribute must match #[clap(...)");
            parse_clap_app_attribute(list, docs)
        })
        .unwrap_or_else(|| Ok(ClapAppParseResult::new(Span::call_site())))?
        .normalize()
}

pub(crate) fn extract_env_prefix(attrs: &[Meta]) -> Option<String> {
    match attrs.iter().find(|a| a.path().is_ident(ENV_PREFIX_KEY)) {
        None => Some(String::new()),
        Some(meta) => match meta {
            Meta::Path(_) => None,
            Meta::NameValue(meta_value_lit!(input_name)) => Some(input_name.value()),
            _ => panic!(
                "{ENV_PREFIX_KEY} must match #[{ENV_PREFIX_KEY} = \"...\"] or \
                     #[{ENV_PREFIX_KEY}]"
            ),
        },
    }
}

pub(crate) fn extract_debug_cmd_input(attrs: &[Meta]) -> Option<TokenStream> {
    let meta = attrs.iter().find(|a| a.path().is_ident(DEBUG_INPUT_KEY))?;

    Some(
        meta.require_list()
            .unwrap_or_else(|_| {
                panic!("{DEBUG_INPUT_KEY} attribute must match #[{DEBUG_INPUT_KEY}(...)")
            })
            .tokens
            .clone(),
    )
}

pub(crate) fn extract_table_name(attrs: &[Meta]) -> Option<String> {
    let meta = attrs.iter().find(|a| a.path().is_ident(TABLE_NAME_KEY))?;

    match meta {
        Meta::NameValue(meta_value_lit!(lit_str)) => Some(lit_str.value()),
        _ => panic!("{TABLE_NAME_KEY} must match #[{TABLE_NAME_KEY} = \"...\"]"),
    }
}

pub(crate) fn extract_source_order(attrs: &[Meta]) -> Option<ExtractedAttributes> {
    let meta = attrs.iter().find(|m| m.path().is_ident(SOURCE_ORDER_KEY))?;

    let list = meta
        .require_list()
        .unwrap_or_else(|_| panic!("{SOURCE_ORDER_KEY} must match #[{SOURCE_ORDER_KEY}(...)]"));
    let nested = list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .unwrap();

    let mut res = ExtractedAttributes::default();
    for meta in nested {
        let p = meta.require_path_only().unwrap_or_else(|_| {
            panic!(
                "default_order nested attributes can be on of: {CLAP_KEY}, \
                                 {ENV_KEY}, {CONFIG_KEY} and {DEFAULT}"
            )
        });
        match path_to_string(p).as_str() {
            CLAP_KEY => res
                .variables
                .push(FieldAttribute::Clap(ClapFieldParseResult::new(p.span()))),
            ENV_KEY => res.variables.push(FieldAttribute::Env(Default::default())),
            CONFIG_KEY => res
                .variables
                .push(FieldAttribute::Config(Default::default())),
            DEFAULT => res.default = Some(crate::utils::field::utils::Default::default()),
            other => panic!(
                "Error in \"{other}\" attribute: only {CLAP_KEY}, {ENV_KEY}, \
                                     {CONFIG_KEY} and {DEFAULT} are allowed as default_order \
                                     nested attribute"
            ),
        };
    }
    Some(res)
}
