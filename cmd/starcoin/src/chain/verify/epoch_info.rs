// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_vm_types::on_chain_config::ConsensusConfig;
use std::cmp::min;
use std::collections::HashMap;
use structopt::StructOpt;

/// Verify epoch_info.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch")]
pub struct EpochOpt {
    #[structopt(name = "number", long, short = "n", default_value = "0")]
    block_number: u64,
}

pub struct VerifyEpochCommand;

impl CommandAction for VerifyEpochCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = EpochOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();
        let block_number = opt.block_number;

        let epoch_info = client.get_epoch_info_by_number(block_number)?;
        let start = epoch_info.start_block_number();
        let block_number1 = if block_number >= 240 {
            block_number + 1
        } else {
            block_number
        };
        let end = min(epoch_info.end_block_number(), block_number1);
        let mut block_map = HashMap::new();
        //check uncles
        let uncles = epoch_info.uncles();
        let mut total_uncle = 0u64;
        for number in start..end {
            let block = client
                .chain_get_block_by_number(number)?
                .ok_or_else(|| format_err!("block: {} not found", number))?;
            block_map.insert(number, block.clone());
            total_uncle += block.uncles.len() as u64;
        }
        assert_eq!(uncles, total_uncle);
        info!("verify uncle ok!");
        //block reward
        let total_reward = epoch_info.total_reward();
        let mut block_total_reward = 0u128;
        let reward_per_block = epoch_info.reward_per_block();
        let reward_per_uncle_percent = epoch_info.reward_per_uncle_percent();

        for number in start..end {
            let block = block_map.get(&number).unwrap();
            let block_reward = reward_per_block
                + reward_per_block
                    * (reward_per_uncle_percent as u128)
                    * (block.uncles.len() as u128)
                    / 100_u128;
            block_total_reward += block_reward;
        }
        assert_eq!(block_total_reward, total_reward);
        info!("verify reward ok!");

        //time_target increase
        if block_number >= 240 {
            let last_number = block_number - 240;
            let last_epoch_info = client.get_epoch_info_by_number(last_number)?;
            let last_time_target = last_epoch_info.block_time_target();
            let blocks = last_epoch_info.end_block_number() - last_epoch_info.start_block_number();
            let uncles_rate = last_epoch_info.uncles() * 1000 / blocks;
            let total_time = client
                .chain_get_block_by_number(last_epoch_info.end_block_number())?
                .ok_or_else(|| {
                    format_err!("block: {} not found", last_epoch_info.end_block_number())
                })?
                .header
                .timestamp
                .0
                - last_epoch_info.start_time();
            let avg_block_time = total_time / blocks;
            let chain_state_reader = RemoteStateReader::new(client)?;
            let account_state_reader = AccountStateReader::new(&chain_state_reader);
            let consensus_config = account_state_reader
                .get_on_chain_config::<ConsensusConfig>()?
                .ok_or_else(|| format_err!("ConsensusConfig not exist on chain."))?;

            let mut time_target =
                (1000 + uncles_rate) * avg_block_time / (consensus_config.uncle_rate_target + 1000);
            if time_target > last_time_target * 2 {
                time_target = last_time_target * 2;
            };
            if time_target < last_time_target / 2 {
                time_target = last_time_target / 2;
            };

            let min_block_time_target = consensus_config.min_block_time_target;
            let max_block_time_target = consensus_config.max_block_time_target;
            if time_target < min_block_time_target {
                time_target = min_block_time_target;
            };
            if time_target > max_block_time_target {
                time_target = max_block_time_target;
            };
            assert_eq!(time_target, epoch_info.block_time_target());
            info!("verify time_target increase ok!");
        } else {
            warn!("current epoch not exist last epoch!");
        }

        Ok("verify ok!".parse()?)
    }
}
