// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use consensus::{Consensus, ConsensusVerifyError};
use starcoin_chain_api::{
    verify_block, ChainReader, ConnectBlockError, VerifiedBlock, VerifyBlockField,
};
use starcoin_types::block::{Block, BlockHeader, ALLOWED_FUTURE_BLOCKTIME};
use std::collections::HashSet;

const MAX_UNCLE_COUNT_PER_BLOCK: usize = 2;
//TODO this trait should move to consensus?
pub trait BlockVerifier {
    fn verify_header<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader;
    fn verify_block<R>(current_chain: &R, new_block: Block) -> Result<VerifiedBlock>
    where
        R: ChainReader,
    {
        //verify header
        let new_block_header = new_block.header();
        Self::verify_header(current_chain, new_block_header)?;

        //verify body
        let body_hash = new_block.body.hash();
        verify_block!(
            VerifyBlockField::Body,
            body_hash == new_block_header.body_hash(),
            "verify block body hash mismatch, expect: {}, got: {}",
            new_block_header.body_hash(),
            body_hash,
        );
        //verify uncles
        Self::verify_uncles(
            current_chain,
            new_block.uncles().unwrap_or_default(),
            new_block_header,
        )?;
        Ok(VerifiedBlock(new_block))
    }

    fn verify_uncles<R>(
        current_chain: &R,
        uncles: &[BlockHeader],
        header: &BlockHeader,
    ) -> Result<()>
    where
        R: ChainReader,
    {
        let epoch = current_chain.epoch();

        let switch_epoch = header.number() == epoch.end_block_number();
        // epoch first block's uncles should empty.
        if switch_epoch {
            verify_block!(
                VerifyBlockField::Uncle,
                uncles.is_empty(),
                "Invalid block: first block of epoch's uncles must be empty."
            );
        }

        if uncles.is_empty() {
            return Ok(());
        }
        verify_block!(
            VerifyBlockField::Uncle,
            uncles.len() <= MAX_UNCLE_COUNT_PER_BLOCK,
            "too many uncles {} in block {}",
            uncles.len(),
            header.id()
        );

        let mut uncle_ids = HashSet::new();
        for uncle in uncles {
            let uncle_id = uncle.id();
            verify_block!(
                VerifyBlockField::Uncle,
                !uncle_ids.contains(&uncle.id()),
                "repeat uncle {:?} in current block {:?}",
                uncle_id,
                header.id()
            );

            verify_block!(
                VerifyBlockField::Uncle,
                uncle.number < header.number ,
               "uncle block number bigger than or equal to current block ,uncle block number is {} , current block number is {}", uncle.number, header.number
            );

            verify_block!(
                VerifyBlockField::Uncle,
                Self::can_be_uncle(current_chain, uncle)?,
                "invalid block: block {} can not be uncle.",
                uncle_id
            );
            // uncle's parent exist in current chain is check in can_be_uncle, so this fork should bean success.
            let uncle_branch = current_chain.fork(uncle.parent_hash())?;
            Self::verify_header(&uncle_branch, uncle)?;
            uncle_ids.insert(uncle_id);
        }
        Ok(())
    }

    fn can_be_uncle<R>(current_chain: &R, block_header: &BlockHeader) -> Result<bool>
    where
        R: ChainReader,
    {
        let epoch = current_chain.epoch();
        let uncles = current_chain.epoch_uncles();
        Ok(epoch.start_block_number() <= block_header.number()
            && epoch.end_block_number() > block_header.number()
            && current_chain.exist_block(block_header.parent_hash())?
            && !current_chain.exist_block(block_header.id())?
            && !uncles.contains(&block_header.id())
            && block_header.number() <= current_chain.current_header().number())
    }
}

pub struct BasicVerifier;

impl BlockVerifier for BasicVerifier {
    fn verify_header<R>(current_chain: &R, new_block_header: &BlockHeader) -> Result<()>
    where
        R: ChainReader,
    {
        let new_block_parent = new_block_header.parent_hash();
        let chain_status = current_chain.status();
        let current = chain_status.head();
        let current_id = current.id();
        let expect_number = current.number() + 1;

        verify_block!(
            VerifyBlockField::Header,
            expect_number == new_block_header.number,
            "Invalid block: Unexpect block number, expect:{}, got: {}.",
            expect_number,
            new_block_header.number
        );

        verify_block!(
            VerifyBlockField::Header,
            current_id == new_block_parent,
            "Invalid block: Parent id mismatch, expect:{}, got: {}, number:{}.",
            current_id,
            new_block_parent,
            new_block_header.number
        );

        verify_block!(
            VerifyBlockField::Header,
            current_id == new_block_parent,
            "Invalid block: Parent id mismatch, expect:{}, got: {}, number:{}.",
            current_id,
            new_block_parent,
            new_block_header.number
        );

        verify_block!(
            VerifyBlockField::Header,
            new_block_header.timestamp() > current.timestamp(),
            "Invalid block: block timestamp too old, parent time:{}, block time: {}, number:{}.",
            current.timestamp(),
            new_block_header.timestamp(),
            new_block_header.number
        );

        let now = current_chain.time_service().now_millis();
        verify_block!(
            VerifyBlockField::Header,
            new_block_header.timestamp() <= ALLOWED_FUTURE_BLOCKTIME + now,
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

        let current_block_info = current_chain
            .get_block_info(Some(current_id))?
            .expect("head block's block info should exist.");
        verify_block!(
            VerifyBlockField::Header,
            current_block_info
                .get_block_accumulator_info()
                .get_accumulator_root()
                == &new_block_header.parent_block_accumulator_root(),
            "Block accumulator root miss match {:?} : {:?}",
            current_block_info
                .get_block_accumulator_info()
                .get_accumulator_root(),
            new_block_header.parent_block_accumulator_root(),
        );
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

    fn verify_block<R>(_current_chain: &R, new_block: Block) -> Result<VerifiedBlock>
    where
        R: ChainReader,
    {
        Ok(VerifiedBlock(new_block))
    }

    fn verify_uncles<R>(
        _current_chain: &R,
        _uncles: &[BlockHeader],
        _header: &BlockHeader,
    ) -> Result<()>
    where
        R: ChainReader,
    {
        Ok(())
    }
}
