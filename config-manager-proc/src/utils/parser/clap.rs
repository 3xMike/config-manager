// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use std::collections::HashMap;

use super::{super::attributes::*, *};
use crate::*;

#[derive(Default, Clone)]
pub(crate) enum ClapOption<T> {
    #[default]
    None,
    Empty,
    Some(T),
}

type MaybeString = ClapOption<TokenStream>;

impl<T> ClapOption<T> {
    fn on_empty<F: FnOnce() -> T>(self, alternative: F) -> Option<T> {
        match self {
            ClapOption::None => None,
            ClapOption::Some(v) => Some(v),
            ClapOption::Empty => Some(alternative()),
        }
    }

    fn on_empty_res<F: FnOnce() -> Result<T>>(self, alternative: F) -> Result<Option<T>> {
        match self {
            ClapOption::None => Ok(None),
            ClapOption::Some(v) => Ok(Some(v)),
            ClapOption::Empty => Some(alternative()).transpose(),
        }
    }
}

fn meta_to_maybe(meta: &Meta) -> Result<MaybeString> {
    Ok(match_literal_or_init_from(meta, AcceptedLiterals::String)?
        .map(ClapOption::Some)
        .unwrap_or(ClapOption::Empty))
}

pub(crate) struct ClapAppParseResult {
    pub(crate) span: Span,

    pub(crate) docs: Option<String>,
    pub(crate) attributes: HashMap<String, MaybeString>,
}

impl ClapAppParseResult {
    pub(crate) fn new(span: Span) -> Self {
        Self {
            span,
            docs: Default::default(),
            attributes: ALLOWED_CLAP_APP_ATTRS
                .iter()
                .map(|name| (name.to_string(), MaybeString::None))
                .collect(),
        }
    }
}

pub(crate) fn parse_clap_app_attribute(
    attributes: &MetaList,
    docs: Option<String>,
) -> Result<ClapAppParseResult> {
    let mut res = ClapAppParseResult::new(attributes.span());
    res.docs = docs;

    let attrs = &attributes.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for attr in attrs {
        let attr_name = path_to_string(attr.path());
        if ALLOWED_CLAP_APP_ATTRS.contains(&attr_name.as_str()) {
            let not_set = res.attributes.get_mut(&attr_name).unwrap();
            if !matches!(not_set, ClapOption::None) {
                panic_span!(attr.span(), "trying to set clap({attr_name}) twice");
            }
            *not_set = meta_to_maybe(attr)?;
        } else {
            panic_span!(
                attr.span(),
                "clap attibute \"{attr_name}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_APP_ATTRS
            )
        }
    }

    Ok(res)
}

impl ClapAppParseResult {
    pub(crate) fn normalize(mut self) -> Result<NormalClapAppInfo> {
        let span = self.span;
        let name = match self.attributes.remove("name").unwrap() {
            ClapOption::Empty | ClapOption::None => {
                quote_spanned!(span=> ::config_manager::__private::clap::crate_name!())
            }
            ClapOption::Some(n) => n,
        };
        let mut attributes = HashMap::new();
        // Ungeneric ones
        let version = self.attributes.remove("version").unwrap();
        if let Some(v) = version.on_empty(
            || quote_spanned!(span=>  ::config_manager::__private::clap::crate_version!()),
        ) {
            attributes.insert("version".to_string(), v);
        }

        let long_about = self.attributes.remove("long_about").unwrap();
        if let Some(v) = long_about.on_empty_res(|| self.get_docs("long_about"))? {
            attributes.insert("long_about".to_string(), v);
        }

        let author = self.attributes.remove("author").unwrap();
        if let Some(v) = author.on_empty(|| {
            quote_spanned! {self.span=>
                    ::config_manager::__private::clap::crate_authors!("\n")
            }
        }) {
            attributes.insert("author".to_string(), v);
        }

        let about = self.attributes.remove("about").unwrap();
        if let Some(v) = about.on_empty(
            || quote_spanned!(self.span=> ::config_manager::__private::clap::crate_description!()),
        ) {
            attributes.insert("about".to_string(), v);
        }
        // Generic ones
        for (attr, val) in self.attributes {
            let flag = CLAP_FLAG_ATTRIBUTES.contains(&attr.as_str());
            match (val, flag) {
                (ClapOption::None, _) => (),
                (ClapOption::Empty, true) => {
                    attributes.insert(attr, quote_spanned!(span=> true));
                }
                (ClapOption::Empty, false) => panic_span!(
                    span,
                    "clap attribute \"{attr}\" must take value(s), can't be empty"
                ),
                (ClapOption::Some(v), true) => panic_span!(
                    v.span(),
                    "clap attribute \"{attr}\" can't take any value(s), it's a flag"
                ),
                (ClapOption::Some(v), false) => {
                    attributes.insert(attr, v);
                }
            }
        }
        Ok(NormalClapAppInfo {
            span,
            name,
            attributes,
        })
    }

    fn get_docs<S: AsRef<str>>(&self, attr_name: S) -> Result<TokenStream> {
        match self.docs.clone() {
            Some(val) => Ok(str_to_tokens(val, self.span)),
            None => Err(Error::new(
                self.span,
                format!("if clap({}) is used without value, struct docs must be provided. But there are no docs", attr_name.as_ref())
            )),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ClapFieldParseResult {
    pub(crate) span: Span,

    pub(crate) docs: Option<String>,
    pub(crate) attributes: HashMap<String, MaybeString>,
}

impl ClapFieldParseResult {
    pub(crate) fn new(span: Span) -> Self {
        Self {
            span,
            docs: Default::default(),
            attributes: ALLOWED_CLAP_FIELD_ATTRS
                .iter()
                .map(|name| (name.to_string(), MaybeString::None))
                .collect(),
        }
    }
}

pub(crate) fn parse_clap_field_attribute(
    attributes: &MetaList,
    is_bool: bool,
) -> Result<ClapFieldParseResult> {
    let mut res = ClapFieldParseResult::new(attributes.span());

    let attrs = &attributes.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for attr in attrs {
        let attr_name = path_to_string(attr.path());
        if ALLOWED_CLAP_FIELD_ATTRS.contains(&attr_name.as_str()) {
            let not_set = res.attributes.get_mut(&attr_name).unwrap();
            if !matches!(not_set, ClapOption::None) {
                panic_span!(attr.span(), "trying to set clap({attr_name}) twice");
            }

            if attr_name == "flag" && !is_bool {
                panic_span!(attr.span(), "Only boolean arguments can be flags")
            }

            *not_set = if attr_name == "short" {
                match_literal_or_init_from(attr, AcceptedLiterals::Char)?
                    .map(ClapOption::Some)
                    .unwrap_or(ClapOption::Empty)
            } else {
                meta_to_maybe(attr)?
            };
        } else {
            panic_span!(
                attr.span(),
                "clap attibute \"{attr_name}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_FIELD_ATTRS
            )
        }
    }

    Ok(res)
}

impl ClapFieldParseResult {
    fn get_docs<S: AsRef<str>>(&self, attr_name: S) -> Result<TokenStream> {
        match self.docs.clone() {
            Some(val) => Ok(str_to_tokens(val, self.span)),
            None => Err(Error::new(
                self.span,
                format!("if clap({}) is used without value, field docs must be provided. But there are no docs", attr_name.as_ref())
            )),
        }
    }

    pub(crate) fn normal_long(&self, field_name: &str) -> TokenStream {
        match self.attributes.get("long").unwrap() {
            ClapOption::Empty | ClapOption::None => str_to_tokens(field_name, self.span),
            ClapOption::Some(n) => n.clone(),
        }
    }

    pub(crate) fn has_explicit_long(&self) -> bool {
        matches!(self.attributes.get("long").unwrap(), ClapOption::Some(_))
    }

    pub(crate) fn normalize(mut self, field_name: &str) -> Result<NormalClapFieldInfo> {
        let span = self.span;
        let long = match self.attributes.remove("long").unwrap() {
            ClapOption::Empty | ClapOption::None => str_to_tokens(field_name, span),
            ClapOption::Some(n) => n,
        };
        let mut attributes = HashMap::new();
        // Ungeneric ones
        let help = self.attributes.remove("help").unwrap();
        if let Some(v) = help.on_empty_res(|| self.get_docs("help"))? {
            attributes.insert("help".to_string(), v);
        }

        let long_help = self.attributes.remove("long_help").unwrap();
        if let Some(v) = long_help.on_empty_res(|| self.get_docs("long_help"))? {
            attributes.insert("long_help".to_string(), v);
        }

        let short = self.attributes.remove("short").unwrap();
        if let Some(v) = short.on_empty_res(|| {
            field_name
                .chars()
                .next()
                .ok_or_else(|| {
                    Error::new(self.span, "empty clap(short) is forbidden for config files")
                })
                .map(|c| LitChar::new(c, self.span).to_token_stream())
        })? {
            attributes.insert("short".to_string(), v);
        }

        // Generic ones
        for (attr, val) in self.attributes {
            let flag = CLAP_FLAG_ATTRIBUTES.contains(&attr.as_str());
            match (val, flag) {
                (ClapOption::None, _) => (),
                (ClapOption::Empty, true) => {
                    attributes.insert(attr, quote_spanned!(span=> true));
                }
                (ClapOption::Empty, false) => panic_span!(
                    span,
                    "clap attribute \"{attr}\" must take value(s), can't be empty"
                ),
                (ClapOption::Some(v), true) => panic_span!(
                    v.span(),
                    "clap attribute \"{attr}\" can't take any value(s), it's a flag"
                ),
                (ClapOption::Some(v), false) => {
                    attributes.insert(attr, v);
                }
            }
        }

        Ok(NormalClapFieldInfo {
            span,
            long,
            attributes,
        })
    }
}
