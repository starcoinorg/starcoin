// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use scmd::CommandAction;

#[derive(Debug, Parser)]
#[clap(name = "txn-factory", alias = "txn_factory")]
pub enum TxnFactoryCmdOpt {
    /// dev txn-factory generate 100
    Generate {
        #[clap(short, long, default_value = "1")]
        count: usize,
    },
}

pub struct TxnFactoryCmd;

impl CommandAction for TxnFactoryCmd {
    type State = crate::cli_state::CliState;
    type GlobalOpt = crate::StarcoinOpt;
    type Opt = TxnFactoryCmdOpt;
    type ReturnItem = String;

    fn run(
        &self,
        ctx: &scmd::ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> anyhow::Result<Self::ReturnItem> {
        let state = ctx.state();
        let client = state.vm2()?;
        let accounts_file = state.txn_factory_accounts_file();
        let txn_factory = client.txn_factory();
        Ok(txn_factory.to_string())
    }
}
