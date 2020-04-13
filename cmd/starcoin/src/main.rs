// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use anyhow::Result;
use scmd::{CmdContext, Command};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;

use starcoin_config::ChainNetwork;
pub use starcoin_config::StarcoinOpt;

mod debug;
mod dev;
mod helper;
mod txn;
mod wallet;

pub mod state;

fn run() -> Result<()> {
    let logger_handle = starcoin_logger::init();
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        |opt| -> Result<CliState> {
            info!("Starcoin opts: {:?}", opt);
            let config = Arc::new(starcoin_config::load_config_with_opt(opt)?);
            info!("Final data-dir is : {:?}", config.data_dir());
            info!(
                "Attach a new console by command: starcoin -n {} -d {} console",
                config.net(),
                config.base.base_data_dir().to_str().unwrap()
            );
            let ipc_file = config.rpc.get_ipc_file();
            let node_handle = if !ipc_file.exists() {
                let node_handle = match config.net() {
                    ChainNetwork::Dev => starcoin_node::run_dev_node(config.clone()),
                    _ => starcoin_node::run_normal_node(config.clone()),
                };
                info!("Waiting node start...");
                helper::wait_until_file_created(ipc_file)?;
                Some(node_handle)
            } else {
                None
            };
            info!("Try to connect node by ipc: {:?}", ipc_file);
            let client = RpcClient::connect_ipc(ipc_file)?;
            let state = CliState::new(config, client, node_handle);
            Ok(state)
        },
        |_, _, state| {
            let (_, _, handle) = state.into_inner();
            match handle {
                Some(handle) => match handle.join() {
                    Err(e) => {
                        error!("{:?}", e);
                    }
                    _ => {}
                },
                None => {}
            }
        },
        move |_, _, _| {
            info!("Start console, disable stderr output.");
            logger_handle.disable_stderr();
        },
        |_, _, state| {
            let (_, _, handle) = state.into_inner();
            match handle {
                Some(handle) => match handle.stop() {
                    Err(e) => {
                        error!("{:?}", e);
                    }
                    _ => {}
                },
                None => {}
            }
        },
    );
    context
        .command(
            Command::with_name("wallet")
                .subcommand(wallet::CreateCommand)
                .subcommand(wallet::ShowCommand)
                .subcommand(wallet::ListCommand)
                .subcommand(wallet::SignTxnCommand)
                .subcommand(wallet::UnlockCommand)
                .subcommand(wallet::ExportCommand)
                .subcommand(wallet::ImportCommand),
        )
        .command(Command::with_name("txn").subcommand(txn::TransferCommand))
        .command(Command::with_name("dev").subcommand(dev::GetCoinCommand))
        .command(Command::with_name("debug").subcommand(debug::LogLevelCommand))
        .exec()
}

//TODO error and crash handle.
fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => panic!(format!("Unexpect error: {:?}", e)),
    }
}
