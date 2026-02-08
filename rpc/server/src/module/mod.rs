// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

mod account_rpc;
mod chain_rpc;
mod contract_rpc;
mod debug_rpc;
mod helpers;
mod miner_rpc;
mod network_manager_rpc;
mod node_manager_rpc;
mod node_rpc;
mod state_rpc;
mod sync_manager_rpc;
mod txfactory_rpc;
mod txpool_rpc;

pub use self::account_rpc::AccountRpcImpl;
pub use self::chain_rpc::ChainRpcImpl;
pub use self::contract_rpc::ContractRpcImpl;
pub use self::debug_rpc::DebugRpcImpl;
pub use self::miner_rpc::MinerRpcImpl;
pub use self::network_manager_rpc::NetworkManagerRpcImpl;
pub use self::node_manager_rpc::NodeManagerRpcImpl;
pub use self::node_rpc::NodeRpcImpl;
pub use self::state_rpc::StateRpcImpl;
pub use self::sync_manager_rpc::SyncManagerRpcImpl;
pub use self::txfactory_rpc::TxFactoryStatusHandle;
pub use self::txpool_rpc::TxPoolRpcImpl;

pub fn map_err(err: anyhow::Error) -> anyhow::Error {
    err
}

pub fn convert_to_rpc_error<T: Into<anyhow::Error>>(err: T) -> anyhow::Error {
    err.into()
}

pub fn to_invalid_param_err<E>(err: E) -> anyhow::Error
where
    E: Into<anyhow::Error>,
{
    anyhow::anyhow!("Invalid param error: {:?}", err.into())
}
