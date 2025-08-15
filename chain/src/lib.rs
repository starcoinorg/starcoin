// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(clippy::arithmetic_side_effects)]
mod chain;
mod fixed_blocks;
pub mod verifier;
use std::sync::Arc;

pub use chain::BlockChain;
use starcoin_accumulator::{node::AccumulatorStoreType, Accumulator, MerkleAccumulator};
pub use starcoin_chain_api::{ChainReader, ChainWriter};

pub use starcoin_data_migration::*;

use anyhow::{format_err, Result};
use starcoin_crypto::HashValue;
use starcoin_dag::blockdag::BlockDAG;
use starcoin_storage::Store;
use starcoin_types::block::BlockHeader;

pub fn block_merkle_tree_from_header(
    header: &BlockHeader,
    storage: Arc<dyn starcoin_storage::Store>,
) -> Result<MerkleAccumulator> {
    let block_info = storage
        .get_block_info(header.id())?
        .ok_or_else(|| format_err!("Can not find block info by hash {}", header.id()))?;
    let block_accumulator_info = block_info.get_block_accumulator_info();
    let block_accumulator = MerkleAccumulator::new_with_info(
        block_accumulator_info.clone(),
        storage.get_accumulator_store(AccumulatorStoreType::Block),
    );
    Ok(block_accumulator)
}

pub fn get_merge_bound_hash(
    selected_parent: HashValue,
    dag: BlockDAG,
    storage: Arc<dyn Store>,
) -> Result<HashValue> {
    let header = storage
        .get_block_header_by_hash(selected_parent)?
        .ok_or_else(|| {
            format_err!(
                "Cannot find block header by hash {:?} when get merge bound hash",
                selected_parent
            )
        })?;
    let merge_depth = dag.block_depth_manager().merge_depth();
    if header.number() <= merge_depth {
        return Ok(storage.get_genesis()?.expect("genesis cannot be none"));
    }
    let merge_depth_index = (header.number().checked_div(merge_depth))
        .ok_or_else(|| format_err!("checked_div error"))?
        .checked_mul(merge_depth)
        .ok_or_else(|| format_err!("checked_mul error"))?;
    let block_accumulator = block_merkle_tree_from_header(&header, storage.clone())?;
    let leaf_hash = block_accumulator
        .get_leaf(merge_depth_index)?
        .ok_or_else(|| {
            format_err!(
                "cannot find hash at merge_depth_index {:?} from block_accumulator",
                merge_depth_index
            )
        })?;
    Ok(leaf_hash)
}
