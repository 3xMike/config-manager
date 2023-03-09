// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

pub(crate) fn binary_name() -> TokenStream {
    quote! {
        {
            (|| -> ::std::result::Result<::std::string::String, ::config_manager::Error> {
                ::std::result::Result::Ok(
                    ::std::env::current_exe()
                        .map_err(|err| {
                            ::config_manager::Error::ExternalError(::std::format!(
                                "failed to retrieve path to current exe: {}",
                                err
                            ))
                        })?
                        .file_name()
                        .ok_or_else(|| {
                            ::config_manager::Error::ExternalError(
                                "failed to read file name of current exe".to_string(),
                            )
                        })?
                        .to_str()
                        .ok_or_else(|| {
                            ::config_manager::Error::ExternalError(
                                "OS file name of current exe is not valid UTF".to_string(),
                            )
                        })?
                        .to_string(),
                )
            })()
        }
    }
}

pub(crate) fn path_to_string(path: &Path) -> String {
    let segments = &path.segments;
    assert_eq!(
        segments.len(),
        1,
        "not a single segment in path: {:?}",
        path
    );
    segments
        .first()
        .unwrap()
        .ident
        .to_token_stream()
        .to_string()
}

pub(crate) fn compare_attribute_name(a: &Attribute, name: &str) -> bool {
    name == path_to_string(&a.path)
}

pub(crate) fn is_type_an_optional(ty: &Type) -> Option<Type> {
    let ty = ty.clone();
    let path = match ty {
        Type::Path(path) if path.qself.is_none() => path.path,
        _ => return None,
    };
    let idents_of_path = path
        .segments
        .iter()
        .into_iter()
        .fold(String::new(), |mut acc, v| {
            acc.push_str(&v.ident.to_string());
            acc.push('|');
            acc
        });

    let option_segment = vec![
        "std|option|Option|",
        "core|option|Option|",
        "option|Option|",
        "Option|",
    ]
    .into_iter()
    .find(|s| idents_of_path == *s)
    .and_then(|_| path.segments.last());

    option_segment
        .and_then(|path_seg| match path_seg.arguments {
            PathArguments::AngleBracketed(ref params) => params.args.first(),
            _ => None,
        })
        .and_then(|generic_arg| match generic_arg {
            GenericArgument::Type(ty) => Some(ty.clone()),
            _ => None,
        })
}
