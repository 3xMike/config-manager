// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use super::{nested::*, primal::*};
use crate::{utils::option_to_tokens, *};

pub(crate) struct InitializationInfo {
    pub(crate) env_prefix: Option<String>,
    pub(crate) class_ident: proc_macro2::Ident,
    pub(crate) clap_app_info: NormalClapAppInfo,
    pub(crate) configs: ConfigFilesInfo,
    pub(crate) clap_fields: Vec<ClapInitialization>,
    pub(crate) fields_json_definition: Vec<(proc_macro2::Ident, TokenStream)>,
    pub(crate) debug_cmd_input: Option<TokenStream>,
}

pub(crate) fn generate_final_struct_and_supporting_code(info: InitializationInfo) -> TokenStream {
    let InitializationInfo {
        env_prefix,
        class_ident,
        clap_app_info,
        configs:
            ConfigFilesInfo {
                configs_as_clap_args,
                configs_attributes,
            },
        clap_fields,
        fields_json_definition,
        debug_cmd_input,
    } = info;

    let clap_app = gen_clap_app(clap_app_info, configs_as_clap_args, clap_fields);
    let clap_data = gen_clap_matches(debug_cmd_input);
    let config_file_data = gen_config_file_data(configs_attributes);
    let env_data = gen_env_data();

    let initialization = struct_initialization(
        &class_ident,
        fields_json_definition,
        &clap_app,
        &clap_data,
        &env_data,
        &option_to_tokens(&env_prefix),
        &config_file_data,
    );

    quote! {
        impl ::config_manager::ConfigInit for #class_ident {
            fn parse_options(options: ::config_manager::ConfigOptions) -> ::std::result::Result<Self, ::config_manager::Error> {
                #initialization
            }

            fn get_command() -> ::config_manager::__private::clap::Command {
                #clap_app
            }
        }
    }
}

pub(crate) fn generate_flatten_implementation(
    class: Ident,
    clap_info: Punctuated<ClapInitialization, Token![.]>,
    fields_init: Vec<(Ident, TokenStream)>,
) -> TokenStream {
    let get_args_impl = generate_get_args_impl(clap_info.into_iter());
    let parse_impl = generate_parse_impl(fields_init, &class);

    quote! {
        impl ::config_manager::__private::Flatten for #class {
            fn get_args() -> ::std::vec::Vec<::config_manager::__private::clap::Arg> {
                #get_args_impl
            }

            fn parse(
                env_data: &::config_manager::__private::EnvData,
                config_file_data: &::std::collections::HashMap<::std::string::String, ::config_manager::__private::config::Value>,
                clap_data: &::config_manager::__private::clap::ArgMatches,
                env_prefix: ::std::option::Option::<::std::string::String>,
            ) -> Result<Self, ::config_manager::Error>
            where Self: ::std::marker::Sized
            {
                #parse_impl
            }
        }
    }
}
