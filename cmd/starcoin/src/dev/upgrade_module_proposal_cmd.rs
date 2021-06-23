// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::sign_txn_helper::get_dao_config;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::{format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::StateReaderExt;
use starcoin_transaction_builder::build_module_upgrade_proposal;
use starcoin_types::transaction::Package;
use starcoin_vm_types::genesis_config::StdlibVersion;
use starcoin_vm_types::on_chain_config::Version;
use starcoin_vm_types::transaction::TransactionPayload;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

/// Submit a module upgrade proposal
#[derive(Debug, StructOpt)]
#[structopt(name = "module-proposal", alias = "module_proposal")]
pub struct UpgradeModuleProposalOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(
        short = "e",
        name = "enforced",
        long = "enforced",
        help = "enforced upgrade regardless of compatible or not"
    )]
    enforced: bool,

    #[structopt(
        short = "m",
        name = "module-package-file",
        long = "module",
        help = "path for module package file.",
        parse(from_os_str)
    )]
    module_package_file: PathBuf,

    #[structopt(
        short = "v",
        name = "module-version",
        long = "module_version",
        help = "new version number for the module"
    )]
    version: u64,
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
        let module_file = opt.module_package_file.as_path();
        let mut bytes = vec![];
        File::open(module_file)?.read_to_end(&mut bytes)?;
        let upgrade_package: Package = bcs_ext::from_bytes(&bytes)?;
        eprintln!(
            "upgrade package address : {:?}",
            upgrade_package.package_address()
        );
        let min_action_delay = get_dao_config(cli_state)?.min_action_delay;
        let chain_state_reader = RemoteStateReader::new(ctx.state().client())?;
        let stdlib_version = chain_state_reader
            .get_on_chain_config::<Version>()?
            .map(|version| version.major)
            .ok_or_else(|| format_err!("on chain config stdlib version can not be empty."))?;
        eprintln!("stdlib version {:?}", StdlibVersion::new(stdlib_version));
        let (module_upgrade_proposal, package_hash) = build_module_upgrade_proposal(
            &upgrade_package,
            module_version,
            min_action_delay,
            opt.enforced,
            StdlibVersion::new(stdlib_version),
        );
        eprintln!("package_hash {:?}", package_hash);
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::ScriptFunction(module_upgrade_proposal),
        )
    }
}
