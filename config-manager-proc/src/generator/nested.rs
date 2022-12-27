// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

pub(super) fn generate_get_args_impl(
    clap_fields: impl Iterator<Item = ClapInitialization>,
) -> TokenStream {
    let mut pushes = TokenStream::new();
    for field in clap_fields {
        match field {
            ClapInitialization::None => (),
            ClapInitialization::Normal(arg) => {
                pushes.extend(quote! {
                    res.push(#arg);
                });
            }
            ClapInitialization::Flatten(struct_type) => {
                pushes.extend(quote! {
                    res.extend_from_slice(&<#struct_type as ::config_manager::__private::Flatten>::get_args());
                })
            }
            ClapInitialization::Subcommand(sub) => panic!("Subcommand(type = {}) in a nested struct", sub.to_token_stream())
        }
    }
    quote! {
        use ::config_manager::__private::clap;
        let mut res = ::std::vec::Vec::new();
        #pushes
        res
    }
}

pub(super) fn generate_parse_impl(
    fields_init: Vec<(Ident, TokenStream)>,
    class: &Ident,
) -> TokenStream {
    let fields_initialization =
        fields_init
            .iter()
            .fold(TokenStream::new(), |mut acc, (name, definition)| {
                acc.extend(quote! {
                    #name: #definition,
                });
                acc
            });

    quote! {
        ::std::result::Result::<_, ::config_manager::Error>::Ok(
            #class {
                #fields_initialization
            }
        )
    }
}
