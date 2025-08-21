// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Result};
use sp_utils::stop_watch::{watch, CHAIN_WATCH_NAME};
use starcoin_chain_api::{
    verify_block, ChainReader, ConnectBlockError, VerifiedBlock, VerifyBlockField,
};
use starcoin_config::upgrade_config::vm1_offline_height;
use starcoin_consensus::{Consensus, ConsensusVerifyError};
use starcoin_crypto::HashValue;
use starcoin_dag::types::ghostdata::GhostdagData;
use starcoin_logger::prelude::error;
use starcoin_types::{
    block::{Block, BlockHeader, ALLOWED_FUTURE_BLOCKTIME},
    consensus_header::ConsensusHeader,
};
use std::{collections::HashSet, str::FromStr};

#[derive(Debug, Clone)]
pub enum Verifier {
    Basic,
    Consensus,
    Full,
    None,
}

impl Verifier {
    pub fn variants() -> [&'static str; 4] {
        ["basic", "consensus", "full", "none"]
    }
}

impl FromStr for Verifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "basic" => Ok(Self::Basic),
            "consensus" => Ok(Self::Consensus),
            "full" => Ok(Self::Full),
            "none" => Ok(Self::None),
            _ => Err(format!("invalid verifier type: {}", s)),
        }
    }
}

pub struct StaticVerifier;

impl StaticVerifier {
    pub fn verify_body_hash(block: &Block) -> Result<()> {
        // verify body
        let body_hash = block.body.hash();
        verify_block!(
            VerifyBlockField::Body,
            body_hash == block.header().body_hash(),
            "verify block body hash mismatch, expect: {}, got: {}",
            block.header().body_hash(),
            body_hash,
        );
        Ok(())
    }

    pub fn verify_vm1_offline(block: &Block) -> Result<()> {
        let vm1_offline_height = vm1_offline_height(block.header().chain_id().id().into());
        let vm1_offline = block.header().number() >= vm1_offline_height;
        verify_block!(
            VerifyBlockField::Body,
            !vm1_offline || block.transactions().is_empty(),
            "vm1 offline at {} but block {} has {} vm1 transactions",
            vm1_offline_height,
            block.header().number(),
            block.transactions().len()
        );
        Ok(())
    }
}

//TODO this trait should move to consensus?
pub trait BlockVerifier {
    fn verify_header<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader;

    fn verify_block<R>(current_chain: &R, new_block: Block) -> Result<VerifiedBlock>
    where
        R: ChainReader,
    {
        watch(CHAIN_WATCH_NAME, "n11");
        //verify header
        let new_block_header = new_block.header();
        Self::verify_header(current_chain, new_block_header)?;
        watch(CHAIN_WATCH_NAME, "n12");
        StaticVerifier::verify_body_hash(&new_block)?;
        StaticVerifier::verify_vm1_offline(&new_block)?;
        watch(CHAIN_WATCH_NAME, "n13");
        //verify uncles
        let ghostdata = Self::verify_uncles(
            current_chain,
            new_block.uncles().unwrap_or_default(),
            new_block_header,
        )?;
        watch(CHAIN_WATCH_NAME, "n14");
        Ok(VerifiedBlock {
            block: new_block,
            ghostdata,
        })
    }

    fn verify_blue_blocks<R>(
        current_chain: &R,
        uncles: &[BlockHeader],
        header: &BlockHeader,
    ) -> Result<GhostdagData>
    where
        R: ChainReader,
    {
        let ghostdata = current_chain.calc_ghostdata_and_check_bounded_merge_depth(header)?;
        match current_chain.validate_pruning_point(&ghostdata, header.pruning_point()) {
            Ok(()) => (),
            Err(e) => {
                error!("validate the pruning point failed, error: {:?}", e);
                return Err(e);
            }
        }

        Self::can_be_uncle(header, uncles, &ghostdata)?;

        Ok(ghostdata)
    }

    fn verify_uncles<R>(
        current_chain: &R,
        uncles: &[BlockHeader],
        header: &BlockHeader,
    ) -> Result<GhostdagData>
    where
        R: ChainReader,
    {
        Self::verify_blue_blocks(current_chain, uncles, header)
    }

    fn can_be_uncle(
        header: &BlockHeader,
        uncles: &[BlockHeader],
        ghostdata: &GhostdagData,
    ) -> Result<()> {
        let uncles_set = uncles
            .iter()
            .map(|header| header.id())
            .collect::<HashSet<_>>();
        let blue_set = ghostdata
            .mergeset_blues
            .iter()
            .skip(1)
            .cloned()
            .collect::<HashSet<_>>();
        verify_block!(
            VerifyBlockField::Uncle,
            uncles_set == blue_set,
            "Uncles in header are not the same as ghostdata, uncles: {:?}, ghostdata blue set: {:?}.",
            uncles,
            blue_set,
        );

        let selected_parent = *ghostdata.mergeset_blues.first().ok_or_else(|| {
            format_err!("no selected parent in blue set in ghostdata.mergeset_blues")
        })?;
        verify_block!(
            VerifyBlockField::Uncle,
            header.parent_hash() == selected_parent,
            "The selected parent is not the same as the parent in header, the first one in ghostdata.mergeset_blues: {:?}, selected parent in header: {:?}.",
            selected_parent,
            header.parent_hash(),
        );

        Ok(())
    }
}

pub struct BasicVerifier;

impl BlockVerifier for BasicVerifier {
    fn verify_header<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader,
    {
        let new_block_parent = new_block_header.parent_hash();

        // In DAG mode, parent doesn't need to be current head
        // Just verify parent exists and number is correct
        let parent_header = current_chain
            .get_header(new_block_parent)?
            .ok_or_else(|| format_err!("Parent block {:?} not found in chain", new_block_parent))?;

        // Check block number = parent.number + 1
        let expect_number = parent_header.number().saturating_add(1);
        verify_block!(
            VerifyBlockField::Header,
            expect_number == new_block_header.number(),
            "Invalid block: Unexpect block number, expect:{} (parent.number + 1), got: {}.",
            expect_number,
            new_block_header.number()
        );

        // No longer check parent == current.id in DAG mode
        // The parent just needs to exist in the chain/DAG

        // Check timestamp > parent's timestamp (not current's)
        verify_block!(
            VerifyBlockField::Header,
            new_block_header.timestamp() > parent_header.timestamp(),
            "Invalid block: block timestamp too old, parent time:{}, block time: {}, number:{}.",
            parent_header.timestamp(),
            new_block_header.timestamp(),
            new_block_header.number()
        );

        let now = current_chain.time_service().now_millis();
        verify_block!(
            VerifyBlockField::Header,
            new_block_header.timestamp() <= ALLOWED_FUTURE_BLOCKTIME.saturating_add(now),
            "Invalid block: block timestamp too new, now:{}, block time:{}",
            now,
            new_block_header.timestamp()
        );

        let epoch = current_chain.epoch();

        verify_block!(
            VerifyBlockField::Header,
            new_block_header.number() > epoch.start_block_number()
                && new_block_header.number() <= epoch.end_block_number(),
            "block number is {:?}, epoch start number is {:?}, epoch end number is {:?}",
            new_block_header.number(),
            epoch.start_block_number(),
            epoch.end_block_number(),
        );

        let block_gas_limit = epoch.block_gas_limit();

        verify_block!(
            VerifyBlockField::Header,
            new_block_header.gas_used() <= block_gas_limit,
            "invalid block: gas_used should not greater than block_gas_limit"
        );

        // Get parent's block info for accumulator verification
        let parent_block_info = current_chain
            .get_block_info(Some(new_block_parent))?
            .ok_or_else(|| {
                format_err!("Can not find block info by parent id: {}", new_block_parent)
            })?;
        verify_block!(
            VerifyBlockField::Header,
            parent_block_info
                .get_block_accumulator_info()
                .get_accumulator_root()
                == &new_block_header.block_accumulator_root(),
            "Block accumulator root miss match {:?} : {:?}",
            parent_block_info
                .get_block_accumulator_info()
                .get_accumulator_root(),
            new_block_header.block_accumulator_root(),
        );

        Self::verify_dag(current_chain, new_block_header)?;

        Ok(())
    }
}

impl BasicVerifier {
    fn verify_dag<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader,
    {
        let parents_hash = new_block_header.parents_hash();
        verify_block!(
            VerifyBlockField::Header,
            parents_hash.len() == parents_hash.iter().collect::<HashSet<_>>().len(),
            "The dag block contains repeated hash values, block header: {:?}",
            new_block_header,
        );

        verify_block!(
            VerifyBlockField::Header,
            parents_hash.contains(&new_block_header.parent_hash()),
            "header: {:?}, tips {:?} do not contain the selected parent {:?}",
            new_block_header,
            parents_hash,
            new_block_header.parent_hash()
        );

        parents_hash.iter().try_for_each(|parent_hash| {
            verify_block!(
                VerifyBlockField::Header,
                current_chain.has_dag_block(*parent_hash).map_err(|e| {
                    ConnectBlockError::VerifyBlockFailed(
                        VerifyBlockField::Header,
                        anyhow::anyhow!(
                            "failed to get the block: {:?} 's parent: {:?} from db, error: {:?}",
                            new_block_header.id(),
                            parent_hash,
                            e
                        ),
                    )
                })?,
                "Invalid block: parent {} might not exist.",
                parent_hash
            );
            Ok::<(), ConnectBlockError>(())
        })?;

        // verify the pruning point
        let parent_header = current_chain.current_header();
        if parent_header.pruning_point() != HashValue::zero() {
            // the chain had pruning point already checking the descendants of the pruning point is a must
            // check the parents are the descendants of the pruning point
            parents_hash.iter().try_for_each(|parent_hash| {
                verify_block!(
                    VerifyBlockField::Header,
                    current_chain.is_dag_ancestor_of(new_block_header.pruning_point(), *parent_hash).map_err(|e| {
                        ConnectBlockError::VerifyBlockFailed(
                            VerifyBlockField::Header,
                            anyhow::anyhow!(
                                "the block {:?} 's parent: {:?} is not the descendant of pruning point {:?}, error: {:?}",
                                new_block_header.id(),
                                parent_hash,
                                new_block_header.pruning_point(),
                                e
                            ),
                        )
                    })?,
                    "Invalid block: parent {:?} is not the descendant of pruning point: {:?}",
                    parent_hash, new_block_header.pruning_point()
                );
                Ok::<(), ConnectBlockError>(())
            })?;
        }

        Ok(())
    }
}

pub struct ConsensusVerifier;

impl BlockVerifier for ConsensusVerifier {
    fn verify_header<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader,
    {
        let epoch = current_chain.epoch();
        let consensus = epoch.strategy();
        if let Err(e) = consensus.verify(current_chain, new_block_header) {
            return match e.downcast::<ConsensusVerifyError>() {
                Ok(e) => Err(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Consensus,
                    e.into(),
                )
                .into()),
                Err(e) => Err(e),
            };
        }
        Ok(())
    }
}

pub struct FullVerifier;

impl BlockVerifier for FullVerifier {
    fn verify_header<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader,
    {
        BasicVerifier::verify_header(current_chain, new_block_header)?;
        ConsensusVerifier::verify_header(current_chain, new_block_header)
    }
}

pub struct NoneVerifier;

impl BlockVerifier for NoneVerifier {
    fn verify_header<R>(_current_chain: &R, _new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader,
    {
        Ok(())
    }

    fn verify_block<R>(current_chain: &R, new_block: Block) -> Result<VerifiedBlock>
    where
        R: ChainReader,
    {
        let ghostdata = Self::verify_uncles(
            current_chain,
            new_block.uncles().unwrap_or_default(),
            new_block.header(),
        )?;
        Ok(VerifiedBlock {
            block: new_block,
            ghostdata,
        })
    }

    fn verify_uncles<R>(
        current_chain: &R,
        _uncles: &[BlockHeader],
        header: &BlockHeader,
    ) -> Result<GhostdagData>
    where
        R: ChainReader,
    {
        let ghostdata = current_chain.dag().ghostdata(&header.parents())?;
        Ok(ghostdata)
    }
}
