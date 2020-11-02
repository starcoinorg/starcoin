// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::upgrade_stdlib::sign_txn_with_association_account_by_rpc_client;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_transaction_builder::build_stdlib_package;
use starcoin_vm_types::genesis_config::ChainNetwork;
use starcoin_vm_types::transaction::TransactionPayload;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use stdlib::StdLibOptions;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "stdlib_exe")]
pub struct UpgradeStdlibExeOpt {
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
        short = "s",
        name = "stdlib-file",
        long = "stdlib",
        help = "path for stdlib file, can be empty.",
        parse(from_os_str)
    )]
    stdlib_file: Option<PathBuf>,
}

pub struct UpgradeStdlibExeCommand;

impl CommandAction for UpgradeStdlibExeCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeStdlibExeOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let cli_state = ctx.state();
        let net = ChainNetwork::new_builtin(
            *cli_state
                .net()
                .as_builtin()
                .ok_or_else(|| format_err!("Only support builtin network"))?,
        );
        let upgrade_package = if let Some(stdlib_file) = &opt.stdlib_file {
            let mut bytes = vec![];
            File::open(stdlib_file)?.read_to_end(&mut bytes)?;
            scs::from_bytes(&bytes)?
        } else {
            build_stdlib_package(&net, StdLibOptions::Fresh, false)?
        };

        let signed_txn = sign_txn_with_association_account_by_rpc_client(
            cli_state,
            opt.max_gas_amount,
            opt.gas_price,
            opt.expiration_time,
            TransactionPayload::Package(upgrade_package),
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
        Ok(txn_hash)
    }
}
