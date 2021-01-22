// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Auto derive CryptoHash implement.
extern crate proc_macro;

mod hasher;

use crate::hasher::camel_to_snake;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::iter::FromIterator;
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(CryptoHash)]
pub fn crypto_hash(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let name = &item.ident;
    let hasher_name = Ident::new(&format!("{}Hasher", name.to_string()), Span::call_site());
    let error_msg = syn::LitStr::new(
        &format!("Serialization of {} should not fail", name.to_string()),
        Span::call_site(),
    );

    let out = quote!(

        impl starcoin_crypto::hash::CryptoHash for #name {
            type Hasher = #hasher_name;
            fn hash(&self) -> starcoin_crypto::HashValue {
                let mut state = Self::Hasher::default();
                bcs_ext::serialize_into(&mut state, &self).expect(#error_msg);
                state.finish()
            }
        }

    );
    out.into()
}

#[proc_macro_derive(CryptoHasher)]
pub fn hasher_dispatch(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let hasher_name = Ident::new(
        &format!("{}Hasher", &item.ident.to_string()),
        Span::call_site(),
    );
    let snake_name = camel_to_snake(&item.ident.to_string());
    let static_seed_name = Ident::new(
        &format!("{}_SEED", snake_name.to_uppercase()),
        Span::call_site(),
    );

    let static_hasher_name = Ident::new(
        &format!("{}_HASHER", snake_name.to_uppercase()),
        Span::call_site(),
    );
    let type_name = &item.ident;
    let param = if item.generics.params.is_empty() {
        quote!()
    } else {
        let args = proc_macro2::TokenStream::from_iter(
            std::iter::repeat(quote!(())).take(item.generics.params.len()),
        );
        quote!(<#args>)
    };

    let out = quote!(
        #[derive(Clone)]
        pub struct #hasher_name(starcoin_crypto::hash::DefaultHasher);

        static #static_seed_name: starcoin_crypto::_once_cell::sync::OnceCell<[u8; 32]> = starcoin_crypto::_once_cell::sync::OnceCell::new();

        impl #hasher_name {
            fn new() -> Self {
                let name = starcoin_crypto::_serde_name::trace_name::<#type_name #param>()
                    .expect("The `CryptoHasher` macro only applies to structs and enums");
                #hasher_name(
                    starcoin_crypto::hash::DefaultHasher::new(&name.as_bytes()))
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
            fn seed() -> &'static [u8; 32] {
                #static_seed_name.get_or_init(|| {
                    let name = starcoin_crypto::_serde_name::trace_name::<#type_name #param>()
                        .expect("The `CryptoHasher` macro only applies to structs and enums.").as_bytes();
                    starcoin_crypto::hash::DefaultHasher::prefixed_hash(&name)
                })
            }

            fn update(&mut self, bytes: &[u8]) {
                self.0.update(bytes);
            }

            fn finish(self) -> starcoin_crypto::hash::HashValue {
                self.0.finish()
            }
        }

        impl std::io::Write for #hasher_name {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                use starcoin_crypto::hash::CryptoHasher;

                self.0.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

    );
    out.into()
}
