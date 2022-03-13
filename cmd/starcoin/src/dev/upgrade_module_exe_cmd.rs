// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::dev_helper;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::Result;
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;
use structopt::StructOpt;

/// Execute module upgrade plan, submit a package transaction.
#[derive(Debug, StructOpt)]
#[structopt(name = "module-exe", alias = "module_exe")]
pub struct UpgradeModuleExeOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(
        short = "m",
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
        let upgrade_package = dev_helper::load_package_from_file(opt.mv_or_package_file.as_path())?;
        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::Package(upgrade_package),
        )
    }
}
