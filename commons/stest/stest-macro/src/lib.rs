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
/// ```no_run
/// use std::time::Duration;
/// use actix::{Context, Actor};
/// #[stest::test]
/// async fn my_async_test() {
///     assert!(true);
/// }
///
/// #[stest::test]
/// async fn my_async_test_with_result() -> anyhow::Result<()> {
///     assert!(true);
///     Ok(())
/// }
///
/// #[stest::test]
/// fn my_sync_test() {
///     assert!(true);
/// }
///
///
/// #[stest::test]
/// fn my_sync_test_false() {
///     assert!(false);
/// }
///
/// #[stest::test(timeout = 1)]
/// fn test_timeout() {
///     std::thread::sleep(Duration::from_secs(6));
/// }
///
/// #[stest::test(timeout = 1)]
/// fn test_ok() -> anyhow::Result<()> {
///     Ok(())
/// }
///
/// #[stest::test(timeout = 1)]
/// fn test_timeout_result() -> anyhow::Result<()> {
///     std::thread::sleep(Duration::from_secs(6));
///     Ok(())
/// }
///
/// #[stest::test(timeout = 1)]
/// async fn test_async_timeout() {
///     stest::actix_export::time::delay_for(Duration::from_secs(6)).await;
/// }
///
/// #[stest::test(timeout = 1)]
/// async fn test_async_timeout_result() -> anyhow::Result<()> {
///     stest::actix_export::time::delay_for(Duration::from_secs(6)).await;
///     Ok(())
/// }
///
/// #[stest::test(timeout = 10)]
/// async fn my_async_test_false() {
///     actix::clock::delay_for(Duration::from_secs(1)).await;
///     assert!(false);
/// }
///
/// struct MyActor;
/// impl Actor for MyActor {
///     type Context = Context<Self>;
/// }
///
/// #[stest::test(timeout = 10)]
/// async fn my_async_test_actor() {
///     let myactor = MyActor {};
///     myactor.start();
/// }
///
///
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

    let timeout: u64 = args.timeout.unwrap_or(60);
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

                    stest::timeout(#timeout,move ||{
                        #body;
                    },tx);

                    stest::wait_channel(rx)
                }
            }
        } else {
            quote! {
                #[test]
                #(#attrs)*
                fn #name() #ret {
                    stest::init_test_logger();
                    let (tx,rx) = std::sync::mpsc::channel();

                    stest::timeout(#timeout,move ||{
                        #body
                    },tx);

                    stest::wait_channel(rx)
                }
            }
        }
    } else if has_test_attr {
        quote! {
            #(#attrs)*
            fn #name() #ret {
                stest::init_test_logger();
                let (tx,mut rx) = stest::make_channel();

                let mut rt = stest::Runtime::new().expect("Tokio runtime");

                let local = stest::LocalSet::new();
                let future = stest::actix_export::System::run_in_tokio("test", &local);
                local.spawn_local(future);

                stest::actix_export::Arbiter::spawn(stest::timeout_future(#timeout,tx.clone()));
                stest::actix_export::Arbiter::spawn(stest::test_future(async{ #body },tx));

                local.block_on(&mut rt,stest::wait_result(rx))
            }
        }
    } else {
        quote! {
            #[test]
            #(#attrs)*
            fn #name() #ret {
                stest::init_test_logger();
                let (tx,mut rx) = stest::make_channel();

                let mut rt = stest::Runtime::new().expect("Tokio runtime");

                let local = stest::LocalSet::new();
                let future = stest::actix_export::System::run_in_tokio("test", &local);
                local.spawn_local(future);

                stest::actix_export::Arbiter::spawn(stest::timeout_future(#timeout,tx.clone()));
                stest::actix_export::Arbiter::spawn(stest::test_future(async{ #body },tx));

                local.block_on(&mut rt,stest::wait_result(rx))
             }
        }
    };

    result.into()
}
