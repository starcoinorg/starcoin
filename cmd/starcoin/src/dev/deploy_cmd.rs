// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::dev_helper;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::{ensure, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;
use structopt::StructOpt;

/// Deploy Move modules
#[derive(Debug, StructOpt)]
#[structopt(name = "deploy")]
pub struct DeployOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(name = "mv-or-package-file")]
    /// move bytecode file path or package binary path
    mv_or_package_file: PathBuf,
}

pub struct DeployCommand;

impl CommandAction for DeployCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = DeployOpt;
    type ReturnItem = ExecuteResultView;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let package = dev_helper::load_package_from_file(opt.mv_or_package_file.as_path())?;

        let package_address = package.package_address();
        let mut transaction_opts = opt.transaction_opts.clone();
        match transaction_opts.sender.as_ref() {
            Some(sender) => {
                ensure!(*sender == package_address, "please use package address({}) account to deploy package, currently sender is {}.", package_address,sender);
            }
            None => {
                eprintln!(
                    "Use package address ({}) as transaction sender",
                    package_address
                );
                transaction_opts.sender = Some(package_address);
            }
        };

        ctx.state()
            .build_and_execute_transaction(transaction_opts, TransactionPayload::Package(package))
    }
}
