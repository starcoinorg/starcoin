// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
use anyhow::Result;
use scmd::error::CmdError;
use scmd::CmdContext;
use starcoin_account_provider::ProviderFactory;
use starcoin_cmd::subcommand_vm2::add_command_vm2;
use starcoin_cmd::*;
use starcoin_cmd::{CliState, StarcoinOpt};
use starcoin_config::{Connect, G_APP_VERSION, G_CRATE_VERSION};
use starcoin_logger::prelude::*;
use starcoin_node_api::errors::NodeStartError;
use starcoin_rpc_client::RpcClient;
use starcoin_vm2_vm_types::genesis_config::ChainId;
use std::sync::Arc;
use std::time::Duration;

/// This exit code means is that the node failed to start and required human intervention.
/// Node start script can do auto task when meet this exist code.
static G_EXIT_CODE_NEED_HELP: i32 = 120;

fn run() -> Result<()> {
    let logger_handle = starcoin_logger::init();
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        G_CRATE_VERSION,
        Some(G_APP_VERSION.as_str()),
        |opt| -> Result<CliState> {
            info!("Starcoin opts: {}", opt);
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
                                helper::wait_until_file_created(ipc_file.as_path())?;
                                info!(
                                    "Attach a new console by ipc: starcoin -c {} console",
                                    ipc_file.to_str().expect("invalid ipc file path.")
                                );
                                if let Some(ws_address) = config.rpc.get_ws_address() {
                                    info!(
                                        "Attach a new console by rpc: starcoin -c {} console",
                                        ws_address
                                    );
                                }
                                info!("Starcoin node started.");
                                info!("Try to connect node by ipc: {:?}", ipc_file);
                                let client = RpcClient::connect_ipc(ipc_file)?;
                                (client, None)
                            }
                        }
                    }
                }
                Connect::WebSocket(address) => {
                    info!("Try to connect node by websocket: {:?}", address);
                    let client = RpcClient::connect_websocket(address)?;
                    (client, None)
                }
            };

            let node_info = client.node_info()?;
            let client = Arc::new(client);

            let account_provider = ProviderFactory::create_provider(
                client.clone(),
                node_info.net.chain_id(),
                &opt.account_provider,
            )?;

            let account_provider2_option = Some(ProviderFactory::create_provider2(
                client.clone(),
                ChainId::new(node_info.net.chain_id().id()),
                &opt.account_provider,
            )?);

            let state = CliState::new(
                node_info.net,
                client,
                opt.watch_timeout.map(Duration::from_secs),
                node_handle,
                account_provider,
                account_provider2_option,
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
            print_logo();
            (*scmd::G_DEFAULT_CONSOLE_CONFIG, Some(state.history_file()))
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

#[rustfmt::skip]
#[allow(clippy::print_literal)]
fn print_logo(){
    println!("{}{}{}","\x1b[34;1m",r#"                                                                                
                                                      ::::::::███████::      
                                                :::::███████████████████:    
                                  :█████████:  :████████:::        :██████   
                                  █████:█████:  █::                  █████:  
                                 █████: :█████:                     :█████   
                                :█████   :████:                    :█████:   
                               :█████:    :████:                  :█████:    
                              :█████:      █████:               :██████      
               :███████████████████:       :███████████████:   ██████:       
              :████████████████████         :████████████:   :█████:         
              █████:                                      :██████:           
             ::███████::                                :██████:             
          :█::  ::████████:                          :███████:               
        :████:::    :████████:                    ::██████:                  
      :█████::         ::█████:                  :█████::                    
    :██████:            :████:                   :█████:                     
   :█████:             :█████           :         :█████:                    
  :████:              :█████:        :█████::      :█████:                   
 :████:              :█████:     :█████████████:    :█████                   
:█████               █████: :██████████: :████████:  :████:                  
:████:               :████████████::        ::████████████:                  
 :█████::::::::::::██   ::███::                 :██████:                     
  :█████████████████::                                                     
      :::█████::::

         ██████╗████████╗  ███╗  ██████╗  █████╗  █████╗ ██╗███╗  ██╗
        ██╔════╝╚══██╔══╝ ██ ██╗ ╚════██╗██╔══██╗██╔══██╗██║████╗ ██║
        ╚█████╗    ██║   ██   ██║██████╔╝██║  ╚═╝██║  ██║██║██╔██╗██║
         ╚═══██╗   ██║   ██╔══██║██╔══██╗██║  ██╗██║  ██║██║██║╚████║
        ██████╔╝   ██║   ██║  ██║██║  ██║╚█████╔╝╚█████╔╝██║██║ ╚███║
        ╚═════╝    ╚═╝   ╚═╝  ╚═╝╚═╝  ╚═╝ ╚════╝  ╚════╝ ╚═╝╚═╝  ╚══╝
"#,"\x1b[0m");
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(e) => {
            match e.downcast::<NodeStartError>() {
                Ok(e) => match e {
                    //TODO not suggest clean data dir in main network.
                    NodeStartError::LoadConfigError(e) => {
                        error!("{:?}, please fix config.", e);
                        std::process::exit(G_EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::StorageInitError(e) => {
                        error!("{:?}, please clean your data dir.", e);
                        std::process::exit(G_EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::GenesisError(e) => {
                        error!("{:?}, please clean your data dir.", e);
                        std::process::exit(G_EXIT_CODE_NEED_HELP);
                    }
                    NodeStartError::Other(e) => {
                        error!("Node exit for an unexpected error: {:?}", e);
                        std::process::exit(1);
                    }
                },
                Err(e) => match e.downcast::<CmdError>() {
                    Ok(e) => match e {
                        CmdError::ClapError(e) => {
                            println!("{}", e);
                        }
                        CmdError::Other(e) => {
                            error!("Starcoin cmd return error: {:?}", e);
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        error!("Starcoin cmd exits abnormally: {:?}", e);
                        std::process::exit(1);
                    }
                },
            }
        }
    }
}
