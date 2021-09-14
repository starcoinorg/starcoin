// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::StateReaderExt;
use starcoin_transaction_builder::build_module_upgrade_queue;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::on_chain_config::Version;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::TransactionPayload;
use structopt::StructOpt;

/// Queue the upgrade module proposal
#[derive(Debug, StructOpt)]
#[structopt(name = "module-queue", alias = "module_queue")]
pub struct UpgradeModuleQueueOpt {
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
            sender
        } else {
            ctx.state().default_account()?.address
        };

        let chain_state_reader = ctx.state().client().state_reader(StateRootOption::Latest)?;
        let stdlib_version = chain_state_reader
            .get_on_chain_config::<Version>()?
            .map(|version| version.major)
            .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;

        let module_upgrade_queue = build_module_upgrade_queue(
            proposer_address,
            opt.proposal_id,
            opt.dao_token.clone(),
            StdlibVersion::new(stdlib_version),
        );
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(module_upgrade_queue),
        )
    }
}
