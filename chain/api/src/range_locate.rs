// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use anyhow::format_err;
use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schemadb::ReachabilityStoreReader};
use starcoin_storage::Store;

use crate::ChainReader;

pub enum FindCommonHeader {
    AllInRange,                    // all in range
    InRange(HashValue, HashValue), // start in range but end not
    Found(HashValue),              // found the common header
    NotInRange,                    // all are not in reachability
}

pub enum FindCommonHeaderError {
    InvalidRange(String),  // the range is invalid, there must be something wrong
    CheckAncestor(String), // failed to check the hash whether in reachability
    RangeLen,              // the length of the range is not greater than 2
}

pub enum RangeInPruningPoint {
    NotInSelectedChain,
    InSelectedChain(HashValue, Vec<HashValue>),
}

pub fn find_common_header_in_range(
    dag: &BlockDAG,
    ranges: &[HashValue],
) -> std::result::Result<FindCommonHeader, FindCommonHeaderError> {
    if ranges.len() < 2 {
        return std::result::Result::Err(FindCommonHeaderError::RangeLen);
    }
    for range in ranges.windows(2) {
        let start = *range
            .first()
            .ok_or_else(|| FindCommonHeaderError::InvalidRange("error start index".to_string()))?;
        let end = *range
            .get(1)
            .ok_or_else(|| FindCommonHeaderError::InvalidRange("error end index".to_string()))?;

        match (
            dag.storage
                .reachability_store
                .read()
                .has(start)
                .map_err(|err| {
                    FindCommonHeaderError::CheckAncestor(format!(
                        "failed to check the reachability has for start for error: {:?}",
                        err
                    ))
                })?,
            dag.storage
                .reachability_store
                .read()
                .has(end)
                .map_err(|err| {
                    FindCommonHeaderError::CheckAncestor(format!(
                        "failed to check the reachability has for end for error: {:?}",
                        err
                    ))
                })?,
        ) {
            (true, true) => {
                continue;
            }
            (true, false) => return std::result::Result::Ok(FindCommonHeader::InRange(start, end)),
            (false, true) => {
                return std::result::Result::Err(FindCommonHeaderError::InvalidRange(
                    "the start is not in reachability but the end is".to_string(),
                ));
            }
            (false, false) => return std::result::Result::Ok(FindCommonHeader::NotInRange),
        }
    }

    Ok(FindCommonHeader::AllInRange)
}


pub fn get_range_in_location(chain: &dyn ChainReader, dag: &BlockDAG, storage: Arc<dyn Store>, start_id: HashValue) -> anyhow::Result<RangeInPruningPoint> {
        let current_pruning_point = chain.current_header().pruning_point();
        if current_pruning_point == start_id
            || 
                dag.check_ancestor_of_chain(start_id, current_pruning_point)?
        {
            let start_block_header = storage
                .get_block_header_by_hash(start_id)?
                .ok_or_else(|| format_err!("Cannot find block header by hash: {}", start_id))?;

            let mut result = vec![];

            for index in 0..=17 {
                let block_number = start_block_header.number().saturating_add(2u64.pow(index));
                if block_number >= chain.current_header().number() {
                    break;
                }
                let block_id =
                chain.get_header_by_number(index as u64)?
                        .ok_or_else(|| 
                            format_err!("cannot find the block by number: {:?}", block_number)
                        )?.id();

                result.push(block_id);
            }
            return Ok(RangeInPruningPoint::InSelectedChain(start_id, result));
        }

        Ok(RangeInPruningPoint::NotInSelectedChain)
}