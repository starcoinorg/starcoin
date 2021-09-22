// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::helper::{compute_arg_identifiers, compute_args, compute_returns};
use proc_macro2::TokenStream;
use syn::ItemTrait;

pub fn generate_client_module(rpc_trait: &ItemTrait) -> anyhow::Result<TokenStream> {
    let mut rpc_info = Vec::new();
    let client_methods: Vec<TokenStream> = rpc_trait
        .items
        .iter()
        .filter_map(|trait_item| {
            if let syn::TraitItem::Method(method) = trait_item {
                let name = &method.sig.ident;
                let args = compute_args(method);
                let arg_names = compute_arg_identifiers(&args).unwrap();
                let returns = match compute_returns(method) {
                    Ok(r) => r,
                    Err(e) => panic!("{}", e)
                };
                if arg_names.len() < 2 {
                    panic!("network Rpc method must has at least two argument");
                }
                let peer_id_indent = arg_names[0];
                // TODO: Only support one user custom argument currently
                let user_arg_indent = arg_names[1];
                rpc_info.push(name.clone());
                Some(quote! {
                    pub fn #name(&self, #args)-> BoxFuture<anyhow::Result::<#returns>> {
                        async move {
                            let input_arg_serialized = match #user_arg_indent.encode(){
                                Ok(arg_ser) => arg_ser,
                                Err(e) => {return Err(anyhow::anyhow!("Failed to encode rpc input argument: {:?}", e).into())}
                            };

                            let peer_id = #peer_id_indent;
                            info!("[network-rpc] call method: {:?}, peer_id:{:?} args: {:?} ", stringify!(#name), peer_id, #user_arg_indent);
                            let rpc_path = Cow::from(stringify!(#name).to_string());
                            let result = self.request(peer_id.clone(), rpc_path, input_arg_serialized).await;
                            match result {
                                Ok(result) => {
                                    let result = from_bytes::<network_rpc_core::Result::<Vec<u8>>>(&result);
                                    match result {
                                        Ok(r) => match r {
                                            Ok(v) => {
                                                let result = from_bytes::<#returns>(&v);
                                                debug!("[network-rpc] response: {} {:?} ", peer_id, result);
                                                result
                                            },
                                            Err(e) => {
                                                error!("[network-rpc] response error: {} {:?}", peer_id, e);
                                                Err(e).with_context(|| peer_id)
                                            },
                                        },
                                        Err(e) => {
                                            error!("[network-rpc] response error: {} {:?} ", peer_id, e);
                                            Err(e)
                                        },
                                    }
                                },
                                Err(e) => {
                                    error!("[network-rpc] response error: {:?} ", e);
                                    Err(e)
                                }
                            }
                            }.boxed()}
                })
            } else {
                None
            }
        })
        .collect();
    let get_rpc_info_method = quote! {
        pub fn get_rpc_info() -> Vec<&'static str> {
            vec![#(stringify!(#rpc_info)),*]
        }
    };

    let client_mod = quote! {
    pub mod gen_client{
        use super::*;
        use std::time::Duration;
        use network_rpc_core::export::bcs_ext::{BCSCodec,from_bytes};
        use network_rpc_core::export::log::*;
        use futures::prelude::*;
        use network_rpc_core::{RawRpcClient, PeerId};
        use std::sync::Arc;
        use network_rpc_core::NetRpcError;
        use anyhow::Context;
        use std::borrow::Cow;
        #get_rpc_info_method

        #[derive(Clone)]
        pub struct NetworkRpcClient
        {
            raw_client: Arc<dyn RawRpcClient + Send + Sync>,
        }

        impl NetworkRpcClient {
            pub fn new<C>(raw_rpc_client: C) -> Self where C: RawRpcClient + Send + Sync +'static {
                Self {
                    raw_client: Arc::new(raw_rpc_client),
                }
            }
        }

        impl NetworkRpcClient {
            async fn request(&self, peer_id: PeerId, path: Cow<'static, str>, request: Vec<u8>) -> anyhow::Result<Vec<u8>> {
                    self.raw_client
                    .send_raw_request(
                        peer_id,
                        path,
                        request,
                    )
                    .await
            }
            #(#client_methods)*
        }

    }};
    Ok(client_mod)
}
