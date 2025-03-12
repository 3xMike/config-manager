// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

pub(crate) enum InitFrom {
    Fn(String),
    Literal(Lit),
}

impl InitFrom {
    pub(crate) fn as_string(&self) -> String {
        match self {
            InitFrom::Fn(value) => format!("{{{value}}}"),
            InitFrom::Literal(lit) => lit.to_token_stream().to_string(),
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) enum AcceptedLiterals {
    String,
    Char,
    Bool,
}

pub(crate) struct OptionalAttribute<T: ToString> {
    pub(crate) ty: T,
    pub(crate) value: Option<String>,
    pub(crate) accepted_literals: AcceptedLiterals,
}

pub(crate) enum SetOptionalAttrResult {
    Set,
    ErrorAlreadySet,
    NameMismatch,
}

pub(crate) fn try_set_optional_attribute<T: ToString>(
    attr: &Meta,
    opt_atr: &mut OptionalAttribute<T>,
    default_behaviour: bool,
) -> SetOptionalAttrResult {
    if path_to_string(attr.path()) != opt_atr.ty.to_string() {
        SetOptionalAttrResult::NameMismatch
    } else {
        if opt_atr.value.is_some() {
            return SetOptionalAttrResult::ErrorAlreadySet;
        }
        opt_atr.value = Some(
            match match_literal_or_init_from(attr, opt_atr.accepted_literals) {
                Some(InitFrom::Fn(value)) => format!("{{{value}}}"),
                Some(InitFrom::Literal(lit)) => {
                    if !default_behaviour {
                        lit.to_token_stream().to_string()
                    } else {
                        match lit {
                            Lit::Str(str) => str.value(),
                            lit => lit.to_token_stream().to_string(),
                        }
                    }
                }
                None => panic!("Nested attributes of `file` can't be empty"),
            },
        );
        SetOptionalAttrResult::Set
    }
}

pub(crate) fn match_literal_or_init_from(
    attribute: &Meta,
    accepted_literals: AcceptedLiterals,
) -> Option<InitFrom> {
    match attribute {
        Meta::Path(_) => None,
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit { lit, .. }),
            ..
        }) => Some(match accepted_literals {
            AcceptedLiterals::String => {
                if matches!(lit, Lit::Str(_)) {
                    InitFrom::Literal(lit.clone())
                } else {
                    panic!("expected string, got {:#?}", lit);
                }
            }
            AcceptedLiterals::Bool => {
                if matches!(lit, Lit::Bool(_)) {
                    InitFrom::Literal(lit.clone())
                } else {
                    panic!("expected bool, got {:#?}", lit);
                }
            }
            AcceptedLiterals::Char => {
                if matches!(lit, Lit::Char(_)) {
                    InitFrom::Literal(lit.clone())
                } else {
                    panic!("expected char, got {:#?}", lit);
                }
            }
        }),
        Meta::List(list) => {
            let args = list
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
                .unwrap();

            if !args[0].path().is_ident("init_from") {
                panic!("len of nested args must be exactly 1 and it must be \"init_from = ...\"")
            }

            match &args[0] {
                Meta::NameValue(expr) => {
                    Some(InitFrom::Fn(expr.value.to_token_stream().to_string()))
                }
                any => panic!(
                    "unexpected attribute type, must be string literal: {:#?}",
                    any
                ),
            }
        }
        other => panic!("Unknown attribute meta {}", other.to_token_stream()),
    }
}

pub(crate) fn extract_default(meta: &Meta) -> Option<String> {
    match meta {
        Meta::Path(_) => None,
        Meta::List(_) => {
            panic!("default attribute must be #[source(default = \"...\")] of #[source(default)]")
        }
        Meta::NameValue(MetaNameValue { value, .. }) => {
            Some(format!("{{{}}}", value.to_token_stream()))
        }
    }
}
