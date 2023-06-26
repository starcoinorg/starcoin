// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod helper;
mod options;
mod rpc_trait;
mod to_client;
mod to_server;

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use proc_macro::TokenStream;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn net_rpc(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_tokens = parse_macro_input!(input as syn::Item);
    let args = syn::parse_macro_input!(attr as syn::AttributeArgs);
    let derive_options = options::DeriveOptions::new(args);
    let token_stream = rpc_trait::rpc_impl(input_tokens, &derive_options).unwrap();
    token_stream.into()
}
