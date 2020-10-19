// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use scmd::CmdContext;
use starcoin_cmd::*;
use starcoin_cmd::{CliState, StarcoinOpt};
use starcoin_config::Connect;
use starcoin_logger::prelude::*;
use starcoin_node::crash_handler;
use starcoin_node_api::errors::NodeStartError;
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;
use tokio::time::Duration;

/// This exit code means is that the node failed to start and required human intervention.
/// Node start script can do auto task when meet this exist code.
static EXIT_CODE_NEED_HELP: i32 = 120;

fn run() -> Result<()> {
    let logger_handle = starcoin_logger::init();
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        |opt| -> Result<CliState> {
            info!("Starcoin opts: {:?}", opt);
            let mut rt = tokio_compat::runtime::Runtime::new()?;
            let connect = opt.connect.as_ref().unwrap_or(&Connect::IPC(None));
            let (client, node_handle) = match connect {
                Connect::IPC(ipc_file) => {
                    if let Some(ipc_file) = ipc_file {
                        info!("Try to connect node by ipc: {:?}", ipc_file);
                        let client = RpcClient::connect_ipc(ipc_file, &mut rt)?;
                        (client, None)
                    } else {
                        info!("Start starcoin node...");
                        let (node_handle, config) = starcoin_node::run_node_by_opt(opt)?;
                        match node_handle {
                            //first cli use local connect.
                            Some(node_handle) => {
                                info!("Connect by in process channel");
                                let rpc_service = node_handle.rpc_service()?;
                                let client = RpcClient::connect_local(rpc_service)?;
                                (client, Some(node_handle))
                            }
                            None => {
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
                                let client = RpcClient::connect_ipc(ipc_file, &mut rt)?;
                                (client, None)
                            }
                        }
                    }
                }
                Connect::WebSocket(address) => {
                    info!("Try to connect node by websocket: {:?}", address);
                    let client = RpcClient::connect_websocket(address, &mut rt)?;
                    (client, None)
                }
            };

            let node_info = client.node_info()?;
            let state = CliState::new(
                node_info.net,
                Arc::new(client),
                opt.watch_timeout.map(Duration::from_secs),
                node_handle,
                Some(rt),
            );
            Ok(state)
        },
        |_, _, state| {
            let (_, client, handle) = state.into_inner();
            match Arc::try_unwrap(client) {
                Err(_) => {
                    error!("Can not close rpc client normal.");
                }
                Ok(client) => {
                    client.close();
                }
            }
            if let Some(handle) = handle {
                if let Err(e) = handle.join() {
                    error!("{:?}", e);
                }
            }
        },
    );
    let context = context.with_console_support(
        move |_app, _opt, state| {
            info!("Start console, disable stderr output.");
            logger_handle.disable_stderr();
            (*scmd::DEFAULT_CONSOLE_CONFIG, Some(state.history_file()))
        },
        |_, _, state| {
            let (_, _, handle) = state.into_inner();
            if let Some(handle) = handle {
                if let Err(e) = handle.stop() {
                    error!("{:?}", e);
                }
            }
        },
    );
    add_command(context).exec()
}

fn main() {
    crash_handler::setup_panic_handler();
    match run() {
        Ok(()) => {}
        Err(e) => {
            match e.downcast::<NodeStartError>() {
                Ok(e) => match e {
                    //TODO not suggest clean data dir in main network.
                    NodeStartError::LoadConfigError(e) => {
                        error!("{:?}, please fix config.", e);
                        std::process::exit(EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::StorageInitError(e) => {
                        error!("{:?}, please clean your data dir.", e);
                        std::process::exit(EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::GenesisError(e) => {
                        error!("{:?}, please clean your data dir.", e);
                        std::process::exit(EXIT_CODE_NEED_HELP);
                    }
                    _ => {
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    error!("Node exits abnormally: {:?}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
