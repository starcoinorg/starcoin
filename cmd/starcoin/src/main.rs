// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use anyhow::Result;
use scmd::{CmdContext, Command, CommandAction};
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;

use starcoin_config::ChainNetwork;
pub use starcoin_config::StarcoinOpt;

mod account;
mod debug;
mod txn;

mod helper;
pub mod state;

fn run() -> Result<()> {
    let _logger_handle = starcoin_logger::init();
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        |opt| -> Result<CliState> {
            info!("Starcoin opts: {:?}", opt);
            let config = Arc::new(starcoin_config::load_config_with_opt(opt)?);
            info!("Final data-dir is : {:?}", config.data_dir());
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
            let config = state.config();
            info!(
                "Attach a new console by command: starcoin -n {} -d {:?} console",
                config.net(),
                config.base.base_data_dir()
            );
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
            Command::with_name("account")
                .subcommand(account::CreateCommand {}.into_cmd())
                .subcommand(account::ShowCommand {}.into_cmd())
                .subcommand(account::ListCommand {}.into_cmd())
                .subcommand(account::SignTxnCommand {}.into_cmd())
                .subcommand(account::UnlockCommand.into_cmd())
                .subcommand(account::ExportCommand.into_cmd())
                .subcommand(account::ImportCommand.into_cmd()),
        )
        .command(Command::with_name("txn").subcommand(txn::TransferCommand {}.into_cmd()))
        .command(Command::with_name("debug").subcommand(debug::LogLevelCommand {}.into_cmd()))
        .exec()
}

//TODO error and crash handle.
fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => panic!(format!("Unexpect error: {:?}", e)),
    }
}
