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
use starcoin_logger::prelude::info;
use starcoin_rpc_client::RpcClient;
use starcoin_vm_types::transaction::TransactionPayload;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Parser)]
pub struct DeploymentCommand {
    #[clap(name = "rpc", long)]
    /// use remote starcoin rpc as initial state.
    rpc: String,

    #[clap(long = "block-number", requires("rpc"))]
    /// block number to read state from. default to latest block number.
    block_number: Option<u64>,

    #[clap(long = "watch-timeout")]
    /// Watch timeout in seconds
    pub watch_timeout: Option<u64>,

    #[clap(long = "local-account-dir", parse(from_os_str))]
    pub account_dir: Option<PathBuf>,

    #[clap(long = "password")]
    pub account_passwd: Option<String>,

    #[clap(long = "secret-file",
        help = "file path of private key",
        parse(from_os_str),
        conflicts_with("local-account-dir")
    )]
    pub secret: Option<PathBuf>,

    #[clap(name = "mv-or-package-file")]
    /// move bytecode file path or package binary path
    mv_or_package_file: PathBuf,
}

pub fn handle_deployment(_move_args: &Move, cmd: DeploymentCommand) -> anyhow::Result<()> {
    info!("Try to connect node by websocket: {:?}", cmd.rpc);
    let client = RpcClient::connect_websocket(&cmd.rpc)?;

    let node_info = client.node_info()?;
    let client = Arc::new(client);
    let node_handle = None;
    let account_config = AccountProviderConfig::new_local_provider_config(cmd.account_dir.unwrap());
    let provider = ProviderFactory::create_provider(
        client.clone(),
        node_info.net.chain_id(),
        &account_config,
    )?;

    let package = dev_helper::load_package_from_file(cmd.mv_or_package_file.as_path())?;
    let package_address = package.package_address();

    if cmd.account_passwd.is_some() {
        provider.unlock_account(
            package_address,
            cmd.account_passwd.unwrap(),
            Duration::from_secs(300),
        )?;
    }
    let state = CliState::new(
        node_info.net,
        client,
        cmd.watch_timeout.map(Duration::from_secs),
        node_handle,
        provider,
    );

    let mut transaction_opts = TransactionOptions {
        sender: Some(package_address),
        sequence_number: None,
        max_gas_amount: None,
        gas_unit_price: None,
        expiration_time_secs: None,
        blocking: true,
        dry_run: false,
    };
    match transaction_opts.sender.as_ref() {
        Some(sender) => {
            ensure!(
                *sender == package_address,
                "please use package address({}) account to deploy package, currently sender is {}.",
                package_address,
                sender
            );
        }
        None => {
            eprintln!(
                "Use package address ({}) as transaction sender",
                package_address
            );
            transaction_opts.sender = Some(package_address);
        }
    };

    let item =
        state.build_and_execute_transaction(transaction_opts, TransactionPayload::Package(package));
    println!("{:?}", item);
    if item.is_ok() {
        println!("Deploy successed.");
    } else {
        println!("Deploy failed.");
    };
    Ok(())
}
