// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_state::CliState, view::TransactionOptions, view_vm2::ExecuteResultView, StarcoinOpt,
};
use anyhow::{ensure, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_vm2_transaction_builder::empty_txn_payload;

/// Trigger a new block in dev.
#[derive(Debug, Parser)]
#[clap(name = "gen-block")]
pub struct GenBlockOpt {}

pub struct GenBlockCommand;

impl CommandAction for GenBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenBlockOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let cli_state = ctx.state();
        let net = cli_state.vm2()?.net();
        ensure!(net.is_dev(), "Only dev network support this command");
        let txn_opts = TransactionOptions {
            blocking: true,
            dry_run: false,
            ..Default::default()
        };
        ctx.state()
            .vm2()?
            .build_and_execute_transaction(txn_opts, empty_txn_payload())
    }
}
