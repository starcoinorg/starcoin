// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::consensus::Consensus;
use crate::time::{MockTimeService, TimeService};
use anyhow::Result;
use logger::prelude::*;
use rand::Rng;
use starcoin_crypto::HashValue;
use starcoin_state_api::AccountStateReader;
use starcoin_statedb::ChainStateReader;
use starcoin_traits::ChainReader;
use starcoin_types::block::BlockHeader;
use starcoin_types::U256;
use starcoin_vm_types::on_chain_config::EpochInfo;

#[derive(Default)]
pub struct DummyConsensus {
    time_service: MockTimeService,
}

impl DummyConsensus {
    pub fn new() -> Self {
        Self {
            // 0 is genesis time, so default init with 1.
            time_service: MockTimeService::new_with_value(1),
        }
    }
}

impl Consensus for DummyConsensus {
    fn init(&self, reader: &dyn ChainStateReader) -> Result<()> {
        let account_state_reader = AccountStateReader::new(reader);
        let init_seconds = account_state_reader.get_timestamp()?.seconds;
        if init_seconds > 0 {
            info!("Adjust time service with on chain time: {}", init_seconds);
            //add 1 seconds to on chain seconds, for avoid time conflict
            self.time_service.set(init_seconds + 1);
        }
        Ok(())
    }

    fn calculate_next_difficulty(
        &self,
        _chain: &dyn ChainReader,
        epoch: &EpochInfo,
    ) -> Result<U256> {
        Ok(epoch.block_time_target().into())
    }

    fn solve_consensus_nonce(&self, _mining_hash: HashValue, difficulty: U256) -> u64 {
        let mut rng = rand::thread_rng();
        let time: u64 = rng.gen_range(1, difficulty.as_u64() * 2);
        debug!(
            "DummyConsensus rand sleep time in millis second : {}, difficulty : {}",
            time,
            difficulty.as_u64()
        );
        self.time_service.sleep(time);
        time
    }

    fn verify(
        &self,
        _reader: &dyn ChainReader,
        _epoch: &EpochInfo,
        _header: &BlockHeader,
    ) -> Result<()> {
        Ok(())
    }

    fn calculate_pow_hash(&self, _mining_hash: HashValue, _nonce: u64) -> Result<HashValue> {
        unreachable!()
    }

    fn time(&self) -> &dyn TimeService {
        &self.time_service
    }
}
