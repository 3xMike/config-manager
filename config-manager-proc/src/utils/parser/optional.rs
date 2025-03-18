// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

#[derive(Copy, Clone)]
pub(crate) enum AcceptedLiterals {
    String,
    Char,
    Int,
    Code,
}

pub(crate) fn match_literal_or_init_from(
    attribute: &Meta,
    accepted_literals: AcceptedLiterals,
) -> Result<Option<TokenStream>> {
    match attribute {
        Meta::Path(_) => Ok(None),
        Meta::NameValue(MetaNameValue { value, .. }) => {
            if matches!(accepted_literals, AcceptedLiterals::Code) {
                return Ok(Some(value.to_token_stream()));
            }
            let lit = match value {
                Expr::Lit(ExprLit { lit, .. }) => lit,
                other => panic_span!(
                    attribute.span(),
                    "Unknown attribute meta {}",
                    other.to_token_stream()
                ),
            };
            let code = lit.to_token_stream();
            match accepted_literals {
                AcceptedLiterals::String => {
                    if !matches!(lit, Lit::Str(_)) {
                        panic_span!(attribute.span(), "expected string, got {code}");
                    }
                }
                AcceptedLiterals::Char => {
                    if !matches!(lit, Lit::Char(_)) {
                        panic_span!(attribute.span(), "expected char, got {code}");
                    }
                }
                AcceptedLiterals::Int => {
                    if !matches!(lit, Lit::Int(_) | Lit::Float(_)) {
                        panic_span!(attribute.span(), "expected char, got {code}");
                    }
                }
                AcceptedLiterals::Code => unreachable!(),
            }
            Ok(Some(lit.to_token_stream()))
        }
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
                    Ok(Some(quote_spanned!(attribute.span()=> #expr)))
                }
                any => panic_span!(
                    attribute.span(),
                    "unexpected attribute type, must be string literal: {:#?}",
                    any
                ),
            }
        }
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
            Ok(Some(quote_spanned!(meta.span()=>  #value )))
        }
    }
}

pub(crate) fn meta_to_option(meta: &Meta) -> Result<Option<TokenStream>> {
    match_literal_or_init_from(meta, AcceptedLiterals::String)
}
