// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_state::CliState, dev::dev_helper_vm2, view::TransactionOptions,
    view_vm2::ExecuteResultView, StarcoinOpt,
};
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_vm2_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;

/// Execute module upgrade plan, submit a package transaction.
#[derive(Debug, Parser)]
#[clap(name = "module-exe", alias = "module_exe")]
pub struct UpgradeModuleExeOpt {
    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(
        short = 'm',
        name = "mv-or-package-file",
        long = "mv-or-package-file",
        parse(from_os_str)
    )]
    /// path for module or package file.
    mv_or_package_file: PathBuf,
}

pub struct UpgradeModuleExeCommand;

impl CommandAction for UpgradeModuleExeCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeModuleExeOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let upgrade_package =
            dev_helper_vm2::load_package_from_file(opt.mv_or_package_file.as_path())?;
        ctx.state().vm2()?.build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::Package(upgrade_package),
        )
    }
}
