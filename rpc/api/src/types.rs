// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod contract_call;
pub mod pubsub;

pub use contract_call::ContractCall;
use jsonrpc_core_client::RpcChannel;
use starcoin_service_registry::ServiceRequest;

#[derive(Debug, Clone)]
pub struct ConnectLocal;

impl ServiceRequest for ConnectLocal {
    type Response = RpcChannel;
}
