// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::dev_helper;
use crate::dev::sign_txn_helper::get_dao_config;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::StateRootOption;
use starcoin_state_api::StateReaderExt;
use starcoin_transaction_builder::build_module_upgrade_proposal;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::on_chain_config::Version;
use starcoin_vm_types::token::token_code::TokenCode;
use starcoin_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;
use structopt::StructOpt;

/// Submit a module upgrade proposal
#[derive(Debug, StructOpt)]
#[structopt(name = "module-proposal", alias = "module_proposal")]
pub struct UpgradeModuleProposalOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(short = "e", name = "enforced", long = "enforced")]
    /// enforced upgrade regardless of compatible or not
    enforced: bool,

    #[structopt(
        short = "m",
        name = "mv-or-package-file",
        long = "mv-or-package-file",
        parse(from_os_str)
    )]
    /// path for module or package file.
    mv_or_package_file: PathBuf,

    #[structopt(short = "v", name = "module-version", long = "module-version")]
    /// new version number for the modules
    version: u64,

    #[structopt(
        name = "dao-token",
        long = "dao-token",
        default_value = "0x1::STC::STC"
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
        let upgrade_package = dev_helper::load_package_from_file(opt.mv_or_package_file.as_path())?;
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
        let chain_state_reader = ctx.state().client().state_reader(StateRootOption::Latest)?;
        let stdlib_version = chain_state_reader
            .get_on_chain_config::<Version>()?
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
            StdlibVersion::new(stdlib_version),
        );
        eprintln!("package_hash {:?}", package_hash);
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(module_upgrade_proposal),
        )
    }
}
