// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_api::types::ChainInfoView;
use starcoin_rpc_client::RpcClient;
use starcoin_types::block::BlockNumber;
use starcoin_types::stress_test::TPS;
use structopt::StructOpt;

/// Get tps for an epoch.
#[derive(Debug, StructOpt)]
#[structopt(name = "tps")]
#[allow(clippy::upper_case_acronyms)]
pub struct TPSOpt {
    #[structopt(
        name = "number",
        long,
        short = "n",
        help = "block number, if absent return tps for the latest epoch"
    )]
    number: Option<BlockNumber>,
}

#[allow(clippy::upper_case_acronyms)]
pub struct TPSCommand;

impl CommandAction for TPSCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TPSOpt;
    type ReturnItem = TPS;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let client = ctx.state().client();
        let chain_info = client.chain_info()?;
        let current_header = chain_info.clone().head;
        let current_number = current_header.number.0;
        let block_number = opt.number.unwrap_or(current_number);

        TPSCommand::epoch_tps(client, chain_info, current_number, block_number)
    }
}

impl TPSCommand {
    pub fn epoch_tps(
        client: &RpcClient,
        chain_info: ChainInfoView,
        current_number: u64,
        block_number: u64,
    ) -> Result<TPS> {
        let epoch_info = client.get_epoch_info_by_number(block_number)?;
        let start_time = epoch_info.start_time();
        let start_block_number = epoch_info.start_block_number();
        let end_block_number = epoch_info.end_block_number();

        let start_block_info = client
            .chain_get_block_info_by_number(start_block_number)?
            .ok_or_else(|| format_err!("block_info : {} not found", start_block_number))?;
        let start_leaves = start_block_info.txn_accumulator_info.num_leaves;

        let tps = if current_number < end_block_number {
            let end_time = chain_info.head.timestamp.0;
            let duration = (end_time - start_time) / 1000;

            let end_leaves = chain_info.block_info.txn_accumulator_info.num_leaves;
            let total_count = end_leaves - start_leaves;
            TPS::new(total_count, duration, total_count / duration)
        } else {
            let next_epoch = client.get_epoch_info_by_number(end_block_number)?;
            let end_time = next_epoch.start_time();
            let duration = (end_time - start_time) / 1000;

            // count txn
            let end_block_info = client
                .chain_get_block_info_by_number(end_block_number)?
                .ok_or_else(|| format_err!("block_info : {} not found", end_block_number))?;
            let end_leaves = end_block_info.txn_accumulator_info.num_leaves;
            let total_count = end_leaves - start_leaves;
            TPS::new(total_count, duration, total_count / duration)
        };

        Ok(tps)
    }
}
