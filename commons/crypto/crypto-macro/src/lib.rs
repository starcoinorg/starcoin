// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Auto derive CryptoHash implement.
extern crate proc_macro;

mod hasher;
mod unions;

use crate::hasher::camel_to_snake;
use crate::unions::get_type_from_attrs;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(CryptoHash)]
pub fn crypto_hash(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let name = &item.ident;
    let hasher_name = Ident::new(&format!("{}Hasher", name.to_string()), Span::call_site());

    let out = quote!(

        impl starcoin_crypto::hash::CryptoHash for #name {
            type Hasher = #hasher_name;
            fn hash(&self) -> starcoin_crypto::HashValue {
                let mut state = Self::Hasher::default();
                state.write(scs::to_bytes(self)
                    .expect("Serialization should work.")
                    .as_slice());
                state.finish()
            }
        }

    );
    out.into()
}

#[proc_macro_derive(CryptoHasher, attributes(CryptoHasherSalt))]
pub fn hasher_dispatch(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let hasher_name = Ident::new(
        &format!("{}Hasher", &item.ident.to_string()),
        Span::call_site(),
    );
    let snake_name = camel_to_snake(&item.ident.to_string());
    let static_hasher_name = Ident::new(
        &format!("{}_HASHER", snake_name.to_uppercase()),
        Span::call_site(),
    );
    let fn_name = get_type_from_attrs(&item.attrs, "CryptoHasherSalt")
        .unwrap_or_else(|_| syn::LitStr::new(&item.ident.to_string(), Span::call_site()));

    let out = quote!(
        #[derive(Clone)]
        pub struct #hasher_name(starcoin_crypto::hash::DefaultHasher);

        impl #hasher_name {
            fn new() -> Self {
                let mp = module_path!();
                let f_name = #fn_name;

                #hasher_name(
                    starcoin_crypto::hash::DefaultHasher::new_with_salt(&format!("{}::{}", f_name, mp).as_bytes()))
            }
        }

        static #static_hasher_name: starcoin_crypto::_once_cell::sync::Lazy<#hasher_name> =
            starcoin_crypto::_once_cell::sync::Lazy::new(|| #hasher_name::new());


        impl std::default::Default for #hasher_name
        {
            fn default() -> Self {
                #static_hasher_name.clone()
            }
        }

        impl starcoin_crypto::hash::CryptoHasher for #hasher_name {
            fn finish(self) -> HashValue {
                self.0.finish()
            }

            fn write(&mut self, bytes: &[u8]) -> &mut Self {
                self.0.write(bytes);
                self
            }
        }

    );
    out.into()
}
