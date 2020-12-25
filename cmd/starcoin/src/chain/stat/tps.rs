// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::stress_test::TPS;
use starcoin_vm_types::on_chain_config::ConsensusConfig;
use structopt::StructOpt;

/// Get stat of tps for an epoch.
#[derive(Debug, StructOpt)]
#[structopt(name = "tps")]
pub struct TPSOpt {}

pub struct StatTPSCommand;

impl CommandAction for StatTPSCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = TPSOpt;
    type ReturnItem = Vec<TPS>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let chain_info = client.chain_info().unwrap();
        let end_number = chain_info.head.number;
        let chain_state_reader = RemoteStateReader::new(client)?;
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let consensus_config = account_state_reader.get_on_chain_config::<ConsensusConfig>()?;
        let epoch_block_count = consensus_config.unwrap().epoch_block_count;
        let epoch_count = end_number / epoch_block_count + 1;
        // get tps
        let mut epoch = 1;
        let mut vec_tps = vec![];
        while epoch < epoch_count {
            let mut block_number = epoch * epoch_block_count - 1;
            if block_number >= end_number {
                block_number = end_number;
            }
            let tps = client.tps(Some(block_number)).unwrap();
            println!("tps: {:?}", tps);
            vec_tps.push(tps);
            epoch += 1;
        }
        Ok(vec_tps)
    }
}
