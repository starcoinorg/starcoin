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
                    pub fn #name(&self, #args)-> impl Future<Output=network_rpc_core::Result::<#returns>> {
                        let network = self.network.clone();
                        async move {
                            let input_arg_serialized = match #user_arg_indent.encode(){
                                Ok(arg_ser) => arg_ser,
                                Err(e) => {return Err(anyhow::anyhow!("Failed to encode rpc input argument: {:?}", e).into())}
                            };
                            debug!("Network rpc call method: {:?}, args: {:?}", stringify!(#name), #user_arg_indent);
                            let peer_id = match PeerId::from_bytes(#peer_id_indent.into_bytes()){
                                Ok(peer_id) => peer_id,
                                Err(e) => {return Err(anyhow::anyhow!("Invalid rpc peer id:{:?}",e).into())}
                            };

                            let rpc_path = stringify!(#name).to_string();
                            match Self::request(network, rpc_path, peer_id, input_arg_serialized).await{
                                Ok(result) => {
                                    match from_bytes::<network_rpc_core::Result::<#returns>>(&result){
                                        Ok(r) => r,
                                        Err(e) => Err(e.into()),
                                    }
                                },
                                Err(e) => {Err(e.into())}
                            }
                        }
                    }                      
                })
            } else {
                None
            }
        })
        .collect();
    let get_rpc_info_method = quote! {
        pub fn get_rpc_info() -> (&'static [u8], Vec<String>) {
           (CHAIN_PROTOCOL_NAME, vec![#(stringify!(#rpc_info).to_string()),*])
        }
    };

    let client_mod = quote! {
    pub mod gen_client{
        use super::*;
        use std::time::Duration;
        use network_api::{NetworkService};
        use starcoin_types::peer_info::{PeerId, PeerInfo};
        use starcoin_types::CHAIN_PROTOCOL_NAME;
        use scs::{SCSCodec,from_bytes};
        use futures::prelude::*;
        use starcoin_logger::prelude::*;
        #get_rpc_info_method

        #[derive(Clone)]
        pub struct NetworkRpcClient<N>
        where
            N: NetworkService,
        {
            network: N,
        }

        impl<N> NetworkRpcClient<N> where
            N: NetworkService, {
            pub fn new(network_service: N) -> Self {
                Self {
                    network: network_service
                }
            }
        }

        impl<N> NetworkRpcClient<N>
        where
            N: NetworkService,
        {
            async fn request(network: N, path: String, peer_id: PeerId, request: Vec<u8>) -> anyhow::Result<Vec<u8>> {
                network
                    .send_request_bytes(
                        CHAIN_PROTOCOL_NAME.into(),
                        peer_id.into(),
                        path,
                        request,
                        Duration::from_secs(DELAY_TIME)
                    )
                    .await
            }
            #(#client_methods)*
        }

    }};
    Ok(client_mod)
}
