// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{cli_state::CliState, StarcoinOpt};
use anyhow::{bail, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};

/// Limit the minimal txns in block.
/// This command only available in dev network.
#[derive(Debug, Parser)]
#[clap(name = "minimal-txns-each-block", alias = "minimal_txns_each_block")]
pub struct MinimalTxnsEachBlockOpt {
    /// The minimum pending txn threshold to set. Ignored when --read is used.
    #[clap(short = 'w', long = "write")]
    min: Option<usize>,

    /// Read current threshold instead of setting.
    #[clap(short = 'r', long = "read")]
    read: bool,
}

pub struct MinimalTxnsEachBlockCommand;

impl CommandAction for MinimalTxnsEachBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = MinimalTxnsEachBlockOpt;
    type ReturnItem = Option<()>;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let net = ctx.state().net();

        if !net.is_test_or_dev() {
            bail!(
                "The network {} does not support the minimal-txns-each-block command.",
                net
            );
        }

        let client = ctx.state().client();

        if opt.read {
            let current = client.get_min_pending_txn_threshold()?;
            eprintln!("MIN_PENDING_TXN_THRESHOLD = {}", current);
            return Ok(Some(()));
        }

        if let Some(min) = opt.min {
            client.set_min_pending_txn_threshold(min)?;
            eprintln!("Requested server to set MIN_PENDING_TXN_THRESHOLD to {}", min.max(1));
            let current = client.get_min_pending_txn_threshold()?;
            eprintln!("Current MIN_PENDING_TXN_THRESHOLD = {}", current);
            return Ok(Some(()));
        }

        Ok(Some(()))
    }
}
