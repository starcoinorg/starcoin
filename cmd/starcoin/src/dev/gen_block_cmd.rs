// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::sign_txn_with_account_by_rpc_client;
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_transaction_builder::build_empty_script;
use starcoin_types::transaction::TransactionPayload;
use structopt::StructOpt;

/// Trigger a new block in dev.
#[derive(Debug, StructOpt)]
#[structopt(name = "gen-block")]
pub struct GenBlockOpt {}

pub struct GenBlockCommand;

impl CommandAction for GenBlockCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = GenBlockOpt;
    type ReturnItem = ();

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let cli_state = ctx.state();
        let net = cli_state.net();
        assert!(net.is_dev());
        let sender = cli_state.default_account()?.address;
        let empty = build_empty_script();
        let signed_txn = sign_txn_with_account_by_rpc_client(
            cli_state,
            sender,
            1000000,
            1,
            3000,
            TransactionPayload::ScriptFunction(empty),
        )?;
        let txn_hash = signed_txn.id();
        cli_state.client().submit_transaction(signed_txn)?;

        println!("txn {:#x} submitted.", txn_hash);

        ctx.state().watch_txn(txn_hash)?;
        Ok(())
    }
}
