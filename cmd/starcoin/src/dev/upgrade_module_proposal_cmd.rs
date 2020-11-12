// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::sign_txn_helper::{get_dao_config, sign_txn_with_account_by_rpc_client};
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_logger::prelude::*;
use starcoin_transaction_builder::build_module_upgrade_proposal;
use starcoin_types::transaction::Package;
use starcoin_vm_types::account_address::AccountAddress;
use starcoin_vm_types::transaction::TransactionPayload;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "module_proposal")]
pub struct UpgradeModuleProposalOpt {
    #[structopt(short = "s", long)]
    /// hex encoded string, like 0x1, 0x12
    sender: Option<AccountAddress>,

    #[structopt(
        short = "g",
        name = "max-gas-amount",
        default_value = "1000000",
        help = "max gas used to deploy the module"
    )]
    max_gas_amount: u64,
    #[structopt(
        short = "p",
        long = "gas-price",
        name = "price of gas",
        default_value = "1",
        help = "gas price used to deploy the module"
    )]
    gas_price: u64,

    #[structopt(
        name = "expiration_time",
        long = "timeout",
        default_value = "3000",
        help = "how long(in seconds) the txn stay alive"
    )]
    expiration_time: u64,

    #[structopt(
        short = "b",
        name = "blocking-mode",
        long = "blocking",
        help = "blocking wait txn mined"
    )]
    blocking: bool,

    #[structopt(
        short = "m",
        name = "module-file",
        long = "module",
        help = "path for module file, can be empty.",
        parse(from_os_str)
    )]
    module_file: Option<PathBuf>,

    #[structopt(
        short = "v",
        name = "module-version",
        long = "module_version",
        default_value = "1",
        help = "module version"
    )]
    version: u64,
}

pub struct UpgradeModuleProposalCommand;

impl CommandAction for UpgradeModuleProposalCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeModuleProposalOpt;
    type ReturnItem = (HashValue, HashValue);

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let cli_state = ctx.state();
        let sender = if let Some(sender) = ctx.opt().sender {
            sender
        } else {
            ctx.state().default_account()?.address
        };
        if let Some(module_file) = &opt.module_file {
            let mut bytes = vec![];
            File::open(module_file)?.read_to_end(&mut bytes)?;
            let upgrade_package: Package = scs::from_bytes(&bytes)?;
            info!(
                "upgrade package address : {:?}",
                upgrade_package.package_address()
            );

            let min_action_delay = get_dao_config(cli_state)?.min_action_delay;
            let (module_upgrade_proposal, package_hash) =
                build_module_upgrade_proposal(&upgrade_package, opt.version, min_action_delay);
            let signed_txn = sign_txn_with_account_by_rpc_client(
                cli_state,
                sender,
                opt.max_gas_amount,
                opt.gas_price,
                opt.expiration_time,
                TransactionPayload::Script(module_upgrade_proposal),
            )?;
            let txn_hash = signed_txn.crypto_hash();
            let success = cli_state.client().submit_transaction(signed_txn)?;
            if let Err(e) = success {
                bail!("execute-txn is reject by node, reason: {}", &e)
            }
            println!("txn {:#x} submitted.", txn_hash);

            if opt.blocking {
                ctx.state().watch_txn(txn_hash)?;
            }
            Ok((package_hash, txn_hash))
        } else {
            bail!("file can not be empty.")
        }
    }
}
