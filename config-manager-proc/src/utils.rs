// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

pub(crate) mod attributes;
pub(crate) mod config;
pub(crate) mod field;
pub(crate) mod parser;
pub(crate) mod top_level;

/// Formated string to TokenStream \
/// Same as ```TokenStream::from_str(&format!(...)).unwrap()```
macro_rules! format_to_tokens {
    ($($arg:tt)*) => {
        TokenStream::from_str(&std::format!($($arg)*)).unwrap()
    };
}
// TODO: rm this
// TODO: use quote_spanned everywhere

macro_rules! meta_value_lit {
    ($($arg:tt)*) => {
        syn::MetaNameValue {
            value: syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str($($arg)*),
                ..
            }),
            ..
        }
    };
}

macro_rules! panic_site {
    ($($message:tt)*) => {
        return Err(crate::Error::new(
            proc_macro2::Span::call_site(),
            format!($($message)*),
        ))
    };
}
macro_rules! panic_span {
    ($span: expr, $($message:tt)*) => {
        return Err(crate::Error::new($span, format!($($message)*)))
    };
}

pub(crate) use format_to_tokens;
pub(crate) use meta_value_lit;
pub(crate) use panic_site;
pub(crate) use panic_span;

pub(crate) fn str_to_tokens<S: AsRef<str>>(s: S, span: Span) -> TokenStream {
    let unprefixed = s.as_ref().strip_prefix('\"').unwrap_or_else(|| s.as_ref());
    let s = unprefixed.strip_suffix('\"').unwrap_or(unprefixed);
    LitStr::new(s, span).to_token_stream()
}

pub(crate) fn option_to_tokens(opt: &Option<TokenStream>) -> TokenStream {
    match opt {
        None => quote!(::std::option::Option::None),
        Some(pref) => {
            quote_spanned!(pref.span()=> ::std::option::Option::<::std::string::String>::Some(#pref.to_string()))
        }
    }
}

pub(crate) trait PanicOnNone {
    type Output;
    fn err_on_none<S: AsRef<str>>(self, span: Span, message: S) -> Result<Self::Output>;
}
impl<T> PanicOnNone for Option<T> {
    type Output = T;
    fn err_on_none<S: AsRef<str>>(self, span: Span, message: S) -> Result<T> {
        self.ok_or_else(|| Error::new(span, message.as_ref()))
    }
}
