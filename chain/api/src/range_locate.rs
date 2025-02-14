// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schemadb::ReachabilityStoreReader};
use starcoin_storage::Store;

use crate::ChainReader;

#[derive(Debug)]
pub enum FindCommonHeader {
    AllInRange,                    // all in range
    InRange(HashValue, HashValue), // start in range but end not
    Found(HashValue),              // found the common header
    NotInRange,                    // all are not in reachability
}

#[derive(Debug)]
pub enum FindCommonHeaderError {
    InvalidRange(String),  // the range is invalid, there must be something wrong
    CheckAncestor(String), // failed to check the hash whether in reachability
    RangeLen,              // the length of the range is not greater than 2
}

#[derive(Debug, PartialEq, Eq)]
pub enum RangeInLocation {
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

pub fn get_range_in_location(
    chain: &dyn ChainReader,
    storage: Arc<dyn Store>,
    start_id: HashValue,
    end_id: Option<HashValue>,
) -> anyhow::Result<RangeInLocation> {
    let start_block_header = match storage.get_block_header_by_hash(start_id)? {
        Some(header) => header,
        None => return anyhow::Result::Ok(RangeInLocation::NotInSelectedChain),
    };

    match chain.get_block_info_by_number(start_block_header.number())? {
        Some(block_info) => {
            if *block_info.block_id() != start_id {
                return Ok(RangeInLocation::NotInSelectedChain);
            }
        }
        None => return Ok(RangeInLocation::NotInSelectedChain),
    }
    let mut result = vec![];

    let end_number = if let Some(end_id) = end_id {
        if let Some(end_block_header) = storage.get_block_header_by_hash(end_id)? {
            end_block_header.number()
        } else {
            chain.current_header().number()
        }
    } else {
        chain.current_header().number()
    };

    for index in 0..=17 {
        let block_number = start_block_header.number().saturating_add(2u64.pow(index));
        if block_number > chain.current_header().number() {
            break;
        }
        if block_number > end_number {
            break;
        }

        let block_id = if let Some(header) = chain.get_header_by_number(block_number)? {
            header.id()
        } else {
            break;
        };

        result.push(block_id);
    }
    Ok(RangeInLocation::InSelectedChain(start_id, result))
}
