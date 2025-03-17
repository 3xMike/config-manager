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

#[derive(Clone)]
pub(crate) struct ClapFieldParseResult {
    pub(crate) span: Span,

    pub(crate) docs: Option<String>,
    pub(crate) long: MaybeString,
    pub(crate) short: MaybeString,
    pub(crate) help: MaybeString,
    pub(crate) long_help: MaybeString,
    pub(crate) help_heading: Option<TokenStream>,
    pub(crate) flag: bool,
}

impl ClapFieldParseResult {
    pub(crate) fn new(span: Span) -> Self {
        Self {
            span,
            docs: Default::default(),
            long: Default::default(),
            short: Default::default(),
            help: Default::default(),
            long_help: Default::default(),
            help_heading: Default::default(),
            flag: false,
        }
    }

    fn get_docs<S: AsRef<str>>(&self, attr_name: S) -> Result<TokenStream> {
        match self.docs.clone() {
            Some(val) => Ok(str_to_tokens(val, self.span)),
            None => Err(Error::new(
                self.span,
                format!("if clap({}) is used without value, field docs must be provided. But there are no docs", attr_name.as_ref())
            )),
        }
    }
}

impl ClapFieldParseResult {
    pub(crate) fn normal_long(&self, field_name: &str) -> TokenStream {
        match &self.long {
            ClapOption::None | ClapOption::Empty => str_to_tokens(field_name, self.span),
            ClapOption::Some(long) => long.clone(),
        }
    }

    pub(crate) fn normalize(self, field_name: &str) -> Result<NormalClapFieldInfo> {
        Ok(NormalClapFieldInfo {
            span: self.span,
            help: self.help.clone().on_empty_res(|| self.get_docs("help"))?,
            long_help: self
                .long_help
                .clone()
                .on_empty_res(|| self.get_docs("long_help"))?,
            long: self.normal_long(field_name),
            short: self.short.on_empty_res(|| {
                field_name
                    .chars()
                    .next()
                    .ok_or_else(|| {
                        Error::new(self.span, "empty clap(short) is forbidden for config files")
                    })
                    .map(|c| LitChar::new(c, self.span).to_token_stream())
            })?,
            help_heading: self.help_heading,
            flag: self.flag,
        })
    }
}

pub(crate) struct ClapAppParseResult {
    pub(crate) span: Span,

    pub(crate) docs: Option<String>,
    pub(crate) name: Option<TokenStream>,
    pub(crate) version: MaybeString,
    pub(crate) author: MaybeString,
    pub(crate) about: MaybeString,
    pub(crate) long_about: MaybeString,
}

impl ClapAppParseResult {
    pub(crate) fn new(span: Span) -> Self {
        Self {
            span,
            docs: Default::default(),
            name: Default::default(),
            version: Default::default(),
            author: Default::default(),
            about: Default::default(),
            long_about: Default::default(),
        }
    }

    pub(crate) fn normalize(self) -> Result<NormalClapAppInfo> {
        Ok(NormalClapAppInfo {
            long_about: self
                .long_about
                .clone()
                .on_empty_res(|| self.get_docs("long_about"))?,
            name: match self.name {
                None => {
                    quote_spanned!(self.span=> ::config_manager::__private::clap::crate_name!())
                }
                Some(name) => name,
            },
            version: self.version.on_empty(
                || quote_spanned!(self.span=>  ::config_manager::__private::clap::crate_version!()),
            ),
            author: self.author.on_empty(|| {
                quote_spanned! {self.span=>
                        ::config_manager::__private::clap::crate_authors!("\n")
                }
            }),
            about: self
                .about
                .on_empty(|| quote_spanned!(self.span=> ::config_manager::__private::clap::crate_description!())),
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

fn meta_to_maybe(meta: &Meta) -> Result<MaybeString> {
    Ok(match_literal_or_init_from(meta, AcceptedLiterals::String)?
        .map(ClapOption::Some)
        .unwrap_or(ClapOption::Empty))
}

pub(crate) fn parse_clap_app_attribute(
    attributes: &MetaList,
    docs: Option<String>,
) -> Result<ClapAppParseResult> {
    let mut res = ClapAppParseResult::new(attributes.span());
    res.docs = docs;

    let attrs = &attributes.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for attr in attrs {
        match path_to_string(attr.path()).as_str() {
            "name" => {
                res.name = meta_to_option(attr)?;
            }
            "version" => res.version = meta_to_maybe(attr)?,
            "author" => res.author = meta_to_maybe(attr)?,
            "about" => res.about = meta_to_maybe(attr)?,
            "long_about" => res.long_about = meta_to_maybe(attr)?,
            other => panic_span!(
                attr.span(),
                "clap attibute \"{other}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_APP_ATTRS
            ),
        };
    }

    Ok(res)
}

pub(crate) fn parse_clap_field_attribute(
    attributes: &MetaList,
    is_bool: bool,
) -> Result<ClapFieldParseResult> {
    let mut res = ClapFieldParseResult::new(attributes.span());

    let attrs = &attributes.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for attr in attrs {
        match path_to_string(attr.path()).as_str() {
            "long" => res.long = meta_to_maybe(attr)?,
            "short" => {
                res.short = match_literal_or_init_from(attr, AcceptedLiterals::Char)?
                    .map(ClapOption::Some)
                    .unwrap_or(ClapOption::Empty)
            }
            "help" => res.help = meta_to_maybe(attr)?,
            "long_help" => res.long_help = meta_to_maybe(attr)?,
            "help_heading" => {
                res.help_heading = match attr {
                    Meta::Path(_) => {
                        panic_span!(attr.span(), "help_heading attribute can't be path")
                    }
                    other => meta_to_option(other)?,
                }
            }
            "flag" => {
                if !is_bool {
                    panic_span!(attr.span(), "Only boolean arguments can be flags")
                }
                if !matches!(attr, Meta::Path(_)) {
                    panic_span!(attr.span(), "flag attribute can't take any value(s)")
                }
                res.flag = true
            }
            other => panic_span!(
                attr.span(),
                "clap attibute \"{other}\" is not supported, allowed attrs: {:?}",
                ALLOWED_CLAP_FIELD_ATTRS
            ),
        };
    }

    Ok(res)
}
