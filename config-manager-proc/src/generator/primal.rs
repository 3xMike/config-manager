// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

pub(super) fn gen_clap_app(
    clap_app_info: NormalClapAppInfo,
    configs_as_clap_args: Punctuated<ClapInitialization, Token![.]>,
    mut clap_fields: Vec<ClapInitialization>,
) -> TokenStream {
    let subcommand = if let Some(pos) = clap_fields
        .iter()
        .position(|init| matches!(init, ClapInitialization::Subcommand(_)))
    {
        let sub = clap_fields.remove(pos);
        if clap_fields
            .iter()
            .any(|init| matches!(init, ClapInitialization::Subcommand(_)))
        {
            panic!("Structure can contain only one subcommand field");
        }
        sub.to_token_stream()
    } else {
        quote!(app)
    };

    let fields = if clap_fields.is_empty() {
        TokenStream::new()
    } else {
        let clap_fields = Punctuated::<ClapInitialization, Token![.]>::from_iter(
            clap_fields
                .into_iter()
                .filter(|init| !matches!(init, &ClapInitialization::None)),
        );
        if clap_fields.is_empty() {
            TokenStream::new()
        } else {
            quote!(.#clap_fields)
        }
    };

    let configs = if configs_as_clap_args.is_empty() {
        TokenStream::new()
    } else {
        quote!(.#configs_as_clap_args)
    };
    quote! {
        {
            use ::config_manager::__private::clap;
            let app = #clap_app_info
                #configs
                #fields;
            #subcommand
        }
    }
}

pub(super) fn gen_clap_matches(
    debug_cmd_input: Option<TokenStream>,
) -> TokenStream {
    let parsing_method = match debug_cmd_input {
        None => quote! {.try_get_matches()},
        Some(args) => quote! {.try_get_matches_from(::std::vec!["", #args])},
    };
    
    quote! {
        {
            (|| -> ::std::result::Result<::config_manager::__private::clap::ArgMatches, ::config_manager::Error> {
                ::std::result::Result::Ok({
                    clap_app.clone()
                    #parsing_method
                    .map_err(|err| {
                        if err.kind() == ::config_manager::__private::clap::error::ErrorKind::DisplayHelp
                            || err.kind() == ::config_manager::__private::clap::error::ErrorKind::DisplayVersion {
                            err.exit();
                        } else {
                            ::config_manager::Error::ExternalError(::std::format!(
                                "failed to parse command line: {}",
                                err
                            ))
                        }
                    })?
                })
            })()
        }
    }
}

pub(super) fn gen_env_data() -> TokenStream {
    quote! {
        {
            (|| -> ::std::result::Result::<::config_manager::__private::EnvData, ::config_manager::Error> {
                use ::config_manager::__private::config::Source;

                let env = ::config_manager::__private::config::Environment::default()
                    .collect()
                    .map_err(|err| ::config_manager::Error::ExternalError(::std::format!("failed to collect Environment: {}", err)))?;

                let mut res = ::std::collections::HashMap::<::std::string::String, ::std::string::String>::new();
                for (k, v) in env {
                    res.insert(
                        k,
                        v.into_string().map_err(|err| {
                            ::config_manager::Error::ExternalError(::std::format!(
                                "failed to convert config::Value to string: {}",
                                err
                            ))
                        })?,
                    );
                }
                ::std::result::Result::Ok(::config_manager::__private::EnvData::from(res))
            })()
        }
    }
}

pub(super) fn gen_config_file_data(config_keys: Vec<ConfigFileInfo>) -> TokenStream {
    let env_data = quote!(env_data);
    let matches = quote!(clap_data);

    let mut config_paths_init = TokenStream::new();
    for ConfigFileInfo {
        file_format,
        env_key,
        clap_long,
        is_optional,
        default_path,
    } in config_keys
    {
        config_paths_init.extend(quote! {
            if let ::std::result::Result::Err(err) = (|| {
                let mut err_msg = ::std::vec![];

                if let ::std::option::Option::<&::std::primitive::str>::Some(clap_long) = #clap_long {
                    if let Some(field_match) = #matches.get_one::<::std::string::String>(clap_long) { 
                        res.push((#file_format, field_match.to_string()));
                        return ::std::result::Result::Ok(());
                    }
                    err_msg.push(::std::format!("key `{}` was not found in command line", clap_long));
                }

                if let ::std::option::Option::<&::std::primitive::str>::Some(env_key) = #env_key {
                    let from_env = #env_data.get(env_key);
                    if let ::std::option::Option::Some(path) = from_env {
                        res.push((#file_format, path.clone()));
                        return ::std::result::Result::Ok(());
                    }
                    err_msg.push(::std::format!("key `{}` was not found in environment", env_key));
                }

                if let ::std::option::Option::<&::std::primitive::str>::Some(default_path) = #default_path {
                    res.push((#file_format, default_path.to_string()));
                    return ::std::result::Result::Ok(());
                }
                err_msg.push("default path for file was not specified".into());

                let err_msg = err_msg.join("; ");
                if #is_optional {
                    return ::std::result::Result::Ok(());
                }
                return ::std::result::Result::Err(::config_manager::Error::MissingArgument(
                    err_msg
                ));
            })() {
                return ::std::result::Result::Err(err);
            };
        })
    }

    quote! {
        {
            (|| -> ::std::result::Result<::std::collections::HashMap::<::std::string::String, ::config_manager::__private::config::Value>, ::config_manager::Error> {
                use ::config_manager::__private::config::Source;

                let configs_paths: ::std::vec::Vec<(::config_manager::__private::config::FileFormat, ::std::string::String)> = {
                    let mut res = ::std::vec![];
                    #config_paths_init
                    res
                };

                let mut res = ::std::collections::HashMap::new();
                for (format, path) in configs_paths {
                    res.extend(
                        ::config_manager::__private::config::File::new(&path, format)
                            .collect()
                            .map_err(|err|
                                ::config_manager::Error::ExternalError(
                                    ::std::format!("failed to collect config file {}: {}", path, err)
                                )
                            )?
                    );
                }
                ::std::result::Result::Ok(res)
            })()
        }
    }
}

fn gen_sources() -> TokenStream {
    let clap_err = quote! {
        .map_err(|err| {
            if err.kind() == ::config_manager::__private::clap::error::ErrorKind::DisplayHelp
                || err.kind() == ::config_manager::__private::clap::error::ErrorKind::DisplayVersion
            {
                err.exit();
            } else {
                ::config_manager::Error::ExternalError(::std::format!(
                    "failed to parse command line: {}",
                    err
                ))
            }
        })
    };

    
    quote! {
        let (mut env_data, mut clap_data, mut config_file_data, mut env_prefix) =
            ::std::default::Default::default();
        
            for option in options {
                match option {
                    ::config_manager::ConfigOption::EnvPrefix(pref) => {
                        env_prefix = Some(pref);
                    },
                    ::config_manager::ConfigOption::ExplicitSource(::config_manager::Source::Env(env)) => {
                        use ::std::iter::FromIterator;
                        let env = ::std::collections::HashMap::from_iter(
                            env.into_iter().map(|(key, val)| (key.to_lowercase(), val))
                        );
                        env_data = ::std::option::Option::Some(::config_manager::__private::EnvData::from(env));
                    }
                    ::config_manager::ConfigOption::ExplicitSource(::config_manager::Source::Clap(clap_source)) => {
                        clap_data = ::std::option::Option::Some(match clap_source {
                            ::config_manager::ClapSource::None => clap_app.clone()
                                .try_get_matches_from(::std::iter::empty::<std::ffi::OsString>())
                                #clap_err?,
                            ::config_manager::ClapSource::Args(mut args) => {
                                args.insert(0, "".to_string());
                                clap_app.clone()
                                    .try_get_matches_from(args)
                                    #clap_err?
                            }
                            ::config_manager::ClapSource::Matches(matches) => matches,
                        });
                    }
                    ::config_manager::ConfigOption::ExplicitSource(::config_manager::Source::ConfigFiles(
                        files,
                    )) => {
                        config_file_data = ::std::option::Option::Some({
                            let mut res = ::std::collections::HashMap::new();
                            for ::config_manager::FileOptions { format, path } in files {
                                res.extend(
                                    ::config_manager::__private::config::Source::collect(
                                        &::config_manager::__private::config::File::new(&path, format),
                                    )
                                    .map_err(|err| {
                                        ::config_manager::Error::ExternalError(::std::format!(
                                            "failed to collect config file {}: {}",
                                            path,
                                            err
                                        ))
                                    })?,
                                );
                            }
                            res
                        });
                    }
                }
            }
        
    }
}

pub(super) fn struct_initialization(
    class_ident: &proc_macro2::Ident,
    fields_json_definition: Vec<(proc_macro2::Ident, TokenStream)>,
    clap_app: &TokenStream,
    clap_data: &TokenStream,
    env_data: &TokenStream,
    env_prefix: &TokenStream,
    config_file_data: &TokenStream,
) -> TokenStream {
    let sources = gen_sources();
    let sources = quote! {
        let clap_app = #clap_app;
        #sources

        let clap_data = match clap_data {
            ::std::option::Option::Some(data) => data,
            ::std::option::Option::None => #clap_data?,
        };
        let clap_data = &clap_data;
        let env_data = match env_data {
            ::std::option::Option::Some(data) => data,
            ::std::option::Option::None => #env_data?
        };
        let config_file_data = match config_file_data {
            ::std::option::Option::Some(data) => data,
            ::std::option::Option::None => #config_file_data?
        };
        let env_prefix = match env_prefix {
            ::std::option::Option::Some(prefix) => ::std::option::Option::Some(prefix),
            ::std::option::Option::None => #env_prefix,
        };
    };

    let fields_initialization =
        fields_json_definition
            .iter()
            .fold(TokenStream::new(), |mut acc, (name, definition)| {
                acc.extend(quote! {
                    #name: #definition,
                });
                acc
            });

    let init_body = quote! {
        ::std::result::Result::<_, ::config_manager::Error>::Ok(
            #class_ident {
                #fields_initialization
            }
        )
    };

    quote! {
        #sources
        #init_body
    }   
}