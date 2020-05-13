// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use crate::cli_state::CliState;
use anyhow::Result;
use scmd::{CmdContext, Command};
use starcoin_config::Connect;
pub use starcoin_config::StarcoinOpt;
use starcoin_logger::prelude::*;
use starcoin_rpc_client::RpcClient;

mod chain;
mod cli_state;
mod crash_handler;
mod debug;
mod dev;
mod helper;
mod node;
mod state;
mod txn;
mod view;
mod wallet;

fn run() -> Result<()> {
    let logger_handle = starcoin_logger::init();
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        |opt| -> Result<CliState> {
            info!("Starcoin opts: {:?}", opt);
            let connect = opt.connect.as_ref().unwrap_or(&Connect::IPC(None));
            let (client, node_handle) = match connect {
                Connect::IPC(ipc_file) => {
                    if let Some(ipc_file) = ipc_file {
                        info!("Try to connect node by ipc: {:?}", ipc_file);
                        let client = RpcClient::connect_ipc(ipc_file)?;
                        (client, None)
                    } else {
                        info!("Start starcoin node...");
                        let (node_handle, config) = starcoin_node::run_node_by_opt(opt)?;
                        let ipc_file = config.rpc.get_ipc_file();
                        helper::wait_until_file_created(ipc_file)?;
                        info!(
                            "Attach a new console by ipc: starcoin -c {} console",
                            ipc_file.to_str().expect("invalid ipc file path.")
                        );
                        if let Some(http_address) = config.rpc.get_http_address() {
                            info!(
                                "Attach a new console by rpc: starcoin -c {} console",
                                http_address
                            );
                        }
                        info!("Starcoin node started.");
                        info!("Try to connect node by ipc: {:?}", ipc_file);
                        let client = RpcClient::connect_ipc(ipc_file)?;
                        (client, node_handle)
                    }
                }
                Connect::WebSocket(address) => {
                    info!("Try to connect node by websocket: {:?}", address);
                    let client = RpcClient::connect_websocket(address)?;
                    (client, None)
                }
            };

            let node_info = client.node_info()?;
            let state = CliState::new(node_info.net, client, node_handle);
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
        .command(
            Command::with_name("state")
                .subcommand(state::GetCommand)
                .subcommand(state::GetAccountCommand)
                .subcommand(state::GetProofCommand)
                .subcommand(state::GetRootCommand),
        )
        .command(
            Command::with_name("node")
                .subcommand(node::InfoCommand)
                .subcommand(node::PeersCommand)
                .subcommand(node::MetricsCommand),
        )
        .command(
            Command::with_name("chain")
                .subcommand(chain::ShowCommand)
                .subcommand(chain::GetBlockByNumberCommand)
                .subcommand(chain::ListBlockCommand)
                .subcommand(chain::GetTransactionCommand)
                .subcommand(chain::GetTxnByBlockCommand)
                .subcommand(chain::GetBlockCommand)
                .subcommand(chain::BranchesCommand),
        )
        .command(
            Command::with_name("dev")
                .subcommand(dev::GetCoinCommand)
                .subcommand(dev::CompileCommand)
                .subcommand(dev::DeployCommand)
                .subcommand(dev::ExecuteCommand)
                .subcommand(
                    Command::with_name("subscribe")
                        .subcommand(dev::SubscribeBlockCommand)
                        .subcommand(dev::SubscribeEventCommand)
                        .subcommand(dev::SubscribeNewTxnCommand),
                ),
        )
        .command(
            Command::with_name("debug")
                .subcommand(debug::LogLevelCommand)
                .subcommand(debug::GenTxnCommand)
                .subcommand(debug::PanicCommand),
        )
        .exec();
    Ok(())
}

fn main() {
    crash_handler::setup_panic_handler();
    match run() {
        Ok(()) => {}
        Err(e) => panic!(format!("Unexpect error: {:?}", e)),
    }
}
