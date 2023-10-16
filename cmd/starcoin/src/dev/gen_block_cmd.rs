// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::{anyhow, ensure, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_transaction_builder::build_empty_script;
use starcoin_types::transaction::TransactionPayload;

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
        let net = cli_state.net();
        ensure!(net.is_dev(), "Only dev network support this command");
        let empty = build_empty_script();
        let txn_opts = TransactionOptions {
            blocking: true,
            dry_run: false,
            ..Default::default()
        };
        ctx.state()
            .build_and_execute_transaction(txn_opts, TransactionPayload::ScriptFunction(empty))
    }
}
