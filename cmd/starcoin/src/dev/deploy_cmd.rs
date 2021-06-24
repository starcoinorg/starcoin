// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, TransactionOptions};
use crate::StarcoinOpt;
use anyhow::{ensure, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_move_compiler::MOVE_COMPILED_EXTENSION;
use starcoin_types::transaction::{Module, Package};
use starcoin_vm_types::transaction::TransactionPayload;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

/// Deploy Move modules
#[derive(Debug, StructOpt)]
#[structopt(name = "deploy")]
pub struct DeployOpt {
    #[structopt(flatten)]
    transaction_opts: TransactionOptions,

    #[structopt(
        name = "mv-or-package-file",
        help = "move bytecode file path or package binary path"
    )]
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
        let mv_or_package_file = ctx.opt().mv_or_package_file.as_path();
        ensure!(
            mv_or_package_file.exists(),
            "file {:?} not exist",
            mv_or_package_file
        );

        let mut bytes = vec![];
        File::open(mv_or_package_file)?.read_to_end(&mut bytes)?;

        let package =
            if mv_or_package_file.extension().unwrap_or_default() == MOVE_COMPILED_EXTENSION {
                Package::new_with_module(Module::new(bytes))?
            } else {
                bcs_ext::from_bytes(&bytes)?
            };

        let package_address = package.package_address();
        let mut transaction_opts = opt.transaction_opts.clone();
        match transaction_opts.sender.as_ref() {
            Some(sender) => {
                ensure!(*sender == package_address, "please use package address({}) account to deploy package, currently sender is {}.", package_address,sender);
            }
            None => {
                transaction_opts.sender = Some(package_address);
            }
        };

        ctx.state().build_and_execute_transaction(
            opt.transaction_opts.clone(),
            TransactionPayload::Package(package),
        )
    }
}
