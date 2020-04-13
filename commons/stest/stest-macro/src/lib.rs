// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The stest lib enhances the rust test framework with some useful functions.
//! Copy from /actix/actix-net/actix-macros and do some enhancement.
extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use quote::quote;

#[derive(Debug, FromMeta)]
struct TestAttributeOpts {
    #[darling(default)]
    timeout: Option<u64>,
}

/// Marks test function, support async and sync mehtod both.
/// The async test need actix.
/// ## Usage
///
/// ```no-run
/// #[stest::test]
/// async fn my_async_test() {
///     assert!(true);
/// }
/// #[stest::test]
/// fn my_async_test() {
///     assert!(true);
/// }
/// ```
#[proc_macro_attribute]
pub fn test(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = syn::parse_macro_input!(args as syn::AttributeArgs);
    let args = match TestAttributeOpts::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return e.write_errors().into();
        }
    };

    let timeout: u64 = match args.timeout {
        Some(t) => t,
        None => 60,
    };
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
                    let (tx,rx) = std::sync::mpsc::channel();
                    let tx_clone = tx.clone();

                    stest::timeout(#timeout,move ||{
                        #body
                        let _= tx.send(());
                    },tx_clone);

                    let _= rx.recv();
                    ()
                }
            }
        } else {
            quote! {
                #[test]
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    let (tx,rx) = std::sync::mpsc::channel();
                    let tx_clone = tx.clone();

                    stest::timeout(#timeout,move ||{
                        #body
                        let _= tx.send(());
                    },tx_clone);

                    let _= rx.recv();
                    ()
                }
            }
        }
    } else {
        if has_test_attr {
            quote! {
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    let (tx,mut rx) = stest::make_channel();

                    let mut system = actix_rt::System::new("test");
                    actix_rt::Arbiter::spawn(stest::timeout_future(#timeout,tx.clone()));
                    actix_rt::Arbiter::spawn(stest::test_future(async{ #body },tx));

                    system.block_on(stest::wait_result(rx));
                }
            }
        } else {
            quote! {
                #[test]
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    let (tx,mut rx) = stest::make_channel();

                    let mut system = actix_rt::System::new("test");
                    actix_rt::Arbiter::spawn(stest::timeout_future(#timeout,tx.clone()));
                    actix_rt::Arbiter::spawn(stest::test_future(async{ #body },tx));

                    system.block_on(stest::wait_result(rx));
                 }
            }
        }
    };

    result.into()
}
