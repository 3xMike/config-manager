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
            let span = self.span;
            let name = self.long.clone();
            let long = quote_spanned!(span=> .long(#name));
            let short = match &self.short {
                None => TokenStream::new(),
                Some(short) => quote_spanned!(span=> .short(#short)),
            };
            let flag = if self.flag {
                quote_spanned!(span=> .num_args(0..=1).default_missing_value("true"))
            } else {
                quote_spanned!(span=> .num_args(1))
            };
            let help = match &self.help {
                None => TokenStream::new(),
                Some(help) => quote_spanned!(span=> .help(#help)),
            };
            let long_help = match &self.long_help {
                None => TokenStream::new(),
                Some(long_help) => quote_spanned!(span=> .long_help(#long_help)),
            };
            let help_heading = match &self.help_heading {
                None => TokenStream::new(),
                Some(help_heading) => quote_spanned!(span=> .help_heading(#help_heading)),
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

#[derive(Clone)]
pub(crate) struct ExtractedAttributes {
    pub(crate) span: Span,
    pub(crate) variables: Vec<FieldAttribute>,
    pub(crate) default: Option<Default>,
    pub(crate) deserializer: Option<(TokenStream, Span)>,
}

impl std::default::Default for ExtractedAttributes {
    fn default() -> Self {
        Self {
            span: Span::call_site(),
            variables: vec![],
            default: None,
            deserializer: None,
        }
    }
}

impl ExtractedAttributes {
    fn deserializer(&self) -> TokenStream {
        let span = self.span;
        match &self.deserializer {
            None => quote_spanned! {span=>
                let value = if value.is_empty() {
                    "\"\"".to_string()
                } else {
                    value
                };
                ::config_manager::__private::deser_hjson::from_str(&value)
            },
            Some((deser_fn, span)) => {
                let ident = Ident::new(deser_fn.to_string().trim_matches('\"'), *span);
                quote_spanned! {*span=> (#ident)(&value) }
            }
        }
    }

    fn gen_err(&self, field_name: &str) -> TokenStream {
        let span = self.span;
        let err = format!(
            "field {field_name} not found nor in {} nor as a default",
            self.variables
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .as_slice()
                .join(", ")
        );
        quote_spanned! {span=>
            ::config_manager::Error::MissingArgument(#err.to_string())
        }
    }

    fn gen_rest_init(&self, field_name: &str) -> TokenStream {
        let default_span = self.span;
        self.variables.iter().fold(
            quote_spanned!(default_span=> ::std::option::Option::<::std::string::String>::None),
            |acc, attribute_init| {
                let attribute_init = attribute_init.gen_init(field_name);
                let span = attribute_init.span();
                quote_spanned! {span=>
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

    pub(super) fn gen_init(&self, field: &Field) -> TokenStream {
        let field_name = field.ident.clone().unwrap().to_string();
        let tp = &field.ty;
        let default_initialization = match &self.default {
            None => quote_spanned!(field.span()=> ::std::option::Option::None),
            Some(Default { inner: None }) => {
                quote_spanned!(field.span()=> ::std::option::Option::Some::<#tp>(
                    ::std::default::Default::default()
                ))
            }
            Some(Default { inner: Some(def) }) => quote_spanned! {field.span()=>
                ::std::option::Option::Some::<#tp>(#def)
            },
        };
        let deserializer = self.deserializer();
        let rest = self.gen_rest_init(&field_name);
        let missing_err = self.gen_err(&field_name);

        quote_spanned! {field.span()=>
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
    fn span(&self) -> Span {
        match self {
            Self::Clap(v) => v.span,
            Self::Env(v) => v.span,
            Self::Config(v) => v.span,
        }
    }

    fn gen_init(&self, field_name: &str) -> TokenStream {
        let span = self.span();
        match &self {
            Self::Env(env) => {
                let prefixed_name = env.prefixed_name(field_name);
                quote_spanned! {span=>
                    env_data.get(&(#prefixed_name) as &::std::primitive::str).map(::std::string::ToString::to_string)
                }
            }
            Self::Config(cfg) => {
                let table = cfg.table();
                let key = cfg.key(field_name);
                quote_spanned! {span=>
                    ::config_manager::__private::find_field_in_table(config_file_data, #table, #key.to_string())?
                }
            }
            Self::Clap(clap) => {
                let long = clap.normal_long(field_name);
                quote_spanned! {span=>
                    clap_data.get_one::<::std::string::String>(#long).map(::std::string::ToString::to_string)
                }
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

#[derive(Clone)]
pub(crate) struct Env {
    pub(super) inner: Option<TokenStream>,
    pub(super) span: Span,
}

impl std::default::Default for Env {
    fn default() -> Self {
        Self {
            inner: None,
            span: Span::call_site(),
        }
    }
}

impl Env {
    fn prefixed_name(&self, field_name: &str) -> TokenStream {
        let span = self.span;
        let env_attribute = match &self.inner {
            None => quote_spanned!(span=> ::std::option::Option::<&::std::primitive::str>::None),
            Some(value) => {
                quote_spanned!(span=> ::std::option::Option::<&::std::primitive::str>::Some(#value))
            }
        };
        let binary_name = binary_name();
        let field_name_lowercase = str_to_tokens(field_name.to_lowercase(), span);

        quote_spanned! {span=>
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
    }
}

#[derive(Clone)]
pub(crate) struct Config {
    span: Span,
    pub(super) key: Option<TokenStream>,
    pub(super) table: Option<TokenStream>,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            span: Span::call_site(),
            key: None,
            table: None,
        }
    }
}

impl Config {
    fn key(&self, field_name: &str) -> TokenStream {
        self.key
            .clone()
            .unwrap_or_else(|| str_to_tokens(field_name, self.span))
    }
    fn table(&self) -> TokenStream {
        self.table
            .clone()
            .map(|table| quote_spanned!(table.span()=> ::std::option::Option::Some(#table.to_string())))
            .unwrap_or_else(|| quote_spanned!(self.span=> ::std::option::Option::None))
    }
}

#[derive(Default, Clone)]
pub(crate) struct Default {
    pub(super) inner: Option<TokenStream>,
}

pub(super) fn extract_attributes(
    field: &Field,
    table_name: &Option<TokenStream>,
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
                let mut default_init = extract_default(&arg)?;
                if is_string {
                    default_init = default_init
                        .map(|d| quote_spanned!(d.span()=> ::std::convert::Into::into(#d)));
                }
                res.default = Some(Default {
                    inner: default_init,
                })
            }
            ENV_KEY => res.variables.push(FieldAttribute::Env(Env {
                inner: meta_to_option(&arg)?,
                span: arg.span(),
            })),
            CONFIG_KEY => res.variables.push(FieldAttribute::Config(Config {
                span: arg.span(),
                key: meta_to_option(&arg)?,
                table: table_name.clone(),
            })),
            DESERIALIZER => {
                if res.deserializer.is_some() {
                    panic_span!(
                        arg.span(),
                        "deserialize_with can be assigned only once per field"
                    )
                }
                if matches!(arg, Meta::Path(_)) {
                    panic_span!(arg.span(), "deserialize_with can't be empty")
                }
                res.deserializer = meta_to_option(&arg)?.map(|val| (val, arg.span()));
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
