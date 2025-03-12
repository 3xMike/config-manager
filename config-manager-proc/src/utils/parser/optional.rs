// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::{utils::meta_value_lit, *};

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

            if args.len() != 1 {
                panic!("len of nested args must be exactly 1");
            }

            let attr_path = args[0].path();
            let init_from = parse_quote! { init_from };
            if *attr_path != init_from {
                panic!(
                    "expected {:#?}, got {:#?}",
                    init_from.to_token_stream().to_string(),
                    attr_path.to_token_stream().to_string()
                )
            } else {
                match &args[0] {
                    Meta::NameValue(meta_value_lit!(lit)) => Some(InitFrom::Fn(lit.value())),
                    any => panic!(
                        "unexpected attribute type, must be string literal: {:#?}",
                        any
                    ),
                }
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
        Meta::NameValue(MetaNameValue { value, .. }) => Some(value.to_token_stream().to_string()),
    }
}
