// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::init_or_load_data_dir;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use serde::{Deserialize, Serialize};
use starcoin_chain::ChainReader;
use starcoin_chain_mock::MockChain;
use starcoin_config::StarcoinOpt;
use starcoin_logger::prelude::*;
use starcoin_storage::BlockStore;
use starcoin_types::block::BlockHeader;
use starcoin_types::startup_info::StartupInfo;
use std::time::SystemTime;
use structopt::StructOpt;

///Generate starcoin config and data, just for test.
#[derive(Debug, StructOpt)]
#[structopt(name = "data")]
pub struct GenDataOpt {
    ///How many block to generate.
    #[structopt(long, short = "s", default_value = "100")]
    count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenBlockResult {
    pub count: u64,
    pub use_seconds: u64,
    pub latest_header: BlockHeader,
}

pub struct GenDataCommand;

impl CommandAction for GenDataCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenDataOpt;
    type ReturnItem = GenBlockResult;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let global_opt = ctx.global_opt();
        let (config, storage, chain_info, account) = init_or_load_data_dir(global_opt, None)?;
        if chain_info.head().id() != chain_info.genesis_hash() {
            warn!("start block is not genesis.")
        }
        let begin = SystemTime::now();
        let mut mock_chain = MockChain::new_with_storage(
            config.net().clone(),
            storage.clone(),
            chain_info.head().id(),
            account,
        )?;
        let mut latest_header = mock_chain.head().current_header();
        for i in 0..opt.count {
            latest_header = mock_chain.produce_and_apply()?;
            let startup_info = StartupInfo::new(latest_header.id());
            storage.save_startup_info(startup_info)?;
            if i % 10 == 0 {
                println!(
                    "latest_block: {:?}, {:?}",
                    latest_header.number(),
                    latest_header.id()
                );
            }
        }

        let duration = SystemTime::now().duration_since(begin)?;
        Ok(GenBlockResult {
            count: opt.count,
            use_seconds: duration.as_secs(),
            latest_header,
        })
    }
}
