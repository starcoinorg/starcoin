// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::state::CliState;
use anyhow::Result;
use scmd::{CmdContext, Command, CommandAction};
use starcoin_logger::prelude::*;
use starcoin_node::Node;

use crate::account::account_new_cmd::AccountNewCommand;
pub use starcoin_config::StarcoinOpt;
use starcoin_consensus::{
    argon_consensus::{ArgonConsensus, ArgonConsensusHeader},
    Consensus, ConsensusHeader,
};
use starcoin_executor::{executor::Executor, TransactionExecutor};
use starcoin_rpc_client::RpcClient;
use std::sync::Arc;

pub mod account;
mod helper;
pub mod state;

fn run<C, H, E>() -> Result<()>
where
    C: Consensus + Sync + Send + 'static,
    H: ConsensusHeader + Sync + Send + 'static,
    E: TransactionExecutor + Sync + Send + 'static,
{
    starcoin_logger::init();

    let context = CmdContext::<CliState, StarcoinOpt>::with_default_action(
        Box::new(|opt| -> Result<CliState> {
            info!("Starcoin opts: {:?}", opt);
            let config = Arc::new(starcoin_config::load_config_with_opt(opt)?);
            let node = Node::<C, H, E>::new(config.clone());
            let handle = node.start();
            let ipc_file = config.rpc.get_ipc_file(&config.data_dir);
            info!("Waiting node start...");
            helper::wait_until_file_created(&ipc_file)?;
            info!("Try to connect node by ipc: {:?}", ipc_file);
            let client = RpcClient::connect_ipc(ipc_file)?;
            let state = CliState::new(config, client, Some(handle));
            Ok(state)
        }),
        Box::new(|_, _, state| -> Result<()> {
            let (_, _, handle) = state.into_inner();
            match handle {
                Some(handle) => {
                    handle.join().expect("Join thread error.");
                }
                None => {}
            }
            Ok(())
        }),
    );
    context
        .command(Command::with_name("account").subcommand(AccountNewCommand {}.into_cmd()))
        .exec()
}

//TODO error handle.
fn main() {
    match run::<ArgonConsensus, ArgonConsensusHeader, Executor>() {
        Ok(()) => {}
        Err(e) => panic!(format!("Unexpect error: {:?}", e)),
    }
}
