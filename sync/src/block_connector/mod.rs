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
mod write_block_chain;

pub use block_connector_service::BlockConnectorService;
pub use write_block_chain::WriteBlockChainService;

#[cfg(test)]
pub use test_write_block_chain::create_writeable_block_chain;
#[cfg(test)]
pub use test_write_block_chain::gen_blocks;
#[cfg(test)]
pub use test_write_block_chain::new_block;

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
