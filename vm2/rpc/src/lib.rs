use crate::account_api::AccountApiClient;
/// this is in cargo expand contract_api.rs
use crate::contract_api::ContractApiClient;
use crate::state_api::StateApiClient;
use anyhow::anyhow;
use jsonrpc_core_client::{RawClient, RpcChannel};

#[allow(dead_code)]
#[derive(Clone)]
pub struct RpcClientInner {
    raw_client: RawClient,
    account_client: AccountApiClient,
    contract_client: ContractApiClient,
    state_client: StateApiClient,
}

impl RpcClientInner {
    pub fn new(channel: RpcChannel) -> Self {
        Self {
            raw_client: channel.clone().into(),
            account_client: channel.clone().into(),
            contract_client: channel.clone().into(),
            state_client: channel.clone().into(),
        }
    }
}

#[allow(dead_code)]
fn map_err(rpc_err: jsonrpc_client_transports::RpcError) -> anyhow::Error {
    anyhow!(format!("{}", rpc_err))
}

impl From<RpcChannel> for RpcClientInner {
    fn from(channel: RpcChannel) -> Self {
        Self::new(channel)
    }
}

pub mod account_api;
pub mod contract_api;
pub mod state_api;
