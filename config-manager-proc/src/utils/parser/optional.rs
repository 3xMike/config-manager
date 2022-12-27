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
    AnyLiteral,
    StringOnly,
    CharOnly,
    BoolOnly,
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
        Meta::NameValue(MetaNameValue { lit, .. }) => Some(match accepted_literals {
            AcceptedLiterals::AnyLiteral => InitFrom::Literal(lit.clone()),
            AcceptedLiterals::StringOnly => {
                if matches!(lit, Lit::Str(_)) {
                    InitFrom::Literal(lit.clone())
                } else {
                    panic!("expected string, got {:#?}", lit);
                }
            }
            AcceptedLiterals::BoolOnly => {
                if matches!(lit, Lit::Bool(_)) {
                    InitFrom::Literal(lit.clone())
                } else {
                    panic!("expected bool, got {:#?}", lit);
                }
            }
            AcceptedLiterals::CharOnly => {
                if matches!(lit, Lit::Char(_)) {
                    InitFrom::Literal(lit.clone())
                } else {
                    panic!("expected char, got {:#?}", lit);
                }
            }
        }),
        Meta::List(MetaList { nested: args, .. }) => {
            if args.len() != 1 {
                panic!("len of nested args must be exactly 1");
            }
            return match &args[0] {
                NestedMeta::Meta(attribute) => {
                    let atr_path = attribute.path();
                    let init_from = parse_quote! { init_from };
                    if *atr_path != init_from {
                        panic!(
                            "expected {:#?}, got {:#?}",
                            init_from.to_token_stream().to_string(),
                            atr_path.to_token_stream().to_string()
                        )
                    } else {
                        match attribute {
                            Meta::NameValue(MetaNameValue { lit, .. }) => {
                                if let Lit::Str(lit) = lit {
                                    Some(InitFrom::Fn(lit.value()))
                                } else {
                                    panic!("init_from attribute must be a string literal")
                                }
                            }
                            any => panic!("unexpected attribute type, must be literal: {:#?}", any),
                        }
                    }
                }
                arg => panic!("unexpected attribute: {:#?}", arg),
            };
        }
    }
}
