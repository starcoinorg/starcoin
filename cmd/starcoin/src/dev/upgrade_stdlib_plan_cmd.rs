// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::dev::upgrade_stdlib::to_txn_with_association_account_by_rpc_client;
use crate::StarcoinOpt;
use anyhow::{bail, format_err, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_transaction_builder::build_module_upgrade_plan;
use starcoin_vm_types::genesis_config::ChainNetwork;
use starcoin_vm_types::transaction::TransactionPayload;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "upgrade_stdlib_plan")]
pub struct UpgradeStdlibPlanOpt {
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
        name = "stdlib-proposal-id",
        long = "stdlib",
        help = "proposal id."
    )]
    proposal_id: u64,
}

pub struct UpgradeStdlibPlanCommand;

impl CommandAction for UpgradeStdlibPlanCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeStdlibPlanOpt;
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

        let module_upgrade_plan = build_module_upgrade_plan(net, opt.proposal_id);
        let signed_txn = to_txn_with_association_account_by_rpc_client(
            cli_state,
            opt.max_gas_amount,
            opt.gas_price,
            opt.expiration_time,
            TransactionPayload::Script(module_upgrade_plan),
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
