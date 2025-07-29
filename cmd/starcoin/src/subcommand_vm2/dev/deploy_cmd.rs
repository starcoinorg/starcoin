// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    cli_state::CliState, subcommand_vm2::dev::dev_helper_vm2, view::TransactionOptions,
    view_vm2::ExecuteResultView, StarcoinOpt,
};
use anyhow::Result;
use clap::Parser;
use scmd::{CommandAction, ExecContext};
use starcoin_types::account_address::AccountAddress as AccountAddress1;
use starcoin_vm2_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;

/// Deploy Move modules
#[derive(Debug, Parser)]
#[clap(name = "deploy")]
pub struct DeployOpt {
    #[clap(flatten)]
    transaction_opts: TransactionOptions,

    #[clap(name = "mv-or-package-file")]
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
        let package = dev_helper_vm2::load_package_from_file(opt.mv_or_package_file.as_path())?;

        let package_address = package.package_address();
        let mut transaction_opts = opt.transaction_opts.clone();
        match transaction_opts.sender.as_ref() {
            Some(_sender) => {}
            None => {
                eprintln!(
                    "Use package address ({}) as transaction sender",
                    package_address
                );
                transaction_opts.sender = Some(AccountAddress1::new(package_address.into_bytes()));
            }
        };

        ctx.state()
            .vm2()?
            .build_and_execute_transaction(transaction_opts, TransactionPayload::Package(package))
    }
}
