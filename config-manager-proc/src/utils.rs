// SPDX-License-Identifier: MIT
// Copyright (c) 2022 JSRPC “Kryptonite”

use crate::*;

mod attributes;
pub(crate) mod config;
pub(crate) mod field;
pub(crate) mod parser;
pub(crate) mod top_level;

/// Formated string to TokenStream \
/// Same as ```TokenStream::from_str(&format!(...)).unwrap()```
macro_rules! format_to_tokens {
    ($($arg:tt)*) => {
        TokenStream::from_str(&std::format!($($arg)*)).unwrap()
    };
}

pub(crate) use format_to_tokens;

pub(crate) fn option_to_tokens(opt: &Option<String>) -> TokenStream {
    match opt {
        None => quote!(::std::option::Option::None),
        Some(pref) => {
            quote!(::std::option::Option::<::std::string::String>::Some(#pref.to_string()))
        }
    }
}
