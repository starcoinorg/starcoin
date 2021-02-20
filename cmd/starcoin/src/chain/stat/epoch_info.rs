// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_vm_types::on_chain_config::ConsensusConfig;
use starcoin_vm_types::on_chain_resource::EpochInfo;
use structopt::StructOpt;

/// Get stat of epoch_info from chain.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch")]
pub struct StatEpochOpt {}

pub struct StatEpochCommand;

impl CommandAction for StatEpochCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = StatEpochOpt;
    type ReturnItem = Vec<EpochInfo>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let chain_info = client.chain_info()?;
        let end_number = chain_info.head.number.0;
        let chain_state_reader = RemoteStateReader::new(client)?;
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let consensus_config = account_state_reader
            .get_on_chain_config::<ConsensusConfig>()?
            .ok_or_else(|| format_err!("ConsensusConfig not exist on chain."))?;
        let epoch_block_count = consensus_config.epoch_block_count;
        let epoch_count = end_number / epoch_block_count + 1;
        let chain_info = client.chain_info()?;
        let end_number = chain_info.head.number.0;
        // get epoch_info
        let mut epoch_number = 1;
        let vec_epoch = vec![];
        while epoch_number < epoch_count {
            let mut block_number = epoch_number * 240 - 1;
            if block_number >= end_number {
                block_number = end_number;
            }
            let epoch = client.get_epoch_info_by_number(block_number)?;
            println!(
                "epoch: {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                epoch.number(),
                epoch.block_time_target(),
                epoch.total_reward(),
                epoch.reward_per_block(),
                epoch.reward_per_uncle_percent(),
                epoch.epoch_data().uncles(),
                epoch.epoch_data().total_gas(),
                epoch.start_time(),
            );
            // vec_epoch.push(epoch);
            epoch_number += 1;
        }
        Ok(vec_epoch)
    }
}
