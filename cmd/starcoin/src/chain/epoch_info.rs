// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::StateReaderExt;
use starcoin_vm_types::on_chain_resource::EpochInfo;
use structopt::StructOpt;

/// Show epoch info.
#[derive(Debug, StructOpt)]
#[structopt(name = "epoch-info", alias = "epoch_info")]
pub struct EpochInfoOpt {
    #[structopt(name = "state-root", long, short = "n", alias = "block-number")]
    /// The block number or block hash for get state, if absent, use latest block state_root.
    state_root: Option<StateRootOption>,
}

pub struct EpochInfoCommand;

impl CommandAction for EpochInfoCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = EpochInfoOpt;
    type ReturnItem = EpochInfo;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let client = ctx.state().client();
        let opt = ctx.opt();

        let chain_state_reader = client.state_reader(opt.state_root.unwrap_or_default())?;
        chain_state_reader.get_epoch_info()
    }
}
