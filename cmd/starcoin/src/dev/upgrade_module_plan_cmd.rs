// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_transaction_builder::build_module_upgrade_plan;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::TransactionPayload;
use structopt::StructOpt;

/// Execute the module upgrade proposal and submit module upgrade plan.
#[derive(Debug, StructOpt)]
#[structopt(name = "module-plan", alias = "module_plan")]
pub struct UpgradeModulePlanOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(short = "a", name = "proposer-address", long = "proposer_address")]
    /// the account address for proposer.
    proposer_address: Option<AccountAddress>,

    #[structopt(
        short = "i",
        name = "proposal-id",
        long = "proposal-id",
        help = "proposal id."
    )]
    proposal_id: u64,

    #[structopt(
        name = "dao-token",
        long = "dao-token",
        default_value = "0x1::STC::STC"
    )]
    /// The token for dao governance, default is 0x1::STC::STC
    dao_token: TokenCode,
}

pub struct UpgradeModulePlanCommand;

impl CommandAction for UpgradeModulePlanCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeModulePlanOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let proposer_address = if let Some(address) = ctx.opt().proposer_address {
            address
        } else if let Some(sender) = ctx.opt().transaction_opts.sender {
            sender
        } else {
            ctx.state().default_account()?.address
        };
        let module_upgrade_plan =
            build_module_upgrade_plan(proposer_address, opt.proposal_id, opt.dao_token.clone());
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(module_upgrade_plan),
        )
    }
}
