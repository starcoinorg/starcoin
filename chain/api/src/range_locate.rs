// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use starcoin_crypto::HashValue;
use starcoin_dag::{blockdag::BlockDAG, consensusdb::schemadb::ReachabilityStoreReader};
use starcoin_logger::prelude::*;
use starcoin_storage::Store;

use crate::ChainReader;

const MAX_RANGE_POWER: u32 = 17; // 2^17 = 131,072 blocks

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

        info!("jacktest: go to find the range locate for range: ({start}, {end})",);
        let reader = dag.storage.reachability_store.read();
        let start_has = reader.has(start).map_err(|err| {
            FindCommonHeaderError::CheckAncestor(format!(
                "failed to check the reachability has for start for error: {:?}",
                err
            ))
        })?;
        let end_has = reader.has(end).map_err(|err| {
            FindCommonHeaderError::CheckAncestor(format!(
                "failed to check the reachability has for end for error: {:?}",
                err
            ))
        })?;
        drop(reader);
        match (start_has, end_has) {
            (true, true) => {
                info!("jacktest: found the common header in range: ({start}, {end})");
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
            if end_block_header.number() < start_block_header.number() {
                return Ok(RangeInLocation::NotInSelectedChain);
            }
            end_block_header.number()
        } else {
            chain.current_header().number()
        }
    } else {
        chain.current_header().number()
    };

    for index in 0..=MAX_RANGE_POWER {
        let step = 2u64
            .checked_pow(index)
            .ok_or_else(|| anyhow::format_err!("Block number step calculation overflow"))?;
        let block_number = start_block_header
            .number()
            .checked_add(step)
            .ok_or_else(|| anyhow::format_err!("Block number calculation overflow"))?;
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
