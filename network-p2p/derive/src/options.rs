// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use syn::parse::Parser;

const CLIENT_META_WORD: &str = "client";
const SERVER_META_WORD: &str = "server";

#[derive(Debug)]
pub struct DeriveOptions {
    pub enable_client: bool,
    pub enable_server: bool,
}

impl DeriveOptions {
    pub fn parse(attr: TokenStream) -> Self {
        let mut options = DeriveOptions {
            enable_client: false,
            enable_server: false,
        };
        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident(CLIENT_META_WORD) {
                options.enable_client = true;
                return Ok(());
            }
            if meta.path.is_ident(SERVER_META_WORD) {
                options.enable_server = true;
                return Ok(());
            }
            Err(meta.error(format!(
                "Expecting identifier `{}` or `{}`",
                CLIENT_META_WORD, SERVER_META_WORD
            )))
        });
        if let Err(err) = parser.parse(attr) {
            panic!("{err}");
        }
        if !options.enable_client && !options.enable_server {
            options.enable_client = true;
            options.enable_server = true;
        }
        options
    }
}
