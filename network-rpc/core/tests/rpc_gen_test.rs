// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use futures::executor::block_on;
use futures::future::BoxFuture;
use futures::FutureExt;
use serde::{Deserialize, Serialize};

use network_rpc_core::server::NetworkRpcServer;
use network_rpc_core::{prelude::*, InmemoryRpcClient};

use crate::gen_client::NetworkRpcClient;
use crate::gen_server::KVRpc;

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct EchoStruct {
    pub msg: String,
}

#[net_rpc(client, server)]
#[allow(clippy::upper_case_acronyms)]
pub trait KVRpc: Sized + Send + Sync {
    fn echo_str(&self, peer_id: PeerId, req: String) -> BoxFuture<Result<String>>;
    fn echo_struct(&self, peer_id: PeerId, req: EchoStruct) -> BoxFuture<Result<EchoStruct>>;
    fn echo_err(&self, _peer_id: PeerId, req: String) -> BoxFuture<Result<String>>;
}

#[derive(Default)]
#[allow(clippy::upper_case_acronyms)]
struct KVRpcImpl {}

impl gen_server::KVRpc for KVRpcImpl {
    fn echo_str(&self, _peer_id: PeerId, req: String) -> BoxFuture<Result<String>> {
        futures::future::ready(Ok(req)).boxed()
    }
    fn echo_struct(&self, _peer_id: PeerId, req: EchoStruct) -> BoxFuture<Result<EchoStruct>> {
        futures::future::ready(Ok(req)).boxed()
    }
    fn echo_err(&self, _peer_id: PeerId, req: String) -> BoxFuture<Result<String>> {
        futures::future::ready(Err(NetRpcError::client_err(req).into())).boxed()
    }
}

#[stest::test]
fn test_rpc_str() {
    let rpc_impl = KVRpcImpl::default();
    let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());

    let rpc_client = NetworkRpcClient::new(InmemoryRpcClient::new(PeerId::random(), rpc_server));
    let request = "test".to_string();
    let result =
        block_on(async { rpc_client.echo_str(PeerId::random(), request.clone()).await }).unwrap();
    assert_eq!(result, request);
}

#[stest::test]
fn test_rpc_struct() {
    let rpc_impl = KVRpcImpl::default();
    let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());

    let rpc_client = NetworkRpcClient::new(InmemoryRpcClient::new(PeerId::random(), rpc_server));
    let request = EchoStruct {
        msg: "test".to_string(),
    };
    let result = block_on(async {
        rpc_client
            .echo_struct(PeerId::random(), request.clone())
            .await
    })
    .unwrap();
    assert_eq!(result, request);
}

#[stest::test]
fn test_rpc_err() {
    let rpc_impl = KVRpcImpl::default();
    let rpc_server = NetworkRpcServer::new(rpc_impl.to_delegate());

    let rpc_client = NetworkRpcClient::new(InmemoryRpcClient::new(PeerId::random(), rpc_server));
    let request = "test".to_string();
    let result = block_on(async { rpc_client.echo_err(PeerId::random(), request.clone()).await });
    match result {
        Err(e) => {
            let rpc_err: NetRpcError = e
                .downcast::<NetRpcError>()
                .expect("the error should NetRpcError");
            assert_eq!(rpc_err.message(), request.as_str());
        }
        Ok(_) => panic!("expect error, but get ok"),
    }
}

#[test]
fn test_result_serialize() {
    let str_result: network_rpc_core::Result<String, NetRpcError> =
        network_rpc_core::Result::Ok("test".to_string());
    let bytes = bcs_ext::to_bytes(&str_result).unwrap();
    println!("bytes:{:?}", bytes);
    let str_result2: network_rpc_core::Result<String, NetRpcError> =
        bcs_ext::from_bytes(bytes.as_slice()).unwrap();
    println!("result:{:?}", str_result2);
    assert_eq!(str_result, str_result2);
}
