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
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        Box::new(|opt| -> Result<CliState> {
            let logger_handle = starcoin_logger::init();
            info!("Starcoin opts: {:?}", opt);
            let config = Arc::new(starcoin_config::load_config_with_opt(opt)?);
            let node_handle = match config.net() {
                ChainNetwork::Dev => starcoin_node::run_dev_node(config.clone()),
                _ => starcoin_node::run_normal_node(config.clone()),
            };
            let ipc_file = config.rpc.get_ipc_file();
            info!("Waiting node start...");
            helper::wait_until_file_created(ipc_file)?;
            info!("Try to connect node by ipc: {:?}", ipc_file);
            let client = RpcClient::connect_ipc(ipc_file)?;
            let file_log_path = config.data_dir().join("starcoin.log");
            info!("Redirect log to file: {:?}", file_log_path);
            logger_handle.enable_file(false, file_log_path);
            let state = CliState::new(config, client, logger_handle, Some(node_handle));
            Ok(state)
        }),
        Box::new(|_, _, state| -> Result<()> {
            let (_, _, logger_handle, handle) = state.into_inner();
            match handle {
                Some(handle) => {
                    // if start node server and no subcommand, wait server and output logger to stderr.
                    logger_handle.enable_stderr();
                    match handle.join() {
                        Err(e) => {
                            error!("{:?}", e);
                        }
                        _ => {}
                    }
                }
                None => {}
            }
            Ok(())
        }),
    );
    context
        .command(
            Command::with_name("account")
                .subcommand(account::CreateCommand {}.into_cmd())
                .subcommand(account::ShowCommand {}.into_cmd())
                .subcommand(account::ListCommand {}.into_cmd())
                .subcommand(account::SignTxnCommand {}.into_cmd()),
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
