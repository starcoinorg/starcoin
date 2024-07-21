// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use starcoin_crypto::HashValue;
use starcoin_service_registry::ServiceRequest;
use starcoin_types::block::{Block, ExecutedBlock};

mod block_connector_service;
mod metrics;
#[cfg(test)]
mod test_illegal_block;
#[cfg(test)]
mod test_write_block_chain;
#[cfg(test)]
mod test_write_dag_block_chain;
mod write_block_chain;

pub use block_connector_service::BlockConnectorService;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
pub use write_block_chain::WriteBlockChainService;
#[cfg(test)]
use {
    starcoin_types::transaction::SignedUserTransaction,
    starcoin_vm_types::account_address::AccountAddress,
};

#[cfg(test)]
pub use test_write_block_chain::create_writeable_block_chain;

#[derive(Debug, Clone)]
pub struct ResetRequest {
    pub block_hash: HashValue,
}

impl ServiceRequest for ResetRequest {
    type Response = anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct ExecuteRequest {
    pub block: Block,
}

impl ServiceRequest for ExecuteRequest {
    type Response = anyhow::Result<ExecutedBlock>;
}

#[derive(Clone, Debug)]
pub struct MinerRequest {}

#[derive(Clone, Debug)]
pub struct MinerResponse {
    pub previous_header: BlockHeader,
    pub tips_hash: Vec<HashValue>,
    pub blues_hash: Vec<HashValue>,
    pub strategy: ConsensusStrategy,
    pub on_chain_block_gas_limit: u64,
    pub next_difficulty: U256,
    pub now_milliseconds: u64,
    pub pruning_point: HashValue,
}

impl ServiceRequest for MinerRequest {
    type Response = anyhow::Result<Box<MinerResponse>>;
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct CreateBlockRequest {
    pub count: u64,
    pub author: AccountAddress,
    pub parent_hash: Option<HashValue>,
    pub user_txns: Vec<SignedUserTransaction>,
    pub uncles: Vec<BlockHeader>,
    pub block_gas_limit: Option<u64>,
    pub tips: Vec<HashValue>,
}

#[cfg(test)]
#[derive(Clone, Debug)]
pub struct CreateBlockResponse;

#[cfg(test)]
impl ServiceRequest for CreateBlockRequest {
    type Response = anyhow::Result<CreateBlockResponse>;
}

#[cfg(test)]
#[derive(Debug, Clone)]
pub struct CheckBlockConnectorHashValue {
    pub head_hash: HashValue,
    pub number: u64,
}

#[cfg(test)]
impl ServiceRequest for CheckBlockConnectorHashValue {
    type Response = anyhow::Result<()>;
}
