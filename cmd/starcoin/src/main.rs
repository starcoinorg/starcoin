// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use anyhow::Result;
use scmd::{CmdContext, Command, CommandAction};
use starcoin_logger::prelude::*;
use starcoin_node::Node;

pub use starcoin_config::StarcoinOpt;
use starcoin_consensus::{
    argon_consensus::{ArgonConsensus, ArgonConsensusHeader},
    Consensus, ConsensusHeader,
};
use starcoin_executor::{executor::Executor, TransactionExecutor};
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;

mod account;
mod debug;
mod txn;

mod helper;
pub mod state;

fn run<C, H, E>() -> Result<()>
where
    C: Consensus + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
{
    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        Box::new(|opt| -> Result<CliState> {
            let logger_handle = starcoin_logger::init();
            info!("Starcoin opts: {:?}", opt);
            let config = Arc::new(starcoin_config::load_config_with_opt(opt)?);
            let node = Node::<C, H, E>::new(config.clone());
            let handle = node.start();
            let ipc_file = config.rpc.get_ipc_file();
            info!("Waiting node start...");
            helper::wait_until_file_created(ipc_file)?;
            info!("Try to connect node by ipc: {:?}", ipc_file);
            let client = RpcClient::connect_ipc(ipc_file)?;
            let file_log_path = config.data_dir().join("starcoin.log");
            info!("Redirect log to file: {:?}", file_log_path);
            logger_handle.enable_file(false, file_log_path);
            let state = CliState::new(config, client, logger_handle, Some(handle));
            Ok(state)
        }),
        Box::new(|_, _, state| -> Result<()> {
            let (_, _, logger_handle, handle) = state.into_inner();
            match handle {
                Some(handle) => {
                    // if start node server and no subcommand, wait server and output logger to stderr.
                    logger_handle.enable_stderr();
                    handle.join().expect("Join thread error.");
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

//TODO error handle.
fn main() {
    match run::<ArgonConsensus, ArgonConsensusHeader, Executor>() {
        Ok(()) => {}
        Err(e) => panic!(format!("Unexpect error: {:?}", e)),
    }
}
