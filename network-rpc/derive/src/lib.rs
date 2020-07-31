// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod helper;
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
pub fn net_rpc(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input_tokens = parse_macro_input!(input as syn::Item);
    let token_stream = rpc_trait::rpc_impl(input_tokens).unwrap();
    token_stream.into()
}
