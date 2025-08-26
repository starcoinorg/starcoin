// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_state::CliState,
    dev::{dev_helper_vm2, sign_txn_helper::get_dao_config},
    view::TransactionOptions,
    view_vm2::ExecuteResultView,
    StarcoinOpt,
};
use anyhow::{bail, format_err, Result};
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::StateRootOption;
use starcoin_vm2_transaction_builder::build_module_upgrade_proposal;
use starcoin_vm2_vm_types::{
    genesis_config::StdlibVersion, on_chain_config::Version, state_view::StateReaderExt,
    token::token_code::TokenCode, transaction::TransactionPayload,
};
use std::path::PathBuf;

/// Submit a module upgrade proposal
#[derive(Debug, Parser)]
#[clap(name = "module-proposal", alias = "module_proposal")]
pub struct UpgradeModuleProposalOpt {
    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(short = 'e', name = "enforced", long = "enforced")]
    /// enforced upgrade regardless of compatible or not
    enforced: bool,

    #[clap(
        short = 'm',
        name = "mv-or-package-file",
        long = "mv-or-package-file",
        parse(from_os_str)
    )]
    /// path for module or package file.
    mv_or_package_file: PathBuf,

    #[clap(short = 'v', name = "module-version", long = "module-version")]
    /// new version number for the modules
    version: u64,

    #[clap(
        name = "dao-token",
        long = "dao-token",
        default_value = "0x1::starcoin_coin::STC"
    )]
    /// The token for dao governance, default is 0x1::STC::STC
    dao_token: TokenCode,
}

pub struct UpgradeModuleProposalCommand;

impl CommandAction for UpgradeModuleProposalCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeModuleProposalOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let cli_state = ctx.state();
        let module_version = opt.version;
        let upgrade_package =
            dev_helper_vm2::load_package_from_file(opt.mv_or_package_file.as_path())?;
        eprintln!(
            "upgrade package address : {}",
            upgrade_package.package_address()
        );
        if upgrade_package.package_address() != opt.dao_token.address {
            bail!(
                "the package address {} not match the dao token: {}",
                upgrade_package.package_address(),
                opt.dao_token
            );
        }
        let min_action_delay = get_dao_config(cli_state)?.min_action_delay;
        let chain_state_reader = ctx
            .state()
            .client()
            .state_reader2(StateRootOption::Latest)?;
        let stdlib_version = chain_state_reader
            .get_on_chain_config::<Version>()
            .map(|version| version.major)
            .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;
        eprintln!(
            "current stdlib version {:?}",
            StdlibVersion::new(stdlib_version)
        );
        let (module_upgrade_proposal, package_hash) = build_module_upgrade_proposal(
            &upgrade_package,
            module_version,
            min_action_delay,
            opt.enforced,
            opt.dao_token.clone(),
            true,
        );
        eprintln!("package_hash {:?}", package_hash);
        ctx.state().vm2()?.build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::EntryFunction(module_upgrade_proposal),
        )
    }
}
