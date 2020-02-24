// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.
//! Copy from /actix/actix-net/actix-macros and do some enhancement.
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;

/// Marks test function, support async and sync mehtod both.
/// The async test need actix.
/// ## Usage
///
/// ```no_run
/// #[stest::test]
/// async fn my_async_test() {
///     assert!(true);
/// }
/// #[stest::test]
/// fn my_test(){
///     assert!(true);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(_: TokenStream, item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let ret = &input.sig.output;
    let name = &input.sig.ident;
    let body = &input.block;
    let attrs = &input.attrs;
    let mut has_test_attr = false;

    for attr in attrs {
        if attr.path.is_ident("test") {
            has_test_attr = true;
        }
    }

    let result = if input.sig.asyncness.is_none() {
        if has_test_attr {
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    #body
                }
            }
        } else {
            quote! {
                #[test]
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    #body
                }
            }
        }
    } else {
        if has_test_attr {
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    actix_rt::System::new("test")
                        .block_on(async { #body })
                }
            }
        } else {
            quote! {
                #[test]
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    actix_rt::System::new("test")
                        .block_on(async { #body })
                }
            }
        }
    };

    result.into()
}
