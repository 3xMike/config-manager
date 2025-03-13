// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use std::fmt::Display;

use super::*;

impl ToTokens for ClapInitialization {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::None => unreachable!(),
            Self::Flatten(tp) => {
                quote_spanned! {tp.span()=>
                    args(<#tp as ::config_manager::__private::Flatten>::get_args())
                }
            }
            Self::Subcommand(tp) => {
                quote_spanned! {tp.span()=>
                    <#tp as ::config_manager::__private::clap::Subcommand>::augment_subcommands(app)
                }
            }
            Self::Normal(info) => quote_spanned! {info.span()=>
                 arg(#info)
            },
        })
    }
}

impl ToTokens for NormalClapFieldInfo {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend({
            let long = format_to_tokens!(".long({})", self.long);
            let name = format_to_tokens!("{}", self.long);
            let short = match &self.short {
                None => TokenStream::new(),
                Some(short) => format_to_tokens!(".short({short})"),
            };
            let flag = if self.flag {
                format_to_tokens!(".num_args(0..=1).default_missing_value(\"true\")")
            } else {
                format_to_tokens!(".num_args(1)")
            };
            let help = match &self.help {
                None => TokenStream::new(),
                Some(help) => format_to_tokens!(".help({help})"),
            };
            let long_help = match &self.long_help {
                None => TokenStream::new(),
                Some(long_help) => format_to_tokens!(".long_help({long_help})"),
            };
            let help_heading = match &self.help_heading {
                None => TokenStream::new(),
                Some(help_heading) => format_to_tokens!(".help_heading({help_heading})"),
            };
            quote_spanned! {self.span=>
                clap::Arg::new(#name)
                #long
                #short
                #flag
                #help
                #long_help
                #help_heading
                .required(false)
            }
        })
    }
}

#[derive(Default, Clone)]
pub(crate) struct ExtractedAttributes {
    pub(crate) variables: Vec<FieldAttribute>,
    pub(crate) default: Option<Default>,
    pub(crate) deserializer: Option<String>,
}

impl ExtractedAttributes {
    fn deserializer(&self) -> TokenStream {
        match &self.deserializer {
            None => quote! {
                let value = if value.is_empty() {
                    "\"\"".to_string()
                } else {
                    value
                };
                ::config_manager::__private::deser_hjson::from_str(&value)
            },
            Some(deser_fn) => {
                let deser_fn = deser_fn.trim_matches('\"');
                format_to_tokens!("({deser_fn})(&value)")
            }
        }
    }

    fn gen_err(&self, field_name: &str) -> TokenStream {
        let err = format!(
            "field {field_name} not found nor in {} nor as a default",
            self.variables
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .as_slice()
                .join(", ")
        );
        quote! {
            ::config_manager::Error::MissingArgument(#err.to_string())
        }
    }

    fn gen_rest_init(&self, field_name: &str) -> TokenStream {
        self.variables.iter().fold(
            quote!(::std::option::Option::<::std::string::String>::None),
            |acc, attribute_init| {
                let attribute_init = attribute_init.gen_init(field_name);
                quote! {
                    #acc.or(#attribute_init)
                }
            },
        )
    }

    pub(super) fn clap_field(self, field_name: &str) -> Result<Option<NormalClapFieldInfo>> {
        for attr in self.variables {
            if let FieldAttribute::Clap(clap) = attr {
                return Some(clap.normalize(field_name)).transpose();
            }
        }
        Ok(None)
    }

    pub(super) fn gen_init(&self, field_name: &str) -> TokenStream {
        let default_initialization = match &self.default {
            None => quote!(::std::option::Option::None),
            Some(d) if d.inner.is_none() => quote!(::std::option::Option::Some(
                ::std::default::Default::default()
            )),
            Some(d) => {
                format_to_tokens!("::std::option::Option::Some({})", d.inner.clone().unwrap())
            }
        };
        let deserializer = self.deserializer();
        let rest = self.gen_rest_init(field_name);
        let missing_err = self.gen_err(field_name);

        quote! {
            (|| -> ::std::result::Result<_, ::config_manager::Error> {
                let init_without_default = #rest;
                match (init_without_default, #default_initialization) {
                    (::std::option::Option::<::std::string::String>::None, ::std::option::Option::None) => {
                            ::std::result::Result::<_, ::config_manager::Error>::Err(#missing_err)?
                        },
                    (::std::option::Option::<::std::string::String>::None, ::std::option::Option::Some(default_value)) => ::std::result::Result::Ok(default_value),
                    (::std::option::Option::<::std::string::String>::Some(value), _) => {
                        #deserializer.map_err(|err| {
                            ::config_manager::Error::FailedParse(
                                ::std::format!("Can't deserialize from value: {} of field {}; error is {}", value, #field_name, err)
                            )
                        })
                    }
                }
            })()?
        }
    }
}

#[derive(Clone)]
pub(crate) enum FieldAttribute {
    Clap(ClapFieldParseResult),
    Env(Env),
    Config(Config),
}

impl FieldAttribute {
    fn gen_init(&self, field_name: &str) -> TokenStream {
        match &self {
            Self::Env(env) => {
                format_to_tokens!(
                    "env_data.get(&({}) as \
                     &::std::primitive::str).map(::std::string::ToString::to_string)",
                    env.prefixed_name(field_name)
                )
            }
            Self::Config(cfg) => {
                format_to_tokens!(
                    "::config_manager::__private::find_field_in_table(&config_file_data, {}, \
                     {}.to_string())?",
                    cfg.table(),
                    cfg.key(field_name)
                )
            }
            Self::Clap(clap) => {
                format_to_tokens!(
                    "clap_data.get_one::<::std::string::String>({}).\
                     map(::std::string::ToString::to_string)",
                    clap.normal_long(field_name)
                )
            }
        }
    }
}

impl Display for FieldAttribute {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        let source = match self {
            Self::Clap(_) => "command line",
            Self::Config(_) => "configuration file",
            Self::Env(_) => "env",
        };
        write!(f, "{source}",)
    }
}

#[derive(Default, Clone)]
pub(crate) struct Env {
    pub(super) inner: Option<String>,
}

impl Env {
    fn prefixed_name(&self, field_name: &str) -> String {
        let env_attribute = match &self.inner {
            None => quote!(::std::option::Option::<&::std::primitive::str>::None),
            Some(value) => {
                format_to_tokens!("::std::option::Option::<&::std::primitive::str>::Some({value})")
            }
        };
        let binary_name = binary_name();
        let field_name_lowercase = field_name.to_lowercase();

        quote! {
            {
                let env_prefix = env_prefix.clone();
                match (#env_attribute, env_prefix) {
                    (::std::option::Option::Some(name), _) => name.to_string(),
                    (::std::option::Option::None, ::std::option::Option::None) => {
                        let binary_name = #binary_name?;
                        ::std::format!("{}_{}", binary_name, #field_name_lowercase)
                    },
                    (::std::option::Option::None, ::std::option::Option::Some(pref)) if pref.is_empty() => {
                        #field_name_lowercase.to_string()
                    },
                    (::std::option::Option::None, ::std::option::Option::Some(pref)) => {
                        ::std::format!("{}_{}", pref, #field_name_lowercase)
                    }
                }.to_lowercase()
            }
        }
        .to_string()
    }
}

#[derive(Default, Clone)]
pub(crate) struct Config {
    pub(super) key: Option<String>,
    pub(super) table: Option<String>,
}

impl Config {
    fn key(&self, field_name: &str) -> String {
        self.key
            .clone()
            .unwrap_or_else(|| format!("\"{field_name}\""))
    }
    fn table(&self) -> String {
        self.table
            .clone()
            .map(|table| format!("::std::option::Option::Some(\"{table}\".to_string())"))
            .unwrap_or_else(|| "::std::option::Option::None".to_string())
    }
}

#[derive(Default, Clone)]
pub(crate) struct Default {
    pub(super) inner: Option<String>,
}

pub(super) fn extract_attributes(
    field: Field,
    table_name: &Option<String>,
) -> Result<Option<ExtractedAttributes>> {
    let is_bool = field.ty.to_token_stream().to_string() == "bool";
    let is_string = is_string(&field.ty);
    let docs = extract_docs(&field.attrs);

    let mut res = ExtractedAttributes::default();

    let attr = match field.attrs.iter().find(|a| a.path().is_ident(SOURCE_KEY)) {
        None => return Ok(None),
        Some(attr) => attr,
    };

    let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;

    for arg in nested {
        match path_to_string(arg.path()).as_str() {
            CLAP_KEY => match &arg {
                Meta::Path(_p) => res
                    .variables
                    .push(FieldAttribute::Clap(ClapFieldParseResult::new(arg.span()))),
                Meta::List(clap_metalist) => {
                    let mut clap_attributes = parse_clap_field_attribute(clap_metalist, is_bool)?;
                    clap_attributes.docs = docs.clone();
                    res.variables.push(FieldAttribute::Clap(clap_attributes));
                }
                _ => {
                    panic_span!(
                        arg.span(),
                        "clap attribute must match #[clap(...)] or #[clap]"
                    )
                }
            },
            DEFAULT => {
                if res.default.is_some() {
                    panic_span!(arg.span(), "Default can be assigned only once per field")
                }
                let mut default_init = extract_default(&arg);
                if is_string {
                    default_init = default_init.map(|s| format!("::std::convert::Into::into({s})"));
                }
                res.default = Some(Default {
                    inner: default_init,
                })
            }
            ENV_KEY => res.variables.push(FieldAttribute::Env(Env {
                inner: match_literal_or_init_from(&arg, AcceptedLiterals::String)
                    .as_ref()
                    .map(InitFrom::as_string),
            })),
            CONFIG_KEY => res.variables.push(FieldAttribute::Config(Config {
                key: match_literal_or_init_from(&arg, AcceptedLiterals::String)
                    .as_ref()
                    .map(InitFrom::as_string),
                table: table_name.clone(),
            })),
            DESERIALIZER => {
                if res.deserializer.is_some() {
                    panic_span!(
                        arg.span(),
                        "Deserialize_with can be assigned only once per field"
                    )
                }
                res.deserializer = match_literal_or_init_from(&arg, AcceptedLiterals::String)
                    .as_ref()
                    .map(InitFrom::as_string);
            }
            _ => panic_span!(arg.span(), "Unknown source attribute"),
        };
    }

    Ok(Some(res))
}

fn is_string(ty: &Type) -> bool {
    let path = match ty {
        Type::Path(path) if path.qself.is_none() => &path.path,
        _ => return false,
    };
    let idents_of_path = path.segments.iter().fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push('|');
        acc
    });

    vec![
        "std|string|String|",
        "core|string|String|",
        "string|String|",
        "String|",
    ]
    .into_iter()
    .any(|s| idents_of_path == *s)
}
