// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

#[derive(Copy, Clone)]
pub(crate) enum AcceptedLiterals {
    String,
    Char,
}

pub(crate) fn match_literal_or_init_from(
    attribute: &Meta,
    accepted_literals: AcceptedLiterals,
) -> Result<Option<TokenStream>> {
    match attribute {
        Meta::Path(_) => Ok(None),
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit { lit, .. }),
            ..
        }) => Ok(Some(match accepted_literals {
            AcceptedLiterals::String => {
                if matches!(lit, Lit::Str(_)) {
                    lit.to_token_stream()
                } else {
                    panic_span!(attribute.span(), "expected string, got {:#?}", lit);
                }
            }
            AcceptedLiterals::Char => {
                if matches!(lit, Lit::Char(_)) {
                    lit.to_token_stream()
                } else {
                    panic_span!(attribute.span(), "expected char, got {:#?}", lit);
                }
            }
        })),
        Meta::List(list) => {
            let args = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

            if !args[0].path().is_ident("init_from") {
                panic_span!(
                    attribute.span(),
                    "len of nested args must be exactly 1 and it must be \"init_from = ...\""
                )
            }

            match &args[0] {
                Meta::NameValue(expr) => {
                    let expr = &expr.value;
                    Ok(Some(quote_spanned!(attribute.span()=> {#expr})))
                }
                any => panic_span!(
                    attribute.span(),
                    "unexpected attribute type, must be string literal: {:#?}",
                    any
                ),
            }
        }
        other => panic_span!(
            attribute.span(),
            "Unknown attribute meta {}",
            other.to_token_stream()
        ),
    }
}

pub(crate) fn extract_default(meta: &Meta) -> Result<Option<TokenStream>> {
    match meta {
        Meta::Path(_) => Ok(None),
        Meta::List(_) => {
            panic_span!(
                meta.span(),
                "default attribute must be #[source(default = \"...\")] of #[source(default)]"
            )
        }
        Meta::NameValue(MetaNameValue { value, .. }) => {
            Ok(Some(quote_spanned!(meta.span()=> { #value })))
        }
    }
}

pub(crate) fn meta_to_option(meta: &Meta) -> Result<Option<TokenStream>> {
    match_literal_or_init_from(meta, AcceptedLiterals::String)
}
