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
                let args = compute_args(&method);
                let arg_names = compute_arg_identifiers(&args).unwrap();
                let returns = match compute_returns(method) {
                    Ok(r) => r,
                    Err(e) => panic!(e)
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
                            debug!("Network rpc call method: {:?}, peer_id:{:?} args: {:?} ", stringify!(#name), peer_id, #user_arg_indent);
                            let rpc_path = stringify!(#name).to_string();
                            match self.request(peer_id, rpc_path, input_arg_serialized).await{
                                Ok(result) => {
                                    match from_bytes::<network_rpc_core::Result::<Vec<u8>>>(&result){
                                        Ok(r) => match r{
                                            Ok(v) => {
                                                from_bytes::<#returns>(&v)
                                            },
                                            Err(e) => Err(e.into()),
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                Err(e) => Err(e)
                            }
                            }.boxed()}
                })
            } else {
                None
            }
        })
        .collect();
    let get_rpc_info_method = quote! {
        pub fn get_rpc_info() -> Vec<String> {
            vec![#(stringify!(#rpc_info).to_string()),*]
        }
    };

    let client_mod = quote! {
    pub mod gen_client{
        use super::*;
        use std::time::Duration;
        use network_rpc_core::export::scs::{SCSCodec,from_bytes};
        use network_rpc_core::export::log::*;
        use futures::prelude::*;
        use network_rpc_core::{RawRpcClient, PeerId};
        use std::sync::Arc;
        use network_rpc_core::NetRpcError;

        #get_rpc_info_method

        #[derive(Clone)]
        pub struct NetworkRpcClient
        {
            raw_client: Arc<dyn RawRpcClient + Send + Sync>,
            timeout: Duration,
        }

        impl NetworkRpcClient {
            pub fn new<C>(raw_rpc_client: C) -> Self where C: RawRpcClient + Send + Sync +'static {
                Self {
                    raw_client: Arc::new(raw_rpc_client),
                    //TODO support custom timeout.
                    timeout: Duration::from_secs(15),
                }
            }

             pub fn new_with_timeout<C>(raw_rpc_client: C, timeout: Duration) -> Self where C: RawRpcClient + Send + Sync +'static {
                Self {
                    raw_client: Arc::new(raw_rpc_client),
                    timeout,
                }
            }
        }

        impl NetworkRpcClient {
            async fn request(&self, peer_id: PeerId, path: String, request: Vec<u8>) -> anyhow::Result<Vec<u8>> {
                    self.raw_client
                    .send_raw_request(
                        Some(peer_id),
                        path,
                        request,
                        self.timeout,
                    )
                    .await
            }
            #(#client_methods)*
        }

    }};
    Ok(client_mod)
}
