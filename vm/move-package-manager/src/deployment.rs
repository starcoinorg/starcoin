// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

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
use vm_status_translator::VmStatusExplainView;

#[derive(Parser)]
pub struct DeploymentCommand {
    #[clap(name = "rpc", long)]
    /// use remote starcoin rpc as initial state.
    rpc: String,

    #[clap(flatten)]
    pub account_provider: AccountProviderConfig,

    #[clap(long = "password")]
    pub account_passwd: Option<String>,

    #[clap(flatten)]
    txn_opts: TransactionOptions,

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
        cmd.account_provider.account_dir.is_some()
            || cmd.account_provider.secret_file.is_some()
            || cmd.account_provider.from_env,
        "Please provide an account provider."
    );
    let package = dev_helper::load_package_from_file(cmd.mv_or_package_file.as_path())?;
    let package_address = package.package_address();

    let mut txn_opts = cmd.txn_opts.clone();
    if txn_opts.sender.is_none() {
        println!(
            "Use package address ({}) as transaction sender",
            package_address
        );
        txn_opts.sender = Some(package_address);
    };

    let provider = if cmd.account_provider.account_dir.is_some() {
        let local_provider = ProviderFactory::create_provider(
            client.clone(),
            node_info.net.chain_id(),
            &AccountProviderConfig::new_local_provider_config(
                cmd.account_provider.account_dir.clone().unwrap(),
            )?,
        )?;
        if cmd.account_provider.account_dir.is_some() {
            local_provider.unlock_account(
                package_address,
                cmd.account_passwd.unwrap_or_else(|| {
                    println!("No password given, use empty String.");
                    String::from("")
                }),
                Duration::from_secs(300),
            )?;
        }
        local_provider
    } else {
        let private_key_provider = ProviderFactory::create_provider(
            client.clone(),
            node_info.net.chain_id(),
            &AccountProviderConfig::new_private_key_provider_config(
                cmd.account_provider.secret_file.clone(),
                txn_opts.sender,
                cmd.account_provider.from_env,
            )?,
        )?;
        private_key_provider.unlock_account(
            txn_opts.sender.unwrap(),
            "".to_string(),
            Duration::from_secs(300),
        )?;
        private_key_provider
    };

    // TODO(BobOng):[dual-vm] to support vm2 deployment in mvm command
    let state = CliState::new(node_info.net, client, None, node_handle, provider, None);

    let item =
        state.build_and_execute_transaction(cmd.txn_opts, TransactionPayload::Package(package));
    match item {
        Ok(execute_result_view) => {
            // check dry run result
            if VmStatusExplainView::Executed == execute_result_view.dry_run_output.explained_status
            {
                println!("The deployment is successful.");
            } else {
                println!("The deployment is failed. execute result view is: ");
                println!("{:?}", execute_result_view);
            }
        }
        Err(e) => {
            println!("The deployment is failed. Reason: ");
            println!("{:?}", e);
        }
    }
    Ok(())
}
