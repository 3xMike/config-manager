// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

mod utils;

use super::{attributes::*, format_to_tokens};
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
    pub(crate) long: String,
    pub(crate) short: Option<String>,
    pub(crate) help: Option<String>,
    pub(crate) long_help: Option<String>,
}

pub(crate) struct ProcessFieldResult {
    pub(crate) name: proc_macro2::Ident,
    pub(crate) clap_field: ClapInitialization,
    pub(crate) initialization: TokenStream,
}

pub(crate) fn field_is_source(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| compare_attribute_name(attr, SOURCE_KEY))
}

pub(crate) fn process_field(field: Field, table_name: &Option<String>) -> ProcessFieldResult {
    let field_name = field.ident.clone().expect("Unnamed fields are forbidden");
    if number_of_crate_attribute(&field) != 1 {
        panic!(
            "Error: source attribute must be the only attribute of the field (field's name: \
             \"{}\")",
            &field_name
        );
    }

    let attributes_order = extract_attributes(field, table_name.clone());

    ProcessFieldResult {
        initialization: attributes_order.gen_init(&field_name.to_string()),
        clap_field: match attributes_order.clap_field(&field_name.to_string()) {
            Some(init) => ClapInitialization::Normal(init),
            None => ClapInitialization::None,
        },
        name: field_name,
    }
}

pub(crate) fn field_is_flatten(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| compare_attribute_name(attr, FLATTEN))
}

pub(crate) fn process_flatten_field(field: Field) -> ProcessFieldResult {
    let name = field.ident.clone().expect("Unnamed fields are forbidden");
    if number_of_crate_attribute(&field) != 1 {
        panic!(
            "Error: flatten attribute must be the only attribute of the field (field's name: \
             \"{}\")",
            &name
        );
    }
    let ty = field.ty;

    ProcessFieldResult {
        name,
        clap_field: ClapInitialization::Flatten(ty.clone()),
        initialization: quote! {
            <#ty as ::config_manager::__private::Flatten>::parse(&env_data, &config_file_data ,&clap_data, env_prefix.clone())?
        },
    }
}

pub(crate) fn field_is_subcommand(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| compare_attribute_name(attr, SUBCOMMAND))
}

pub(crate) fn process_subcommand_field(
    field: Field,
    dbg_cmd: &Option<TokenStream>,
) -> ProcessFieldResult {
    let name = field.ident.clone().expect("Unnamed fields are forbidden");
    if number_of_crate_attribute(&field) != 1 {
        panic!(
            "Error: subcommand attribute must be the only attribute of the field (field's name: \
             \"{}\")",
            &name
        );
    }
    let string_name = name.to_string();
    let ty = field.ty;

    let args = match dbg_cmd {
        None => quote!(::std::env::args()),
        Some(args) => quote! {
            ::std::vec!["", #args].into_iter().map(|s| s.to_string())
        },
    };
    let (initialization, ty) = if let Some(nested_ty) = is_type_an_optional(&ty) {
        (
            quote! (::config_manager::__private::parse_subcommand::<#nested_ty>(#args, clap_data)?),
            nested_ty,
        )
    } else {
        (
            quote! {
                ::config_manager::__private::parse_subcommand::<#ty>(#args, clap_data)?
                    .ok_or_else(|| ::config_manager::Error::MissingArgument(
                        ::std::format!("Missing subcommand for non-optional field \"{}\"", #string_name)
                    ))?
            },
            ty,
        )
    };

    ProcessFieldResult {
        name,
        clap_field: ClapInitialization::Subcommand(ty),
        initialization,
    }
}

fn number_of_crate_attribute(field: &Field) -> usize {
    field
        .attrs
        .iter()
        .filter(|attr| {
            [SOURCE_KEY, FLATTEN, SUBCOMMAND].contains(&path_to_string(&attr.path).as_str())
        })
        .count()
}
