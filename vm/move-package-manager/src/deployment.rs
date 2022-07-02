// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use anyhow::ensure;
use clap::Parser;
use move_cli::Move;
use starcoin_account_provider::ProviderFactory;
use starcoin_cmd::dev::dev_helper;
use starcoin_cmd::view::TransactionOptions;
use starcoin_cmd::CliState;
use starcoin_config::account_provider_config::AccountProviderConfig;
use starcoin_rpc_client::RpcClient;
use starcoin_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
pub struct DeploymentCommand {
    #[clap(name = "rpc", long)]
    /// use remote starcoin rpc as initial state.
    rpc: String,

    #[clap(flatten)]
    pub account_provider: AccountProviderConfig,

    #[clap(long = "password")]
    pub account_passwd: Option<String>,

    #[clap(long = "sequence-number")]
    /// transaction's sequence_number
    /// if a transaction in the pool, you want to replace it, can use this option to set transaction's sequence_number
    /// otherwise please let cli to auto get sequence_number from onchain and txpool.
    pub sequence_number: Option<u64>,

    #[clap(long = "max-gas-amount")]
    /// max gas used to deploy the module
    pub max_gas_amount: Option<u64>,

    #[clap(long = "gas-price", name = "price of gas unit")]
    /// gas price used to deploy the module
    pub gas_unit_price: Option<u64>,

    #[clap(name = "expiration-time-secs", long = "expiration-time-secs")]
    /// how long(in seconds) the txn stay alive from now
    pub expiration_time_secs: Option<u64>,

    #[clap(short = 'b', name = "blocking-mode", long = "blocking")]
    /// blocking wait transaction(txn) mined
    pub blocking: bool,

    #[clap(name = "mv-or-package-file")]
    /// move bytecode file path or package binary path
    mv_or_package_file: PathBuf,
}

pub fn handle_deployment(_move_args: &Move, cmd: DeploymentCommand) -> anyhow::Result<()> {
    let client = RpcClient::connect_websocket(&cmd.rpc)?;

    let node_info = client.node_info()?;
    let client = Arc::new(client);
    let node_handle = None;

    assert!(
        cmd.account_provider.account_dir.is_some() || cmd.account_provider.secret_file.is_some(),
        "Please provide an account provider."
    );
    let package = dev_helper::load_package_from_file(cmd.mv_or_package_file.as_path())?;
    let package_address = package.package_address();

    let provider = if cmd.account_provider.account_dir.is_some() {
        let local_provider = ProviderFactory::create_provider(
            client.clone(),
            node_info.net.chain_id(),
            &AccountProviderConfig::new_local_provider_config(
                cmd.account_provider.account_dir.clone().unwrap(),
            ),
        )?;
        if cmd.account_provider.account_dir.is_some() {
            local_provider.unlock_account(
                package_address,
                cmd.account_passwd.unwrap(),
                Duration::from_secs(300),
            )?;
        }
        local_provider
    } else {
        let private_key_provider = ProviderFactory::create_provider(
            client.clone(),
            node_info.net.chain_id(),
            &AccountProviderConfig::new_private_key_provider_config(
                cmd.account_provider.secret_file.clone().unwrap(),
                cmd.account_provider.account_address,
            ),
        )?;
        let sender = cmd
            .account_provider
            .account_address
            .unwrap_or(package_address);
        ensure!(
            sender == package_address,
            "please use package address({}) account to deploy package, currently sender is {}.",
            package_address,
            sender
        );
        private_key_provider.unlock_account(
            package_address,
            "".to_string(),
            Duration::from_secs(300),
        )?;
        private_key_provider
    };

    let state = CliState::new(node_info.net, client, None, node_handle, provider);

    let transaction_opts = TransactionOptions {
        sender: Some(package_address),
        sequence_number: cmd.sequence_number,
        max_gas_amount: cmd.max_gas_amount,
        gas_unit_price: cmd.gas_unit_price,
        expiration_time_secs: cmd.expiration_time_secs,
        blocking: cmd.blocking,
        dry_run: false,
    };

    let item =
        state.build_and_execute_transaction(transaction_opts, TransactionPayload::Package(package));
    match item {
        Ok(_) => println!("The deployment is successful."),
        Err(e) => {
            println!("The deployment is failed. Reason: ");
            println!("{:?}", e);
        }
    }
    Ok(())
}
