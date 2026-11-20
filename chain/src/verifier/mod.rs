// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use sp_utils::stop_watch::{watch, CHAIN_WATCH_NAME};
use starcoin_chain_api::{
    verify_block, ChainReader, ConnectBlockError, VerifiedBlock, VerifyBlockField,
};
use starcoin_consensus::{Consensus, ConsensusVerifyError};
use starcoin_logger::prelude::debug;
use starcoin_open_block::AddressFilter;
use starcoin_types::block::{Block, BlockHeader, ALLOWED_FUTURE_BLOCKTIME};
use starcoin_types::block_permit::validate_block_permit;
use starcoin_types::U256;
use starcoin_vm_types::genesis_config::ConsensusStrategy;
use starcoin_vm_types::on_chain_resource::Epoch;
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
            "basic" => Ok(Verifier::Basic),
            "consensus" => Ok(Verifier::Consensus),
            "full" => Ok(Verifier::Full),
            "none" => Ok(Verifier::None),
            _ => Err(format!("invalid verifier type: {}", s)),
        }
    }
}

pub struct StaticVerifier;
impl StaticVerifier {
    /// Cheap PoW check that needs no chain view, for untrusted headers before expensive work
    /// such as building a fork. Every valid block passes:
    /// - its difficulty is never zero (a target of at most `U256::MAX` is a difficulty of 1);
    /// - its nonce meets that difficulty;
    /// - its difficulty is at least `parent / (4 * window)`. A new target is at most twice its
    ///   window's average and the parent's target is at least half of its own window's average,
    ///   so no target in the child's window exceeds `2 * window` times the parent's.
    ///   The dummy strategy does not adjust difficulty this way and is not bounded.
    pub fn verify_claimed_pow(
        epoch: &Epoch,
        header: &BlockHeader,
        parent: Option<&BlockHeader>,
    ) -> Result<()> {
        let difficulty = header.difficulty();
        let consensus_err =
            |e: Error| ConnectBlockError::VerifyBlockFailed(VerifyBlockField::Consensus, e).into();
        if difficulty.is_zero() {
            return Err(consensus_err(format_err!("block difficulty is zero")));
        }
        let strategy = epoch.strategy();
        if let Some(parent) = parent.filter(|_| strategy != ConsensusStrategy::Dummy) {
            let floor = parent.difficulty()
                / U256::from(epoch.block_difficulty_window().saturating_mul(4).max(1));
            if difficulty < floor {
                return Err(consensus_err(format_err!(
                    "block difficulty {} is below {}, the least a child of difficulty {} can have",
                    difficulty,
                    floor,
                    parent.difficulty()
                )));
            }
        }
        strategy
            .verify_header_difficulty(difficulty, header)
            .map_err(|e| match e.downcast::<ConsensusVerifyError>() {
                Ok(e) => consensus_err(e.into()),
                Err(e) => e,
            })
    }

    pub fn verify_body_hash(block: &Block) -> Result<()> {
        //verify body
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
        Self::verify_blacklisted_txns(&new_block)?;
        Self::verify_header(current_chain, new_block_header)?;
        watch(CHAIN_WATCH_NAME, "n12");
        StaticVerifier::verify_body_hash(&new_block)?;
        watch(CHAIN_WATCH_NAME, "n13");
        let chain_info = current_chain.info();
        validate_block_permit(
            current_chain.block_permit_policy(),
            chain_info.chain_id(),
            &current_chain.current_header(),
            &new_block,
        )
        .map_err(|err| ConnectBlockError::VerifyBlockFailed(VerifyBlockField::Body, err))?;
        //verify uncles
        Self::verify_uncles(
            current_chain,
            new_block.uncles().unwrap_or_default(),
            new_block_header,
        )?;
        watch(CHAIN_WATCH_NAME, "n14");
        Ok(VerifiedBlock(new_block))
    }

    fn verify_blacklisted_txns(new_block: &Block) -> Result<()> {
        let block_number = new_block.header().number();
        for txn in new_block.transactions() {
            verify_block!(
                VerifyBlockField::Body,
                !AddressFilter::is_blacklisted(txn, block_number),
                "Invalid block: the sender of transaction in block must be not blacklisted"
            );
        }
        Ok(())
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
            uncles.len() as u64 <= epoch.max_uncles_per_block(),
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
                uncle.number() < header.number() ,
               "uncle block number bigger than or equal to current block ,uncle block number is {} , current block number is {}", uncle.number(), header.number()
            );

            verify_block!(
                VerifyBlockField::Uncle,
                Self::can_be_uncle(current_chain, uncle)?,
                "invalid block: block {} can not be uncle.",
                uncle_id
            );
            debug!(
                "verify_uncle header number {} hash {:?} uncle number {} hash {:?}",
                header.number(),
                header.id(),
                uncle.number(),
                uncle.id()
            );
            // uncle's parent exists in current chain is checked in `can_be_uncle`, so this fork should success.
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
            && uncles.get(&block_header.id()).is_none()
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
        let expect_number = current.number().saturating_add(1);

        verify_block!(
            VerifyBlockField::Header,
            expect_number == new_block_header.number(),
            "Invalid block: Unexpect block number, expect:{}, got: {}.",
            expect_number,
            new_block_header.number()
        );

        verify_block!(
            VerifyBlockField::Header,
            current_id == new_block_parent,
            "Invalid block: Parent id mismatch, expect:{}, got: {}, number:{}.",
            current_id,
            new_block_parent,
            new_block_header.number()
        );

        verify_block!(
            VerifyBlockField::Header,
            new_block_header.timestamp() > current.timestamp(),
            "Invalid block: block timestamp too old, parent time:{}, block time: {}, number:{}.",
            current.timestamp(),
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

        let current_block_info = current_chain
            .get_block_info(Some(current_id))?
            .ok_or_else(|| format_err!("Can not find block info by head id: {}", current_id))?;
        verify_block!(
            VerifyBlockField::Header,
            current_block_info
                .get_block_accumulator_info()
                .get_accumulator_root()
                == &new_block_header.block_accumulator_root(),
            "Block accumulator root miss match {:?} : {:?}",
            current_block_info
                .get_block_accumulator_info()
                .get_accumulator_root(),
            new_block_header.block_accumulator_root(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use starcoin_time_service::RealTimeService;
    use starcoin_types::block::BlockHeaderBuilder;
    use starcoin_vm_types::account_address::AccountAddress;
    use starcoin_vm_types::event::{EventHandle, EventKey};

    fn keccak_epoch() -> Epoch {
        Epoch::new(
            0,
            0,
            0,
            1000,
            10_000,
            0,
            0,
            24,
            2,
            50_000_000,
            ConsensusStrategy::Keccak.value(),
            EventHandle::new(EventKey::new_from_address(&AccountAddress::ONE, 0), 0),
        )
    }

    #[test]
    fn claimed_pow_accepts_solved_headers_only() {
        let epoch = keccak_epoch();
        let difficulty = U256::from(1000u64);
        let header = BlockHeaderBuilder::random()
            .with_difficulty(difficulty)
            .build();
        let nonce = epoch.strategy().solve_consensus_nonce(
            &header.as_pow_header_blob(),
            difficulty,
            &RealTimeService::new(),
        );
        let solved = header.as_builder().with_nonce(nonce).build();
        StaticVerifier::verify_claimed_pow(&epoch, &solved, None).unwrap();

        let unsolved = (0u32..)
            .map(|n| header.as_builder().with_nonce(n).build())
            .find(|h| {
                epoch
                    .strategy()
                    .verify_header_difficulty(difficulty, h)
                    .is_err()
            })
            .unwrap();
        assert_consensus_failure(StaticVerifier::verify_claimed_pow(&epoch, &unsolved, None));
    }

    fn assert_consensus_failure(result: Result<()>) {
        let err = result.unwrap_err();
        assert!(
            matches!(
                err.downcast_ref::<ConnectBlockError>(),
                Some(ConnectBlockError::VerifyBlockFailed(
                    VerifyBlockField::Consensus,
                    _
                ))
            ),
            "{:?}",
            err
        );
    }

    #[test]
    fn claimed_pow_rejects_zero_difficulty() {
        let header = BlockHeaderBuilder::random()
            .with_difficulty(U256::zero())
            .build();
        assert_consensus_failure(StaticVerifier::verify_claimed_pow(
            &keccak_epoch(),
            &header,
            None,
        ));
    }

    #[test]
    fn claimed_pow_bounds_the_drop_from_the_parent() {
        let epoch = keccak_epoch();
        let floor = U256::from(1000u64);
        let parent = BlockHeaderBuilder::random()
            .with_difficulty(floor * U256::from(4 * epoch.block_difficulty_window()))
            .build();
        let solve = |difficulty: U256| {
            let header = BlockHeaderBuilder::random()
                .with_difficulty(difficulty)
                .build();
            let nonce = epoch.strategy().solve_consensus_nonce(
                &header.as_pow_header_blob(),
                difficulty,
                &RealTimeService::new(),
            );
            header.as_builder().with_nonce(nonce).build()
        };
        StaticVerifier::verify_claimed_pow(&epoch, &solve(floor), Some(&parent)).unwrap();
        assert_consensus_failure(StaticVerifier::verify_claimed_pow(
            &epoch,
            &solve(floor - U256::one()),
            Some(&parent),
        ));
    }

    /// Replays real headers through the check: every one must pass. The input is one
    /// BCS-hex header per line in chain order, e.g. from `chain.get_block_by_number` with
    /// `{"raw": true}`. Run with
    /// `STARCOIN_MAIN_HEADERS=<file> cargo test -p starcoin-chain -- --ignored main_headers`.
    #[test]
    #[ignore]
    fn claimed_pow_accepts_main_headers() {
        let path = match std::env::var("STARCOIN_MAIN_HEADERS") {
            Ok(path) => path,
            Err(_) => return,
        };
        // Main's epoch: CryptoNight with a 24-block difficulty window.
        let epoch = Epoch::new(
            0,
            0,
            0,
            0,
            10_000,
            0,
            0,
            24,
            2,
            50_000_000,
            ConsensusStrategy::CryptoNight.value(),
            EventHandle::new(EventKey::new_from_address(&AccountAddress::ONE, 0), 0),
        );
        let text = std::fs::read_to_string(path).unwrap();
        let headers: Vec<BlockHeader> = text
            .lines()
            .map(|line| {
                let bytes = hex::decode(line.trim().trim_start_matches("0x")).unwrap();
                bcs_ext::from_bytes(&bytes).unwrap()
            })
            .collect();
        let mut with_parent = 0;
        for (i, header) in headers.iter().enumerate() {
            let parent = i
                .checked_sub(1)
                .map(|p| &headers[p])
                .filter(|p| p.id() == header.parent_hash());
            with_parent += usize::from(parent.is_some());
            StaticVerifier::verify_claimed_pow(&epoch, header, parent)
                .unwrap_or_else(|e| panic!("block {} rejected: {:?}", header.number(), e));
        }
        assert!(with_parent > 0);
        println!(
            "{} headers passed, {} with parent",
            headers.len(),
            with_parent
        );
    }
}
