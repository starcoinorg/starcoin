// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Auto derive CryptoHash implement.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(CryptoHash)]
pub fn crypto_hash(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).expect("Incorrect macro input");
    let name = &ast.ident;
    let out = quote!(

        impl starcoin_crypto::hash::CryptoHash for #name {
            fn crypto_hash(&self) -> starcoin_crypto::HashValue {
                starcoin_crypto::HashValue::from_sha3_256(
                    scs::to_bytes(self)
                    .expect("Serialization should work.")
                    .as_slice()
                )
            }
        }

    );
    out.into()
}
