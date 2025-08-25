// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_state::CliState, view::TransactionOptions, view_vm2::ExecuteResultView, StarcoinOpt,
};
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};

use starcoin_vm2_transaction_builder::build_module_upgrade_queue;
use starcoin_vm2_vm_types::{
    account_address::AccountAddress, token::token_code::TokenCode, transaction::TransactionPayload,
};

/// Queue the upgrade module proposal
#[derive(Debug, Parser)]
#[clap(name = "module-queue", alias = "module_queue")]
pub struct UpgradeModuleQueueOpt {
    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(short = 'a', name = "proposer-address", long = "proposer_address")]
    /// the account address for proposer.
    proposer_address: Option<AccountAddress>,

    #[clap(
        short = 'i',
        name = "proposal-id",
        long = "proposal-id",
        help = "proposal id."
    )]
    proposal_id: u64,

    #[clap(
        name = "dao-token",
        long = "dao-token",
        default_value = "0x1::starcoin_coin::STC"
    )]
    /// The token for dao governance, default is 0x1::STC::STC
    dao_token: TokenCode,
}

pub struct UpgradeModuleQueueCommand;

impl CommandAction for UpgradeModuleQueueCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeModuleQueueOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let proposer_address = if let Some(address) = ctx.opt().proposer_address {
            address
        } else if let Some(sender) = ctx.opt().transaction_opts.sender {
            AccountAddress::from_bytes(sender.into_bytes())?
        } else {
            AccountAddress::from_bytes(ctx.state().vm2()?.default_account()?.address)?
        };

        ctx.state().vm2()?.build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::EntryFunction(build_module_upgrade_queue(
                proposer_address,
                opt.proposal_id,
                opt.dao_token.clone(),
                true,
            )),
        )
    }
}
