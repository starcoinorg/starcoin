// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_crypto::hash::{HashValue, PlainCryptoHash};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_transaction_builder::build_stdlib_package;
use starcoin_types::transaction::RawUserTransaction;
use starcoin_vm_types::chain_config::ChainId;
use starcoin_vm_types::transaction::helpers::get_current_timestamp;
use starcoin_vm_types::transaction::TransactionPayload;
use stdlib::StdLibOptions;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "upgrade_stdlib")]
pub struct UpgradeStdlibOpt {
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
}

pub struct UpgradeStdlibCommand;

impl CommandAction for UpgradeStdlibCommand {
    type State = CliState;
    type GlobalOpt = StarcoinOpt;
    type Opt = UpgradeStdlibOpt;
    type ReturnItem = HashValue;

    fn run(
        &self,
        ctx: &ExecContext<Self::State, Self::GlobalOpt, Self::Opt>,
    ) -> Result<Self::ReturnItem> {
        let opt = ctx.opt();
        let cli_state = ctx.state();
        let association_account = cli_state.association_account()?;
        let client = ctx.state().client();
        if let Some(association_account) = association_account {
            let chain_state_reader = RemoteStateReader::new(client);
            let account_state_reader = AccountStateReader::new(&chain_state_reader);
            let account_resource = account_state_reader
                .get_account_resource(association_account.address())?
                .unwrap_or_else(|| panic!("association_account must exist on chain."));
            let upgrade_package =
                build_stdlib_package(cli_state.net(), StdLibOptions::Fresh, false)?;

            let expiration_time = opt.expiration_time + get_current_timestamp();
            let upgrade_txn = RawUserTransaction::new(
                association_account.address,
                account_resource.sequence_number(),
                TransactionPayload::Package(upgrade_package),
                opt.max_gas_amount,
                opt.gas_price,
                expiration_time,
                ChainId::id(ctx.state().net()),
            );
            let signed_txn = client.wallet_sign_txn(upgrade_txn)?;
            let txn_hash = signed_txn.crypto_hash();
            let success = client.submit_transaction(signed_txn)?;
            if let Err(e) = success {
                bail!("execute-txn is reject by node, reason: {}", &e)
            }
            println!("txn {:#x} submitted.", txn_hash);

            if opt.blocking {
                ctx.state().watch_txn(txn_hash)?;
            }
            Ok(txn_hash)
        } else {
            bail!(
                "association_account not exists in wallet, please import association_account first.",
            );
        }
    }
}
