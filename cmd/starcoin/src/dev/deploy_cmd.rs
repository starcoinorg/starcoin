// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::cli_state::CliState;
use crate::view::{ExecuteResultView, ExecutionOutputView};
use crate::StarcoinOpt;
use anyhow::{bail, Result};
use scmd::{CommandAction, ExecContext};
use starcoin_dev::playground;
use starcoin_rpc_api::types::{TransactionOutputView, TransactionVMStatus};
use starcoin_rpc_client::RemoteStateReader;
use starcoin_state_api::AccountStateReader;
use starcoin_types::transaction::{DryRunTransaction, Module, RawUserTransaction};
use starcoin_vm_types::{access::ModuleAccess, file_format::CompiledModule};
use std::fs::OpenOptions;
use std::io::Read;
use structopt::StructOpt;

/// Deploy Move modules
#[derive(Debug, StructOpt)]
#[structopt(name = "deploy")]
pub struct DeployOpt {
    #[structopt(
        short = "g",
        name = "max-gas-amount",
        default_value = "10000000",
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

    #[structopt(long = "dry-run")]
    /// dry-run mode, only get transaction output, no state change to chain
    dry_run: bool,

    #[structopt(name = "bytecode_file", help = "module bytecode file path")]
    bytecode_file: String,
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
        let bytecode_path = ctx.opt().bytecode_file.clone();
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(bytecode_path)?;
        let mut bytecode = vec![];
        file.read_to_end(&mut bytecode)?;
        let compiled_module = match CompiledModule::deserialize(bytecode.as_slice()) {
            Err(e) => {
                bail!("invalid bytecode file, cannot deserialize as module, {}", e);
            }
            Ok(compiled_module) => compiled_module,
        };
        let module_address = *compiled_module.address();
        let client = ctx.state().client();
        let node_info = client.node_info()?;
        let chain_state_reader = RemoteStateReader::new(client)?;
        let account_state_reader = AccountStateReader::new(&chain_state_reader);
        let account_resource = account_state_reader.get_account_resource(&module_address)?;

        if account_resource.is_none() {
            bail!(
                "account of module address {} not exists on chain",
                &module_address
            );
        }

        let account_resource = account_resource.unwrap();

        let expiration_time = opt.expiration_time + node_info.now_seconds;
        let deploy_txn = RawUserTransaction::new_module(
            module_address,
            account_resource.sequence_number(),
            Module::new(bytecode),
            opt.max_gas_amount,
            opt.gas_price,
            expiration_time,
            ctx.state().net().chain_id(),
        );

        let signed_txn = client.account_sign_txn(deploy_txn)?;
        let txn_hash = signed_txn.id();

        let output: TransactionOutputView = {
            let state_view = RemoteStateReader::new(client)?;
            playground::dry_run(
                &state_view,
                DryRunTransaction {
                    public_key: signed_txn.authenticator().public_key(),
                    raw_txn: signed_txn.raw_txn().clone(),
                },
            )
            .map(|(_, b)| b.into())?
        };
        match output.status {
            TransactionVMStatus::Discard { status_code } => {
                bail!("TransactionStatus is discard: {:?}", status_code)
            }
            TransactionVMStatus::Executed => {}
            s => {
                bail!("pre-run failed, status: {:?}", s);
            }
        }

        if !opt.dry_run {
            client.submit_transaction(signed_txn)?;

            println!("txn {:#x} submitted.", txn_hash);

            let mut output_view = ExecutionOutputView::new(txn_hash);

            if opt.blocking {
                let block = ctx.state().watch_txn(txn_hash)?.0;
                output_view.block_number = Some(block.header.number.0);
                output_view.block_id = Some(block.header.block_hash);
            }
            Ok(ExecuteResultView::Run(output_view))
        } else {
            Ok(ExecuteResultView::DryRun(output.into()))
        }
    }
}
