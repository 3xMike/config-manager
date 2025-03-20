// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

pub(crate) mod utils;

use std::{collections::HashMap, default::Default};

use super::attributes::*;
use crate::*;
use utils::*;

#[derive(Clone)]
pub(crate) enum ClapInitialization {
    None,
    Normal(NormalClapFieldInfo),
    Flatten(Type),
    Subcommand(Type),
}

#[derive(Clone)]
pub(crate) struct NormalClapFieldInfo {
    pub(crate) span: Span,

    pub(crate) long: TokenStream,
    pub(crate) attributes: HashMap<String, TokenStream>,
}

pub(crate) struct ProcessFieldResult {
    pub(crate) name: proc_macro2::Ident,
    pub(crate) clap_field: ClapInitialization,
    pub(crate) initialization: TokenStream,
}

pub(crate) fn process_field(
    field: Field,
    table_name: &Option<TokenStream>,
    default_order: &Option<ExtractedAttributes>,
) -> Result<ProcessFieldResult> {
    let field_name = field.ident.clone().unwrap();

    let attributes_order = extract_attributes(&field, table_name)?
        .or_else(|| default_order.clone())
        .unwrap_or_else(|| ExtractedAttributes {
            variables: vec![
                FieldAttribute::Clap(ClapFieldParseResult::new(field_name.span())),
                FieldAttribute::Env(Default::default()),
                FieldAttribute::Config(Default::default()),
            ],
            span: field.span(),
            default: None,
            deserializer: None,
        });

    Ok(ProcessFieldResult {
        initialization: attributes_order.gen_init(&field),
        clap_field: match attributes_order.clap_field(&field_name.to_string())? {
            Some(init) => ClapInitialization::Normal(init),
            None => ClapInitialization::None,
        },
        name: field_name,
    })
}

pub(crate) fn field_is_flatten(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident(FLATTEN))
}

pub(crate) fn process_flatten_field(field: Field) -> Result<ProcessFieldResult> {
    let span = field.span();
    let name = field.ident.clone().unwrap();
    let ty = field.ty;

    Ok(ProcessFieldResult {
        name,
        clap_field: ClapInitialization::Flatten(ty.clone()),
        initialization: quote_spanned! {span=>
            <#ty as ::config_manager::__private::Flatten>::parse(env_data, config_file_data, clap_data, env_prefix.clone())?
        },
    })
}

pub(crate) fn field_is_subcommand(field: &Field) -> Option<&Attribute> {
    field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident(SUBCOMMAND))
}

pub(crate) fn process_subcommand_field(
    field: Field,
    dbg_cmd: &Option<TokenStream>,
) -> Result<ProcessFieldResult> {
    let span = field.span();
    let name = field.ident.clone().unwrap();
    let string_name = name.to_string();
    let ty = field.ty;

    let args = match dbg_cmd {
        None => quote_spanned!(span=> ::std::env::args()),
        Some(args) => quote_spanned! {span=>
            ::std::vec!["", #args].into_iter().map(|s| s.to_string())
        },
    };
    let (initialization, ty) = if let Some(nested_ty) = is_type_an_optional(&ty) {
        (
            quote_spanned! {span=>
                ::config_manager::__private::parse_subcommand::<#nested_ty>(#args, clap_data)?
            },
            nested_ty,
        )
    } else {
        (
            quote_spanned! {span=>
                ::config_manager::__private::parse_subcommand::<#ty>(#args, clap_data)?
                    .ok_or_else(|| ::config_manager::Error::MissingArgument(
                        ::std::format!("Missing subcommand for non-optional field \"{}\"", #string_name)
                    ))?
            },
            ty,
        )
    };

    Ok(ProcessFieldResult {
        name,
        clap_field: ClapInitialization::Subcommand(ty),
        initialization,
    })
}

pub(crate) fn check_field_attributes(field: &Field) -> Result<()> {
    let applied_crate_attrs = field
        .attrs
        .iter()
        .filter_map(|attr| {
            [SOURCE_KEY, FLATTEN, SUBCOMMAND]
                .into_iter()
                .find(|crate_attr| crate_attr == &path_to_string(attr.path()))
        })
        .collect::<Vec<_>>();

    if applied_crate_attrs.len() > 1 {
        let message =
            format!("Can't use {applied_crate_attrs:?} at the same time. Use only one of them");
        Err(Error::new(field.ident.clone().unwrap().span(), message))
    } else {
        Ok(())
    }
}
