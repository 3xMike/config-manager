// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

mod generator;
mod utils;

use std::{fmt::Write, str::FromStr};

use proc_macro::TokenStream as TokenStream0;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::Parser, punctuated::Punctuated, spanned::Spanned, *};

use generator::*;
use utils::{config::*, field::*, panic_site, panic_span, parser::*, top_level::*};

pub(crate) use syn::Error;
pub(crate) type Result<T> = std::result::Result<T, Error>;

/// Macro generating an implementation of the `ConfigInit` trait
/// or constructing global variable. \
///
/// For more info see crate level documentation
#[proc_macro_attribute]
pub fn config(attrs: TokenStream0, input: TokenStream0) -> TokenStream0 {
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let attrs = parser.parse(attrs).unwrap();
    let mut annotations = String::from("#[derive(::config_manager::__private::__Config__)]");
    attrs.iter().for_each(|attr| {
        std::write!(&mut annotations, "\n{}", (quote! { #[#attr]})).unwrap();
    });
    let mut annotations =
        TokenStream0::from_str(&annotations).expect("can't parse annotations as tokenstream");
    annotations.extend(input);
    annotations
}

#[proc_macro_derive(
    __Config__,
    attributes(
        source,
        flatten,
        subcommand,
        config,
        env_prefix,
        clap,
        global_name,
        file,
        table,
        default_order,
        __debug_cmd_input__
    )
)]
pub fn generate_config(input: TokenStream0) -> TokenStream0 {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_config_inner(input) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn generate_config_inner(input: DeriveInput) -> Result<TokenStream> {
    let class_ident = input.ident;

    let AppTopLevelInfo {
        env_prefix,
        clap_app_info,
        configs,
        debug_cmd_input,
        table_name,
        default_order,
    } = AppTopLevelInfo::extract(&input.attrs)?;

    let class: DataStruct = match input.data {
        Data::Struct(s) => s,
        _ => panic_site!("config macro input should be a Struct"),
    };

    let mut fields_json_definition = Vec::new();
    let mut clap_fields = Vec::new();

    for field in class.fields {
        let res = if field_is_flatten(&field) {
            process_flatten_field(field)
        } else if field_is_subcommand(&field).is_some() {
            process_subcommand_field(field, &debug_cmd_input)
        } else {
            process_field(field, &table_name, &default_order)?
        };

        fields_json_definition.push((res.name, res.initialization));
        clap_fields.push(res.clap_field);
    }

    generate_final_struct_and_supporting_code(InitializationInfo {
        env_prefix,
        class_ident,
        clap_app_info,
        configs,
        clap_fields,
        fields_json_definition,
        debug_cmd_input,
    })
}

/// Annotated with this macro structure can be used
/// as a flatten argument in the [config](attr.config.html) macro.
#[proc_macro_derive(Flatten, attributes(source, flatten, subcommand, table, default_order))]
pub fn generate_flatten(input: TokenStream0) -> TokenStream0 {
    let input = parse_macro_input!(input as DeriveInput);

    match generate_flatten_inner(input) {
        Ok(res) => res.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn generate_flatten_inner(input: DeriveInput) -> Result<TokenStream> {
    let table_name = extract_table_name(&input.attrs);
    let default_order = extract_source_order(&input.attrs);

    let class_ident = input.ident;
    let class: DataStruct = match input.data {
        Data::Struct(s) => s,
        _ => panic_site!("config macro input should be a Struct"),
    };

    let mut fields_json_definition = Vec::new();
    let mut clap_fields = Punctuated::<ClapInitialization, Token![.]>::new();

    for field in class.fields {
        let res = if field_is_flatten(&field) {
            Ok(process_flatten_field(field))
        } else if let Some(attr) = field_is_subcommand(&field) {
            Err(Error::new(
                attr.meta.span(),
                "subcommands are forbidden in the nested structures",
            ))
        } else {
            process_field(field, &table_name, &default_order)
        }?;

        fields_json_definition.push((res.name, res.initialization));
        clap_fields.push(res.clap_field);
    }

    generate_flatten_implementation(class_ident, clap_fields, fields_json_definition)
}
