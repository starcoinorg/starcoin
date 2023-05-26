// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

const CLIENT_META_WORD: &str = "client";
const SERVER_META_WORD: &str = "server";

#[derive(Debug)]
pub struct DeriveOptions {
    pub enable_client: bool,
    pub enable_server: bool,
}

impl DeriveOptions {
    pub fn new(args: syn::AttributeArgs) -> Self {
        let mut options = DeriveOptions {
            enable_client: false,
            enable_server: false,
        };
        for arg in args {
            if let syn::NestedMeta::Meta(meta) = arg {
                match meta {
                    syn::Meta::Path(ref p) => {
                        match p
                            .get_ident()
                            .ok_or_else(|| {
                                syn::Error::new_spanned(
                                    p,
                                    format!(
                                        "Expecting identifier `{}` or `{}`",
                                        CLIENT_META_WORD, SERVER_META_WORD
                                    ),
                                )
                            })
                            .unwrap()
                            .to_string()
                            .as_ref()
                        {
                            CLIENT_META_WORD => options.enable_client = true,
                            SERVER_META_WORD => options.enable_server = true,
                            _ => {}
                        };
                    }
                    _ => panic!("Unexpected use of RPC attribute macro"),
                }
            }
        }
        if !options.enable_client && !options.enable_server {
            options.enable_client = true;
            options.enable_server = true;
        }
        options
    }
}
